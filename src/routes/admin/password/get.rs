use actix_web::{HttpResponse, http::header::ContentType, web};
use tera::Tera;

use crate::{
    helpers::{e500, render_html, see_other},
    session_state::TypedSession,
};

pub async fn change_password_form(
    tera: web::Data<Tera>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }

    let rendered_html = match render_html(&tera, &[], "change_password.html".into()) {
        Ok(h) => h,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered_html))
}
