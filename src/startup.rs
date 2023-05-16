//! src/startup.rs

use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::{
    configuration::{DatabaseSettings, Setting},
    email_client::EmailClient,
    routes::{health_check, subscriptions},
};

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Setting) -> Result<Self, std::io::Error> {
        let db_connection_pool = get_connection_pool(&configuration.database);

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");

        let email_client = EmailClient::new(
            configuration.email_client.base_url.clone(),
            sender_email,
            configuration.email_client.authorization_token.clone(),
            std::time::Duration::from_millis(configuration.email_client.timeout_milliseconds),
        );
        tracing::info!("Using email client {:?}", &email_client);

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        tracing::info!("Using application address {:?}", &address);
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, db_connection_pool, email_client)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    // A more expressive name that makes it clear that
    // this function only returns when the application is stopped.
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    tracing::info!("Using {:?}", configuration);
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check::health_check))
            .route("/subscriptions", web::post().to(subscriptions::subscribe))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
