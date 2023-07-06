use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn change_password_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut messages_html = String::new();
    for m in flash_messages.iter() {
        writeln!(messages_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta http-equiv="content-type" content="text/html; charset=utf-8">
<title>Change Password</title>
</head>
<body>
    {messages_html}
<form action="/admin/change_password" method="post">
<label>Current password
<input
type="password"
placeholder="Enter current password"
name="current_password"
>
</label>
<br>
<label>New password
<input
type="password"
placeholder="Enter new password"
name="new_password"
>
</label>
<br>
<label>Confirm new password
<input
type="password"
placeholder="Type the new password again"
name="new_password_confirmation"
>
</label>
<br>
<button type="submit">Change password</button>
</form>
<p><a href="/admin/dashboard">&lt;- Back</a></p>
</body>
</html>"#,
        )))
}
