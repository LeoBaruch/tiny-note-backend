use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};
use serde_json::json;
use crate::{models::user::{RegisterRequest, LoginRequest}, services::auth_service, AppState};
use tracing::{error, info};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
}

async fn register(State(state): State<AppState>, Json(req): Json<RegisterRequest>) -> impl IntoResponse {
    info!(target = "http", route = "/auth/register", username = %req.username, email = %req.email, "incoming register request");
    match auth_service::register(&state, req).await {
        Ok(user) => (axum::http::StatusCode::CREATED, Json(user)).into_response(),
        Err(e) => error_response(e),
    }
}

async fn login(State(state): State<AppState>, Json(req): Json<LoginRequest>) -> impl IntoResponse {
    info!(target = "http", route = "/auth/login", email = %req.email, "incoming login request");
    match auth_service::login(&state, req).await {
        Ok(resp) => (axum::http::StatusCode::OK, Json(resp)).into_response(),
        Err(e) => error_response(e),
    }
}

fn error_response<E: std::fmt::Display>(e: E) -> axum::response::Response {
    error!(target = "http", error = %e, "auth route error");
    (
        axum::http::StatusCode::BAD_REQUEST,
        Json(json!({ "error": e.to_string() })),
    ).into_response()
}