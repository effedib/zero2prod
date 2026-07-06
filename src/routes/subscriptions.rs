use actix_web::{HttpResponse, Responder, ResponseError, web};
use chrono::Utc;
use rand::{RngExt, distr::Alphanumeric, rng};
use sqlx::{Executor, PgPool, Postgres, Transaction};
use tera::Tera;
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, Subscriber},
    email_client::EmailClient,
    helpers::render_confirmation_email,
    startup::ApplicationBaseUrl,
};

#[derive(Debug)]
pub struct StoreTokenError(sqlx::Error);

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while trying to store a subscription token."
        )
    }
}

impl ResponseError for StoreTokenError {}

#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    tera: web::Data<Tera>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, actix_web::Error> {
    let new_subscriber = match form.0.try_into() {
        Ok(new_sub) => new_sub,
        Err(_) => return Ok(HttpResponse::BadRequest().finish()),
    };

    let mut transaction = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let subscriber_id = match insert_subscriber(&mut transaction, &new_subscriber, &pool).await {
        Ok(id) => id,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let subscription_token = generate_subscription_token();

    store_token(&mut transaction, subscriber_id, &subscription_token).await?;

    if transaction.commit().await.is_err() {
        return Ok(HttpResponse::InternalServerError().finish());
    }

    if send_confirmation_email(
        &tera,
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await
    .is_err()
    {
        return Ok(HttpResponse::InternalServerError().finish());
    };

    Ok(HttpResponse::Ok().finish())
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url, subscriptions_token)
)]
pub async fn send_confirmation_email(
    tera: &Tera,
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscriptions_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscriptions_token={}",
        base_url, subscriptions_token
    );

    let plain_body = &format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = render_confirmation_email(tera, confirmation_link.as_str())
        .expect("Impossible to render the confirmation email");
    email_client
        .send_email(
            new_subscriber.email,
            "Welcome!",
            html_body.as_str(),
            plain_body,
        )
        .await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transaction, pool)
)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
    pool: &PgPool,
) -> Result<Uuid, sqlx::Error> {
    if let Some(subscriber) = get_subscriber_from_email(pool, new_subscriber).await?
        && subscriber.status == "pending_confirmation"
    {
        return Ok(subscriber.id);
    }

    let subscriber = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'pending_confirmation')
    "#,
        subscriber,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now(),
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(subscriber)
}

pub fn generate_subscription_token() -> String {
    let mut rng = rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), StoreTokenError> {
    let query = sqlx::query!(
        r#"
        INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)
    "#,
        subscription_token,
        subscriber_id
    );

    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        StoreTokenError(e)
    })?;

    Ok(())
}

#[tracing::instrument(name = "Get subscriber id from email", skip(new_subscriber, pool))]
pub async fn get_subscriber_from_email(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<Option<Subscriber>, sqlx::Error> {
    let fetched_row: Option<Subscriber> = sqlx::query_as::<_, Subscriber>(
        r#"
            SELECT * FROM subscriptions
            WHERE email = $1
        "#,
    )
    .bind(new_subscriber.email.as_ref())
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(fetched_row)
}
