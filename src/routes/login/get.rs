use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, web};
use tera::{Context, Tera};

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: Option<String>,
}

pub async fn login_form(tera: web::Data<Tera>, query: web::Query<QueryParams>) -> HttpResponse {
    let error_html = match query.0.error {
        None => "".to_string(),
        Some(error_message) => error_message,
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
