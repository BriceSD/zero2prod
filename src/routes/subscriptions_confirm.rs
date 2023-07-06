use actix_web::{web, HttpResponse};
use anyhow::Context;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SubscriberToken(String);

impl SubscriberToken {
    pub fn parse(token: String) -> Result<SubscriberToken, ParseTokenError> {
        if validate_token(&token) {
            Ok(Self(token))
        } else {
            Err(ParseTokenError(anyhow::anyhow!(
                "The subscriber token format is not valid",
            )))
        }
    }
}

fn validate_token(token: &str) -> bool {
    token.len() == 25 && token.is_ascii()
}

impl AsRef<str> for SubscriberToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

pub struct ParseTokenError(anyhow::Error);

impl std::error::Error for ParseTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}

impl std::fmt::Display for ParseTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "An error was encountered while \
                trying to parse a subscription token."
        )
    }
}

impl std::fmt::Debug for ParseTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters))]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ConfirmSubscriptionError> {
    let token = SubscriberToken::parse(parameters.0.subscription_token)
        .map_err(ConfirmSubscriptionError::InvalidTokenFormat)?;

    let subscriber_id = get_subscriber_id_from_token(&pool, &token)
        .await
        .context("Failed to retrieve subscriber for associated token")?
        .ok_or(ConfirmSubscriptionError::UnknownToken)?;

    confirm_subscriber(&pool, subscriber_id)
        .await
        .context("Failed to update token to `confirmed`")?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, pool))]
pub async fn confirm_subscriber(
    pool: &PgPool,
    subscriber_id: Uuid,
) -> Result<(), UpdateTokenStatusError> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id,
    )
    .execute(pool)
    .await
    .map_err(UpdateTokenStatusError)?;

    Ok(())
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(subscription_token, pool))]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &SubscriberToken,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token.as_ref(),
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| r.subscriber_id))
}

#[derive(thiserror::Error)]
pub enum ConfirmSubscriptionError {
    #[error("There is no subscriber associated with the provided token")]
    UnknownToken,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error(transparent)]
    InvalidTokenFormat(#[from] ParseTokenError),
}

impl std::fmt::Debug for ConfirmSubscriptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl actix_web::error::ResponseError for ConfirmSubscriptionError {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            ConfirmSubscriptionError::UnknownToken => reqwest::StatusCode::UNAUTHORIZED,
            ConfirmSubscriptionError::InvalidTokenFormat(_) => reqwest::StatusCode::BAD_REQUEST,
            ConfirmSubscriptionError::UnexpectedError(_) => {
                reqwest::StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

pub struct UpdateTokenStatusError(sqlx::Error);

impl std::error::Error for UpdateTokenStatusError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // The compiler transparently casts `&sqlx::Error` into a `&dyn Error`
        Some(&self.0)
    }
}

impl std::fmt::Display for UpdateTokenStatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while \
                trying to update the status of a subscription token."
        )
    }
}

impl std::fmt::Debug for UpdateTokenStatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
