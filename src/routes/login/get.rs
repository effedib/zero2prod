use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, web};
use actix_web_flash_messages::IncomingFlashMessages;
use tera::{Context, Tera};

pub async fn login_form(
    tera: web::Data<Tera>,
    flash_messages: IncomingFlashMessages,
) -> HttpResponse {
    let mut messages: Vec<String> = vec![];
    // handle many messages
    for m in flash_messages.iter() {
        messages.push(m.content().to_string());
    }

    let rendered_html = match render_multiple_msg(&tera, messages, "login.html".into()) {
        Ok(html) => html,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered_html)
}

pub fn render_multiple_msg(
    tera: &Tera,
    args: Vec<String>,
    template_name: String,
) -> Result<String, tera::Error> {
    let mut context = Context::new();

    context.insert("messages", &args);

    tera.render(&template_name, &context)
}
