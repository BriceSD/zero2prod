use actix_web::HttpResponse;
use actix_web::http::header::LOCATION;
use actix_web_flash_messages::FlashMessage;
use anyhow::anyhow;
use reqwest::StatusCode;

use crate::{
    routes::error_chain_fmt,
    session_state::TypedSession,
    utils::{e500, see_other},
};


#[tracing::instrument(
skip(session),
fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn logout(
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    let _user_id = if let Some(user_id) = session.get_user_id().map_err(e500)? {
        user_id
    } else {
         return Err(LogoutError::Unauthorized.into());
    };

    match session.remove_user_id() {
        Some(_) => {
            FlashMessage::success("Successfully logout".to_string()).send();
            Ok(see_other("/login"))
        }
        None => Err(LogoutError::UnexpectedError(anyhow!("Couldn't remove user from session")).into()),
    }
}

#[derive(thiserror::Error)]
pub enum LogoutError {
    #[error("Not authorized")]
    Unauthorized,
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

impl actix_web::error::ResponseError for LogoutError {
    fn error_response(&self) -> HttpResponse {
        match self {
            LogoutError::UnexpectedError(_) => {
                HttpResponse::build(self.status_code())
                    .insert_header((LOCATION, "/admin/dashboard"))
                    .finish()
            } 
            LogoutError::Unauthorized => {
                FlashMessage::error("You must be logged in").send();
                HttpResponse::SeeOther()
                    .insert_header((LOCATION, "/login"))
                    .finish()
            }
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            LogoutError::Unauthorized => StatusCode::FORBIDDEN,
            LogoutError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Debug for LogoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
