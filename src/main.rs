//! main.rs

use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

use zero2prod::{configuration::get_configuration, startup::run, telemetry, email_client::EmailClient};

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

    let sender_email = configuration.email_client.sender()
        .expect("Invalid sender email address");
    let base_url = configuration.email_client.base_url;
    let email_client = EmailClient::new(base_url, sender_email, configuration.email_client.authorization_token);


    run(listener, connection_pool, email_client)?.await
}
