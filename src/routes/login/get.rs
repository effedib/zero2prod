use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, web};
use hmac::{Hmac, KeyInit, Mac};
use secrecy::ExposeSecret;
use tera::{Context, Tera};

use crate::startup::HmacSecret;

pub async fn login_form(
    tera: web::Data<Tera>,
    query: Option<web::Query<QueryParams>>,
    secret: web::Data<HmacSecret>,
) -> HttpResponse {
    let error_html = match query {
        None => "".to_string(),
        Some(query) => match query.0.verify(&secret) {
            Ok(error) => htmlescape::encode_minimal(&error),
            Err(e) => {
                tracing::warn!(
                    error.message = %e,
                        error.cause_chain = ?e,
                        "Failed to verify query parameters using the HMAC tag"
                );
                "".to_string()
            }
        },
    };

    let rendered_html = match render_login_form(&tera, error_html.as_str()) {
        Ok(html) => html,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered_html)
}

pub fn render_login_form(tera: &Tera, error_html: &str) -> Result<String, tera::Error> {
    let mut context = Context::new();
    context.insert("error_html", error_html);

    tera.render("login.html", &context)
}

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: String,
    tag: String,
}

impl QueryParams {
    fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag)?;
        let query_string = format!("error={}", urlencoding::Encoded::new(&self.error));

        let mut mac =
            Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)?;

        Ok(self.error)
    }
}
