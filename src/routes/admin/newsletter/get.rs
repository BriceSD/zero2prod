use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn issue_newsletter_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut messages_html = String::new();
    for m in flash_messages.iter() {
        writeln!(messages_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    let idempotency_key = uuid::Uuid::new_v4();

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Issue newsletter</title>
</head>
<body>
    {}
    <form action="/admin/newsletter" method="post">
        <label>Title
            <input
            type="text"
            placeholder="Enter newsletter title"
            name="title"
            >
        </label>
        <br>

        <label>Content
            <input
            type="text"
            placeholder="Enter newsletter content"
            name="content_text"
            >
        </label>
        <br>

        <label>Content HTML
            <input
            type="text"
            placeholder="Enter newsletter content"
            name="content_html"
            >
        </label>
        <br>
        <input hidden type="text" name="idempotency_key" value="{}">
        <button type="submit">Issue newsletter</button>
    </form>
    <p><a href="/admin/dashboard">&lt;- Back</a></p>
</body>
</html>"#,
            messages_html, idempotency_key
        )))
}
