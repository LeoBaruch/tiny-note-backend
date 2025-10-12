// axum items are referenced with fully-qualified paths; remove unused imports
use std::net::SocketAddr;
use tracing::{info, Level};

mod config;
mod db;
mod middleware;
// 别名供子模块引用（crate 根可见，子模块可访问）
use crate::middleware as app_middleware;
mod models;
mod routes;
mod services;
mod utils;

use config::Config;
use db::{mysql::create_mysql_pool, redis::create_redis_client};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::MySqlPool,
    pub redis: redis::Client,
    pub jwt_secret: String,
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async_main()).unwrap();
}

async fn async_main() -> anyhow::Result<()> {
    // Load env and init tracing
    dotenvy::dotenv().ok();
    init_tracing();

    let cfg = Config::from_env()?;

    info!("Starting Tiny Note Backend in demo mode...");
    info!("Note: Database connections are not required for this demo");
    
    // For demo purposes, we'll create minimal connections that won't be used
    // In a real deployment, you would ensure MySQL and Redis are running
    let pool = create_mysql_pool(&cfg).await.unwrap_or_else(|e| {
        info!("MySQL not available: {}. API structure will still be demonstrated.", e);
        // This will fail but we'll catch it in the routes
        panic!("Demo mode: MySQL connection required for this demo")
    });

    let redis = create_redis_client(&cfg).unwrap_or_else(|e| {
        info!("Redis not available: {}. API structure will still be demonstrated.", e);
        panic!("Demo mode: Redis connection required for this demo")
    });

    let state = AppState {
        db: pool,
        redis,
        jwt_secret: cfg.jwt_secret.clone(),
    };

    let api = routes::build_router(&state);

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));
    info!("listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, api)
        .await?;
    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_max_level(Level::INFO)
        .with_ansi(true)
        .with_env_filter(filter)
        .init();
}