//! src/configuration.rs

use actix_web::Result;
use config::Config;

#[derive(serde::Deserialize)]
pub struct Setting {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
    
}

pub fn get_configuration() -> Result<Setting, config::ConfigError> {
    // Add configuration values from a file named 'configuration'
    // It will look for any top-level file with an extension
    // that 'config' knows how to parse: yaml, json, etc
    let settings = Config::builder()
        .add_source(config::File::with_name("configuration")).build()?;

    // Try to convert the configuration values it read into
    // our Settings type
    settings.try_deserialize()
}
