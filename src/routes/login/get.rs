use actix_web::cookie::Cookie;
use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse, web};
use tera::{Context, Tera};

pub async fn login_form(tera: web::Data<Tera>, request: HttpRequest) -> HttpResponse {
    let error_html: String = match request.cookie("_flash") {
        None => "".into(),
        Some(c) => c.value().to_string(),
    };

    let rendered_html = match render_login_form(&tera, &error_html) {
        Ok(html) => html,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let mut response = HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered_html);

    response
        .add_removal_cookie(&Cookie::new("_flash", ""))
        .unwrap();

    response
}

pub fn render_login_form(tera: &Tera, error_html: &str) -> Result<String, tera::Error> {
    let mut context = Context::new();
    context.insert("error_html", error_html);

    tera.render("login.html", &context)
}
