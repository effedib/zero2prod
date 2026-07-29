use actix_web::{HttpResponse, web};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::PgPool;

use crate::{
    authentication::UserId,
    domain::SubscriberEmail,
    email_client::EmailClient,
    helpers::{e500, see_other},
};

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    #[serde(rename = "html_content")]
    html: String,
    #[serde(rename = "text_content")]
    text: String,
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(body, pool, email_client),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    body: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    _user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let subscribers = get_confirmed_subscribers(&pool).await.map_err(e500)?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(&subscriber.email, &body.title, &body.html, &body.text)
                    .await
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })
                    .map_err(e500)?;
            }
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber. \
                    Their stored contact details are invalid",
                );
            }
        }
    }

    FlashMessage::info("The newsletter issue has been published!").send();
    Ok(see_other("/admin/newsletters"))
}

struct ConfirmedSubscriber {
    pub email: SubscriberEmail,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers = sqlx::query!(
        r#"
            SELECT email
            FROM subscriptions
            WHERE status = 'confirmed'
        "#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| match SubscriberEmail::parse(r.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(error) => Err(anyhow::anyhow!(error)),
    })
    .collect();

    Ok(confirmed_subscribers)
}

// #[derive(thiserror::Error)]
// pub enum PublishError {
//     #[error("Authentication failed")]
//     AuthError(#[source] anyhow::Error),
//     #[error(transparent)]
//     UnexpectedError(#[from] anyhow::Error),
// }
//
// impl std::fmt::Debug for PublishError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         error_chain_fmt(self, f)
//     }
// }
//
// impl ResponseError for PublishError {
//     fn error_response(&self) -> HttpResponse {
//         match self {
//             PublishError::UnexpectedError(_) => {
//                 HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
//             }
//             PublishError::AuthError(_) => {
//                 let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
//                 let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();
//
//                 response
//                     .headers_mut()
//                     .insert(header::WWW_AUTHENTICATE, header_value);
//
//                 response
//             }
//         }
//     }
// }

// fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
//     let header_value = headers
//         .get("Authorization")
//         .context("The 'Authorization' header was missing")?
//         .to_str()
//         .context("The 'Authorization' header was not a valid UTF8 string.")?;
//     let base64encoded_segment = header_value
//         .strip_prefix("Basic ")
//         .context("The authorization scheme was not 'Basic'.")?;
//     let decoded_bytes = base64::engine::general_purpose::STANDARD
//         .decode(base64encoded_segment)
//         .context("Failed to base64-decode 'Basic' credentials.")?;
//     let decoded_credentials = String::from_utf8(decoded_bytes)
//         .context("The decoded credential string is not valid UTF8.")?;
//
//     let mut credentials = decoded_credentials.splitn(2, ':');
//     let username = credentials
//         .next()
//         .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth."))?
//         .to_string();
//     let password = credentials
//         .next()
//         .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth."))?
//         .to_string();
//
//     Ok(Credentials {
//         username,
//         password: SecretString::new(password.into()),
//     })
// }
