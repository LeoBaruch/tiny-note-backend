use std::env;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub port: u16,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("missing env var: {0}")]
    MissingEnv(String),
    #[error("invalid port: {0}")]
    InvalidPort(String),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnv("DATABASE_URL".into()))?;
        let redis_url = env::var("REDIS_URL")
            .map_err(|_| ConfigError::MissingEnv("REDIS_URL".into()))?;
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| ConfigError::MissingEnv("JWT_SECRET".into()))?;
        let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let port = port
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidPort(port))?;
        Ok(Self {
            database_url,
            redis_url,
            jwt_secret,
            port,
        })
    }
}