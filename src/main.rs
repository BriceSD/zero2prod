//! main.rs

use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

use zero2prod::{
    configuration::get_configuration, email_client::EmailClient, startup::run, telemetry,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber =
        telemetry::get_log_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_log_subscriber(subscriber);

    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration.");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    tracing::info!("Using application address {:?}", &address);
    let listener = TcpListener::bind(address)?;

    tracing::info!("Using {:?}", configuration.database);
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let email_client = EmailClient::new(
        configuration.email_client.base_url.clone(),
        configuration.email_client.sender().expect("Invalid sender email address"),
        configuration.email_client.authorization_token,
        std::time::Duration::from_millis(configuration.email_client.timeout_milliseconds),
    );
    tracing::info!("Using email client {:?}", email_client);

    run(listener, connection_pool, email_client)?.await
}
