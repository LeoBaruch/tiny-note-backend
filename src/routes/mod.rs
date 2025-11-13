use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::{
    http::{Method, StatusCode},
    response::IntoResponse,
    Json, Router,
};
use serde_json::json;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};
use tower_http::services::ServeDir;

use crate::AppState;

pub mod auth;
pub mod notes;

async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "message": "Tiny Note Backend is running"
        })),
    )
}

pub fn build_router(state: &AppState) -> Router {
    let auth_routes = auth::router();
    let notes_routes = notes::router().route_layer(axum::middleware::from_fn_with_state(
        state.clone(),
        crate::app_middleware::auth_middleware::require_auth,
    ));

    // CORS：允许任意域名，并支持携带 Cookie（通过镜像请求的 Origin）
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::mirror_request())
        .allow_methods(AllowMethods::list([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ]))
        .allow_headers(AllowHeaders::list([AUTHORIZATION, CONTENT_TYPE]))
        .allow_credentials(true);

    let api = Router::new()
        .route("/health", axum::routing::get(health_check))
        .merge(auth_routes)
        .merge(notes_routes)
        .nest_service("/static", ServeDir::new("static"));

    Router::new()
        .nest("/api/tiny-note", api)
        .layer(axum::middleware::from_fn(
            crate::middleware::logging::request_logger,
        ))
        .layer(cors)
        .with_state(state.clone())
}
