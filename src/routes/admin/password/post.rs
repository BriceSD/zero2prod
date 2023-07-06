use actix_web::HttpResponse;
use actix_web::{http::header::LOCATION, web};
use actix_web_flash_messages::FlashMessage;
use anyhow::anyhow;
use reqwest::StatusCode;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

use crate::authentication::UserId;
use crate::{
    authentication::{verify_password_hash, AuthError},
    domain::AdminPassword,
    routes::error_chain_fmt,
    utils::{e500, see_other},
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_confirmation: Secret<String>,
}

#[tracing::instrument(
skip(form, pool),
fields(username = tracing::field::Empty, user_id = tracing::field::Empty)
)]
pub async fn change_password(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();

    let current_password = AdminPassword::parse(form.0.current_password.expose_secret().clone())
        .map_err(|_| ChangePasswordError::BadRequest(anyhow!("Wrong password")))?;
    let new_password = AdminPassword::parse(form.0.new_password.expose_secret().clone())
        .map_err(ChangePasswordError::BadRequest)?;
    let new_password_confirmation =
        AdminPassword::parse(form.0.new_password_confirmation.expose_secret().clone())
            .map_err(ChangePasswordError::BadRequest)?;

    if new_password.as_ref().expose_secret() != new_password_confirmation.as_ref().expose_secret() {
        return Err(ChangePasswordError::BadRequest(anyhow!(
            "You entered two different new passwords - the field values must match"
        ))
        .into());
    }

    let password_hash = if let Some(password_hash) =
        crate::authentication::get_stored_password_hash(*user_id, &pool)
            .await
            .map_err(e500)?
    {
        password_hash
    } else {
        return Err(e500("User doesn't have a password set"));
    };

    verify_password_hash(&password_hash, current_password.as_ref()).map_err(|e| match e {
        AuthError::InvalidCredentials(_) => {
            ChangePasswordError::BadRequest(anyhow!("Wrong password"))
        }
        AuthError::UnexpectedError(_) => ChangePasswordError::UnexpectedError(e.into()),
    })?;

    match crate::authentication::update_password(*user_id, new_password.as_ref(), &pool).await {
        Ok(_) => {
            FlashMessage::success("Successfully changed password".to_string()).send();
            Ok(see_other("/admin/change_password"))
        }
        Err(e) => Err(e500(e)),
    }
}

#[derive(thiserror::Error)]
pub enum ChangePasswordError {
    #[error("Invalid argument")]
    BadRequest(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

impl actix_web::error::ResponseError for ChangePasswordError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ChangePasswordError::BadRequest(e) => {
                FlashMessage::error(e.to_string()).send();
                HttpResponse::SeeOther()
                    .insert_header((LOCATION, "/admin/change_password"))
                    .finish()
            }
            ChangePasswordError::UnexpectedError(_) => HttpResponse::build(self.status_code())
                .insert_header((LOCATION, "/admin/change_password"))
                .finish(),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ChangePasswordError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ChangePasswordError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Debug for ChangePasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
