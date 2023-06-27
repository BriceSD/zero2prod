use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::{IncomingFlashMessages, Level};
use std::fmt::Write;

pub async fn change_password_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let mut error_html = String::new();
    for m in flash_messages.iter().filter(|m| m.level() == Level::Success) {
        writeln!(error_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }
    for m in flash_messages.iter().filter(|m| m.level() == Level::Error) {
        writeln!(error_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Reset password</title>
</head>
<body>
    {error_html}
    <form action="/admin/change_password" method="post">
    </label>
    <label>Password
        <input
            type="password"
            placeholder="Enter Password"
            name="password"
        >
    </label>
    <button type="submit">Reset</button>
    </form>
</body>
</html>
"#,
        ))
}
