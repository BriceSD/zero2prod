use actix_web::{error::InternalError, HttpResponse};
use actix_web::{http::header::LOCATION, web};
use actix_web_flash_messages::FlashMessage;
use anyhow::{anyhow, Context};
use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, PasswordHasher, Version};
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{routes::error_chain_fmt, session_state::TypedSession};

#[derive(serde::Deserialize)]
pub struct FormData {
    password: Secret<String>,
}

#[tracing::instrument(
skip(form, pool, session),
fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn change_password(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, InternalError<PasswordResetError>> {
    let user_id = if let Some(user_id) = session
        .get_user_id()
        .map_err(|e| login_redirect(PasswordResetError::UnexpectedError(e.into())))?
    {
        user_id
    } else {
        return Err(login_redirect(PasswordResetError::Unauthorized(anyhow!(
            "Not logged in"
        ))));
    };

    let new_password = form.0.password;
    match update_password(user_id, new_password, &pool).await {
        Ok(_) => {
            FlashMessage::success("Successfully changed password".to_string()).send();
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/admin/change_password"))
                .finish())
        }
        Err(e) => {
            let e = PasswordResetError::UnexpectedError(e);

            FlashMessage::error(e.to_string()).send();
            let response = HttpResponse::SeeOther()
                .insert_header((LOCATION, "/admin/change_password"))
                .finish();

            Err(InternalError::from_response(e, response))
        }
    }
}

#[tracing::instrument(name = "Update password", skip(pool))]
async fn update_password(
    user_id: Uuid,
    password: Secret<String>,
    pool: &PgPool,
) -> Result<(), anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.expose_secret().as_bytes(), &salt)
    .unwrap()
    .to_string();

    sqlx::query!(
        r#"
UPDATE users
SET password_hash = $1
WHERE user_id = $2
"#,
        password_hash,
        user_id,
    )
    .execute(pool)
    .await
    .context("Failed to perform a query to update a user password.")?;

    Ok(())
}

// Redirect to the password page with an error message.
fn login_redirect(e: PasswordResetError) -> InternalError<PasswordResetError> {
    FlashMessage::error(e.to_string()).send();
    let response = HttpResponse::SeeOther()
        .insert_header((LOCATION, "/login"))
        .finish();
    InternalError::from_response(e, response)
}

#[derive(thiserror::Error)]
pub enum PasswordResetError {
    #[error("Not authorized")]
    Unauthorized(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PasswordResetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
