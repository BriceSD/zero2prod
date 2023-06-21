//! main.rs

use zero2prod::{
    configuration::get_configuration, startup::Application, telemetry,
};

/// Main app, will panic if no configuration file is found 
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber =
        telemetry::get_log_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_log_subscriber(subscriber);

    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration.");

    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
