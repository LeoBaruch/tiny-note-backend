use axum::{
    response::IntoResponse,
    routing::get,
    Json, Router,
    http::{StatusCode, Method},
};
use serde_json::json;
use std::net::SocketAddr;
use tracing::{info, Level};
use tower_http::cors::{CorsLayer, AllowOrigin, AllowMethods, AllowHeaders};
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({
        "status": "ok",
        "message": "Tiny Note Backend is running",
        "version": "1.0.0",
        "endpoints": {
            "health": "GET /health",
            "auth": {
                "register": "POST /auth/register",
                "login": "POST /auth/login",
                "logout": "POST /auth/logout"
            },
            "notes": {
                "list": "GET /notes",
                "create": "POST /notes",
                "get": "GET /notes/:id",
                "update": "PUT /notes/:id",
                "delete": "DELETE /notes/:id"
            }
        }
    })))
}

async fn api_info() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({
        "name": "Tiny Note Backend",
        "description": "A simple note-taking API built with Rust and Axum",
        "tech_stack": {
            "framework": "Axum",
            "database": "MySQL",
            "cache": "Redis",
            "auth": "JWT"
        },
        "features": [
            "User registration and authentication",
            "JWT-based authorization",
            "CRUD operations for notes",
            "Redis-based token blacklisting",
            "Password hashing with Argon2"
        ]
    })))
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

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async_main()).unwrap();
}

async fn async_main() -> anyhow::Result<()> {
    // 加载环境变量，支持通过 PORT 配置端口
    dotenvy::dotenv().ok();
    init_tracing();
    
    // CORS：允许任意域名，并支持携带 Cookie（通过镜像请求的 Origin）
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::mirror_request())
        .allow_methods(AllowMethods::list([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS]))
        .allow_headers(AllowHeaders::list([AUTHORIZATION, CONTENT_TYPE]))
        .allow_credentials(true);

    let app = Router::new()
        .route("/", get(api_info))
        .route("/health", get(health_check))
        .layer(cors);

    // 从环境变量读取端口，默认 8080
    let port: u16 = std::env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 Tiny Note Backend Demo Server starting on {}", addr);
    info!("📋 Available endpoints:");
    info!("   GET  /        - API information");
    info!("   GET  /health  - Health check");
    
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app)
        .await?;
    Ok(())
}