//! main.rs

use std::net::TcpListener;
use secrecy::ExposeSecret;
use sqlx::PgPool;

use zero2prod::{configuration::get_configuration, startup::run, telemetry};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = telemetry::get_log_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_log_subscriber(subscriber);

    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration.");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;

    let connection_pool = PgPool::connect(configuration.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");

    run(listener, connection_pool)?.await
}
