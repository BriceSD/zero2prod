use claims::assert_ok;
use secrecy::Secret;
use zero2prod::authentication::verify_password_hash;

use crate::helpers::{assert_is_redirect_to, spawn_app, TestApp};

#[tokio::test]
async fn you_must_be_logged_in_to_reset_password() {
    // Given
    let app = spawn_app().await;

    let change_password_body = serde_json::json!({
        "current_password": "",
        "new_password": "",
        "new_password_confirmation": "",
    });

    // When - Access Change Password page
    let response = app.get_change_password().await;

    // Then
    assert_is_redirect_to(&response, "/login");

    // When - Change Password
    let response = app.post_change_password(&change_password_body).await;

    // Then
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn new_passwords_should_match() {
    // Given
    let app = spawn_app().await;
    app.login_admin().await;

    let new_password = "new-random-password";
    let change_password_body = serde_json::json!({
        "current_password": &app.test_user.password,
        "new_password": new_password,
        "new_password_confirmation": format!("x{}", new_password),
    });

    // When change, then
    assert_post_redirect_with_message(
        &change_password_body,
        "You entered two different new passwords - the field values must match",
        &app,
    )
    .await;
}

#[tokio::test]
async fn user_must_provide_current_password() {
    // Given
    let app = spawn_app().await;
    app.login_admin().await;

    let new_password = "new-random-password";
    let change_password_body = serde_json::json!({
        "current_password": format!("x{}", &app.test_user.password),
        "new_password": new_password,
        "new_password_confirmation": new_password,
    });

    // When change, then
    assert_post_redirect_with_message(&change_password_body, "Wrong password", &app).await;
}

#[tokio::test]
async fn bad_new_password_is_redirected_with_error() {
    // Given
    let app = spawn_app().await;
    app.login_admin().await;

    // Min size is 12
    let too_small = serde_json::json!({
        "current_password": &app.test_user.password,
        "new_password": "a",
        "new_password_confirmation": "a",
    });

    // Max size is 128
    let too_long = serde_json::json!({
        "current_password": &app.test_user.password,
        "new_password": "a".repeat(129),
        "new_password_confirmation": "a".repeat(129),
    });

    let test_cases = vec![(too_small, "too short"), (too_long, "too long")];

    // When
    for (invalid_body, error_message) in test_cases {
        // Then
        assert_post_redirect_with_message(&invalid_body, error_message, &app).await;
    }
}

#[tokio::test]
async fn logged_in_user_can_reset_password() {
    // Given
    let app = spawn_app().await;
    app.login_admin().await;

    let new_password = "new-random-password";
    let change_password_body = serde_json::json!({
        "current_password": &app.test_user.password,
        "new_password": new_password,
        "new_password_confirmation": new_password,
    });

    // When change, then
    assert_post_redirect_with_message(&change_password_body, "Successfully changed password", &app)
        .await;

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
        &Secret::new(saved.password_hash),
        &Secret::new(new_password.into()),
    ));
}

async fn assert_post_redirect_with_message(
    body: &serde_json::value::Value,
    message: &str,
    app: &TestApp,
) {
    // Change Password
    let response = app.post_change_password(&body).await;
    assert_is_redirect_to(&response, "/admin/change_password");

    // Follow the redirect
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains(message));
}
