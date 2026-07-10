use crate::{domain::Parameters, helpers::error_chain_fmt};

use actix_web::{HttpResponse, ResponseError, http::StatusCode, web};
use anyhow::Context;
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error)]
pub enum ConfirmSubscriberError {
    #[error("There is no subscriber associated with the provided token.")]
    UnknownToken,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for ConfirmSubscriberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for ConfirmSubscriberError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::UnknownToken => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, pool))]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ConfirmSubscriberError> {
    let id = get_subscriber_id_from_token(&pool, &parameters.subscriptions_token)
        .await
        .context("Failed to retrieve the subscriber associated with the provided token")?
        .ok_or(ConfirmSubscriberError::UnknownToken)?;

    confirm_subscriber(&pool, id)
        .await
        .context("Failed to update the subscriber status to 'confirmed'.")?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(id, pool))]
pub async fn confirm_subscriber(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            UPDATE subscriptions SET status = 'confirmed' WHERE id = $1 AND status = 'pending_confirmation'
        "#,
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(token, pool))]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let fetched_row = sqlx::query!(
        r#"
            SELECT subscriber_id FROM subscription_tokens
            WHERE subscription_token = $1
        "#,
        token
    )
    .fetch_optional(pool)
    .await?;

    Ok(fetched_row.map(|r| r.subscriber_id))
}
