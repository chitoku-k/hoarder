use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Config {
    pub port: u16,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing environment variable: {0}")]
    MissingValue(String),
    #[error("error reading environment variables: {0}")]
    Deserialize(String),
}

pub fn get() -> anyhow::Result<Config> {
    match envy::from_env() {
        Ok(config) => Ok(config),
        Err(envy::Error::MissingValue(field)) => Err(ConfigError::MissingValue(field.to_uppercase()))?,
        Err(envy::Error::Custom(e)) => Err(ConfigError::Deserialize(e))?,
    }
}
