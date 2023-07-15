use config::{Config, ConfigError};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
//fied_attributes - Contains helpers for the fields. - Deserializes a number from string or a number.
//Environment variables are strings for the config
//crate and it will fail to pick up integers if using the standard deserialization routine from serde.
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::{
    postgres::{PgConnectOptions, PgSslMode},
    ConnectOptions,
};
use std::convert::From;
//name of fields should match 1:1 with yaml,
//application_port , database
#[derive(Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub db_settings: DatabaseSettings,
}
#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    pub database_name: String,
    //only being done to take care of env vars which are passed as string, so even a number is parsed as a string
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub require_ssl: bool,
}
#[derive(Deserialize)]
pub struct ApplicationSettings {
    //only being done to take care of env vars which are passed as string, so even a number is parsed as a string
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}
pub enum Environment {
    Local,
    Production,
}
impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(environment_str: String) -> Result<Self, Self::Error> {
        match environment_str.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!("{} is not a supported environment", other)),
        }
    }
}
impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Production => "production",
        }
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let base_path = std::env::current_dir().expect("cannot get current directory");
    let config_dir = dbg!(base_path.join("configuration"));
    //detect running environment
    //gets a string from env::var or defaults to local calls into to conver ti to a string, then calls
    //tryinto <environment> for String
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("failed to parse APP_ENVIRONMENT");
    let environment_filename = format!("{}.yaml", environment.as_str());

    let settings = Config::builder()
        .add_source(config::File::from(config_dir.join("base.yaml")))
        .add_source(config::File::from(config_dir.join(&environment_filename)))
        // Add in settings from environment variables (with a prefix of APP and
        // '__' as separator)
        // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;
    settings.try_deserialize::<Settings>()
}
impl DatabaseSettings {
    pub fn connection_string(&self) -> PgConnectOptions {
        let mut options = self
            .connection_string_without_db()
            .database(&self.database_name);
        //tune sqlx instrumentation to lower filter level from into to trace
        //to eliminate noise
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }
    //Omitting the database name we connect to the Postgres instance, not a specific logical database.
    pub fn connection_string_without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }
}
