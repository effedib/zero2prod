use actix_web::{HttpResponse, http::header::ContentType, web};
use anyhow::Context;
use sqlx::PgPool;
use tera::Tera;
use uuid::Uuid;

use crate::{
    authentication::UserId,
    helpers::{e500, render_html},
};

pub async fn admin_dashboard(
    tera: web::Data<Tera>,
    user_id: web::ReqData<UserId>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    let username = get_username(*user_id, &pool).await.map_err(e500)?;

    let rendered_html = match render_html(
        &tera,
        &[("username", username.as_str())],
        "dashboard.html".into(),
    ) {
        Ok(html) => html,
        Err(_) => return Err(e500("error while trying to render the dashboard html")),
    };

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered_html))
}

#[tracing::instrument(name = "Get username", skip(pool))]
pub async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
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
