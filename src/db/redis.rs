use crate::config::Config;
use redis::{AsyncCommands, Client};

#[derive(Debug)]
pub enum RedisError {
    Client(redis::RedisError),
}

impl std::fmt::Display for RedisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RedisError::Client(e) => write!(f, "redis client error: {}", e),
        }
    }
}

impl std::error::Error for RedisError {}

impl From<redis::RedisError> for RedisError {
    fn from(e: redis::RedisError) -> Self { RedisError::Client(e) }
}

pub fn create_redis_client(cfg: &Config) -> Result<Client, RedisError> {
    let client = Client::open(cfg.redis_url.clone())?;
    Ok(client)
}

pub async fn is_token_blacklisted(client: &Client, jti: &str) -> Result<bool, RedisError> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    let exists: i64 = conn.exists(format!("bl:{}", jti)).await?;
    Ok(exists > 0)
}

#[allow(dead_code)]
pub async fn blacklist_token(client: &Client, jti: &str, ttl_seconds: i64) -> Result<(), RedisError> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    let key = format!("bl:{}", jti);
    let _: () = conn.set_ex(key, 1, ttl_seconds as u64).await?;
    Ok(())
}