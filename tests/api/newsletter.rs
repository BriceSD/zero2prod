use crate::helpers::{assert_is_redirect_to, spawn_app, ConfirmationLinks, TestApp};
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn you_must_be_logged_in_to_issue_a_newsletter() {
    // Given
    let app = spawn_app().await;

    let issue_newsletter_body = serde_json::json!({
        "title": "Newsletter title",
        "content_text": "Newsletter body as plain text",
        "content_html": "<p>Newsletter body as HTML</p>",
    });

    // When
    let response_get = app.get_issue_newsletter().await;
    let response_post = app.post_issue_newsletter(&issue_newsletter_body).await;

    // Then
    assert_is_redirect_to(&response_get, "/login");
    assert_is_redirect_to(&response_post, "/login");
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    // Given
    let app = spawn_app().await;
    app.login_admin().await;

    let test_cases = vec![
        (
            serde_json::json!({
                "content_text": "Newsletter body as plain text",
                "content_html": "<p>Newsletter body as HTML</p>",
            }),
            "missing title",
        ),
        (
            serde_json::json!({"title": "Newsletter!"}),
            "missing content",
        ),
    ];

    // When
    for (invalid_body, error_message) in test_cases {
        let response = app.post_issue_newsletter(&invalid_body).await;

        // Then
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Given
    let app = spawn_app().await;
    app.login_admin().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        // We assert that no request is fired at Postmark!
        .expect(0)
        .mount(&app.email_server)
        .await;

    // When
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content_text": "Newsletter body as plain text",
        "content_html": "<p>Newsletter body as HTML</p>",
    });

    assert_post_redirect_with_message(
        &newsletter_request_body,
        "Successfully issued the newsletter",
        &app,
    )
    .await;
    // Mock verifies on Drop that we haven't sent the newsletter email
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Given
    let app = spawn_app().await;
    app.login_admin().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        // We assert that exactly 1 request is fired at Postmark!
        .expect(1)
        .mount(&app.email_server)
        .await;

    // When
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content_text": "Newsletter body as plain text",
        "content_html": "<p>Newsletter body as HTML</p>",
    });

    assert_post_redirect_with_message(
        &newsletter_request_body,
        "Successfully issued the newsletter",
        &app,
    )
    .await;
    // Mock verifies on Drop that we sent the newsletter email
}

/// Use the public API of the application under test to create
/// an unconfirmed subscriber.
async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.get_confirmation_links(email_request)
}

/// Use the public API of the application under test to create
/// a confirmed subscriber.
async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_links = create_unconfirmed_subscriber(app).await;

    // Confirm subscriber
    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

async fn assert_post_redirect_with_message(
    body: &serde_json::value::Value,
    message: &str,
    app: &TestApp,
) {
    // Issue the newsletter
    let response = app.post_issue_newsletter(&body).await;
    assert_is_redirect_to(&response, "/admin/newsletter");

    // Follow the redirect
    let html_page = app.get_issue_newsletter_html().await;
    assert!(html_page.contains(message));
}
