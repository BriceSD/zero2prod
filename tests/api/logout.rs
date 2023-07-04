use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn you_must_be_logged_in_to_logout() {
    // Given
    let app = spawn_app().await;

    // When
    let response = app.get_logout().await;

    // Then
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn logged_in_user_can_logout() {
    // Given
    let app = spawn_app().await;
    app.login_admin().await;

    // When
    let response = app.get_logout().await;

    // Then
    assert_is_redirect_to(&response, "/login");
}
