use actix_web::HttpResponse;
use actix_web_flash_messages::FlashMessage;

use crate::{session_state::TypedSession, utils::see_other};

#[tracing::instrument(
    skip(session),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn logout(session: TypedSession) -> Result<HttpResponse, actix_web::Error> {
    session.logout();
    FlashMessage::success("Successfully logged out".to_string()).send();
    Ok(see_other("/login"))
}
