use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, web};
use tera::{Context, Tera};

pub async fn home(tera: web::Data<Tera>) -> HttpResponse {
    let context = Context::new();
    let rendered_html = match tera.render("home.html", &context) {
        Ok(r) => r,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered_html)
}
