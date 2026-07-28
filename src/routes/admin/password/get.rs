use actix_web::{HttpResponse, http::header::ContentType, web};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;
use tera::Tera;

use crate::helpers::render_html;

pub async fn change_password_form(
    tera: web::Data<Tera>,
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = String::new();
    for m in flash_messages.iter() {
        write!(msg_html, "{}", m.content()).unwrap();
    }

    let rendered_html = match render_html(
        &tera,
        &[("msg_html", &msg_html)],
        "change_password.html".into(),
    ) {
        Ok(h) => h,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered_html))
}
