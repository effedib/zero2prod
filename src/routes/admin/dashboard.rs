use actix_web::{
    HttpResponse,
    http::header::{ContentType, LOCATION},
    web,
};
use anyhow::Context;
use sqlx::PgPool;
use tera::Tera;
use uuid::Uuid;

use crate::session_state::TypedSession;

fn e500<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorInternalServerError(e)
}

pub async fn admin_dashboard(
    tera: web::Data<Tera>,
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let username = if let Some(user_id) = session.get_user_id().map_err(e500)? {
        get_username(user_id, &pool).await.map_err(e500)?
    } else {
        return Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, "/login"))
            .finish());
    };

    let rendered_html = match render_dashboard_form(&tera, username.as_str()) {
        Ok(html) => html,
        Err(e) => return Err(e500(e)),
    };

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered_html))
}

pub fn render_dashboard_form(tera: &Tera, username: &str) -> Result<String, String> {
    let mut context = tera::Context::new();
    context.insert("username", username);

    tera.render("dashboard.html", &context)
        .map_err(|_| "error while trying to render the dashboard html".into())
}

#[tracing::instrument(name = "Get username", skip(pool))]
async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
    SELECT username
    FROM users
    WHERE user_id = $1
    "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to perform a query to retrieve a username")?;

    Ok(row.username)
}
