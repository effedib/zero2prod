use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, web};
use tera::Tera;

use crate::helpers::render_html;

pub async fn home(tera: web::Data<Tera>) -> HttpResponse {
    let rendered_html = match render_html(&tera, &[], "home.html".into()) {
        Ok(r) => r,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered_html)
}
