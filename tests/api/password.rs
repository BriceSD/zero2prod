use claims::assert_ok;
use secrecy::Secret;
use zero2prod::authentication::verify_password_hash;

use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn you_must_be_logged_in_to_reset_password() {
    // Given
    let app = spawn_app().await;

    let reset_password_body = serde_json::json!({
        "password": "random-password"
    });

    // When - Part 1 - Reset Password
    let response = app.post_change_password(&reset_password_body).await;
    assert_is_redirect_to(&response, "/login");

    // When - Part 2 - Follow the redirect
    let html_page = app.get_login_html().await;
    assert!(html_page.contains("<p><i>Not authorized</i></p>"));
}

#[tokio::test]
async fn logged_in_user_can_reset_password() {
    // Given
    let app = spawn_app().await;
    app.login_admin().await;

    let new_password = "new-random-password";
    let reset_password_body = serde_json::json!({ "password": new_password });

    // When - Part 1 - Reset Password
    let response = app.post_change_password(&reset_password_body).await;
    assert_is_redirect_to(&response, "/admin/change_password");

    // When - Part 2 - Follow the redirect
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("Successfully changed password"));

    // Then password is updated in the database
    let saved = sqlx::query!(
        r#"
        SELECT password_hash
        FROM users
        WHERE user_id = $1
        "#,
        &app.test_user.user_id
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved subscription.");

    assert_ok!(verify_password_hash(
        Secret::new(saved.password_hash),
        Secret::new(new_password.into()),
    ));
}
