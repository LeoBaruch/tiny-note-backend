use crate::config::Config;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};

#[derive(Debug)]
pub enum DbError {
    Pool(sqlx::Error),
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::Pool(e) => write!(f, "mysql pool error: {}", e),
        }
    }
}

impl std::error::Error for DbError {}

impl From<sqlx::Error> for DbError {
    fn from(e: sqlx::Error) -> Self { DbError::Pool(e) }
}

pub async fn create_mysql_pool(cfg: &Config) -> Result<MySqlPool, DbError> {
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&cfg.database_url)
        .await?;
    Ok(pool)
}