use claims::assert_ok;
use secrecy::Secret;
use zero2prod::authentication::verify_password_hash;

use crate::helpers::{assert_is_redirect_to, spawn_app, TestApp};

#[tokio::test]
async fn you_must_be_logged_in_to_logout() {
    // Given
    let app = spawn_app().await;

    // When - Part 1 - Change Password
    let response = app.get_logout().await;
    assert_is_redirect_to(&response, "/login");

    // When - Part 2 - Follow the redirect
    let html_page = app.get_login_html().await;
    assert!(html_page.contains("You must be logged in"));
}

#[tokio::test]
async fn logged_in_user_can_logout() {
    // Given
    let app = spawn_app().await;
    app.login_admin().await;

    // Change Password
    let response = app.get_logout().await;
    assert_is_redirect_to(&response, "/login");

    // Follow the redirect
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("Successfully logged out"));
}
