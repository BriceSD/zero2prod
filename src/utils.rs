use actix_web::http::header::LOCATION;
use actix_web::HttpResponse;
use actix_web_flash_messages::FlashMessage;

// Return an opaque 500 while preserving the error root's cause for logging.
pub fn e500<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorInternalServerError(e)
}

pub fn e400<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorBadRequest(e)
}

pub fn see_other(location: &str) -> HttpResponse {
            FlashMessage::error("You must be logged in".to_string()).send();
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}
