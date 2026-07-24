use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, web};
use actix_web_flash_messages::{IncomingFlashMessages, Level};
use std::fmt::Write;
use tera::Tera;

use crate::helpers::render_html;

pub async fn login_form(
    tera: web::Data<Tera>,
    flash_messages: IncomingFlashMessages,
) -> HttpResponse {
    let mut error_html = String::new();
    for m in flash_messages.iter().filter(|m| m.level() == Level::Error) {
        write!(error_html, "{}", m.content()).unwrap()
    }

    let rendered_html = match render_html(
        &tera,
        &[("error_html", error_html.as_str())],
        "login.html".into(),
    ) {
        Ok(html) => html,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered_html)
}
