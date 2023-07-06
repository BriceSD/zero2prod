use actix_web::HttpResponse;
use actix_web::{http::StatusCode, web::ReqData};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    authentication::UserId,
    idempotency::{save_response, try_processing, IdempotencyKey, NextAction},
    routes::error_chain_fmt,
    utils::{e400, e500, see_other},
};

#[derive(serde::Deserialize, std::fmt::Debug)]
pub struct FormData {
    title: String,
    content_html: String,
    content_text: String,
    idempotency_key: String,
}

#[tracing::instrument(
name = "Publish a newsletter issue",
skip_all,
fields(user_id = % & * user_id)
)]
pub async fn issue_newsletter(
    form: actix_web::web::Form<FormData>,
    pool: actix_web::web::Data<PgPool>,
    user_id: ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let FormData {
        title,
        content_text,
        content_html,
        idempotency_key,
    } = form.0;
    let user_id = user_id.into_inner();
    let idempotency_key: IdempotencyKey = idempotency_key.try_into().map_err(e400)?;

    let mut transaction = match try_processing(&pool, &idempotency_key, *user_id)
        .await
        .map_err(e500)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(saved_response) => {
            success_message().send();
            return Ok(saved_response);
        }
    };

    let issue_id = insert_newsletter_issue(&mut transaction, &title, &content_text, &content_html)
        .await
        .context("Failed to store newsletter issue details")
        .map_err(e500)?;

    enqueue_delivery_tasks(&mut transaction, issue_id)
        .await
        .context("Failed to enqueue delivery tasks")
        .map_err(e500)?;

    let response = see_other("/admin/newsletter");
    let response = save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(e500)?;

    success_message().send();
    Ok(response)
}

#[tracing::instrument(skip_all)]
async fn insert_newsletter_issue(
    transaction: &mut Transaction<'_, Postgres>,
    title: &str,
    text_content: &str,
    html_content: &str,
) -> Result<Uuid, sqlx::Error> {
    let newsletter_issue_id = Uuid::new_v4();
    sqlx::query!(
        r#"
INSERT INTO newsletter_issues (
    newsletter_issue_id,
    title,
    text_content,
    html_content,
    published_at
)
VALUES ($1, $2, $3, $4, now())
"#,
        newsletter_issue_id,
        title,
        text_content,
        html_content
    )
    .execute(transaction)
    .await?;
    Ok(newsletter_issue_id)
}

#[tracing::instrument(skip_all)]
async fn enqueue_delivery_tasks(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
INSERT INTO issue_delivery_queue (
    newsletter_issue_id,
    subscriber_email
)
SELECT $1, email
FROM subscriptions
WHERE status = 'confirmed'
"#,
        newsletter_issue_id,
    )
    .execute(transaction)
    .await?;
    Ok(())
}

fn success_message() -> FlashMessage {
    FlashMessage::info(
        "The newsletter issue has been accepted - \
        emails will go out shortly.",
    )
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Authentication failed.")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl actix_web::ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse {
        match self {
            PublishError::UnexpectedError(_) => {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            }
            PublishError::AuthError(_) => HttpResponse::new(StatusCode::UNAUTHORIZED),
        }
    }
}
