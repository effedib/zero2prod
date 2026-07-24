use actix_web::{HttpResponse, ResponseError, http::StatusCode, web};
use anyhow::Context;
use chrono::Utc;
use rand::{RngExt, distr::Alphanumeric, rng};
use sqlx::{Executor, PgPool, Postgres, Transaction};
use tera::Tera;
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, Subscriber},
    email_client::EmailClient,
    helpers::{error_chain_fmt, render_html},
    startup::ApplicationBaseUrl,
};

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscribeError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

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
) -> Result<HttpResponse, SubscribeError> {
    let new_subscriber = form.0.try_into().map_err(SubscribeError::ValidationError)?;

    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber, &pool)
        .await
        .context("Failed to insert a new subscriber in the database")?;

    let subscription_token = generate_subscription_token();

    store_token(&mut transaction, subscriber_id, &subscription_token)
        .await
        .context("Failed to store the confirmation token for a new subscriber")?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber")?;

    send_confirmation_email(
        &tera,
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await
    .context("Failed to send a confirmation email")?;

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
    let html_body = render_html(
        tera,
        &[("confirmation_link", confirmation_link.as_str())],
        "confirmation.html".into(),
    )
    .expect("Impossible to render the confirmation email");

    email_client
        .send_email(
            &new_subscriber.email,
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

    transaction.execute(query).await?;

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
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)
    "#,
        subscription_token,
        subscriber_id
    );

    transaction.execute(query).await?;

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
