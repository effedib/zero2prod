use crate::authentication::{AuthError, Credentials, validate_credentials};
use crate::helpers::{e500, see_other};
use crate::routes::dashboard::get_username;
use crate::session_state::TypedSession;
use actix_web::{HttpResponse, web};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: SecretString,
    new_password: SecretString,
    new_password_check: SecretString,
}

pub async fn change_password(
    form: web::Form<FormData>,
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = match session.get_user_id().map_err(e500)? {
        Some(u) => u,
        None => return Ok(see_other("/login")),
    };

    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        FlashMessage::error(
            "You entered two different new passwords - the fields values must match.",
        )
        .send();
        return Ok(see_other("/admin/password"));
    }

    let username = get_username(user_id, &pool).await.map_err(e500)?;
    let credentials = Credentials {
        username,
        password: form.0.current_password,
    };

    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect").send();
                Ok(see_other("/admin/password"))
            }
            AuthError::UnexpectedError(_) => Err(e500(e)),
        };
    }

    if !validate_new_password(form.0.new_password.expose_secret()).await {
        FlashMessage::error("The new password must be long between 12 and 129 chars").send();
        return Ok(see_other("/admin/password"));
    }

    crate::authentication::change_password(user_id, form.0.new_password, &pool)
        .await
        .map_err(e500)?;
    FlashMessage::info("Your password has been changed.").send();
    Ok(see_other("/admin/password"))
}

async fn validate_new_password(new_pwd: &str) -> bool {
    new_pwd.len() > 11 && new_pwd.len() < 129
}
