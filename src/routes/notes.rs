use crate::app_middleware::auth_middleware::CurrentUser;
use crate::{
    models::note::{CreateNoteRequest, UpdateNoteRequest},
    services::note_service,
    AppState,
};
use axum::{
    extract::{Extension, Path, Query, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::collections::HashMap;
use tracing::{error, info};
use uuid::Uuid;

// 使用通用查询映射，避免依赖派生宏导致的 IDE 诊断错误

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/notes", post(create).get(list))
        .route("/notes/:id", get(get_one).put(update).delete(remove))
}

async fn create(
    State(state): State<AppState>,
    Extension(CurrentUser(user_id)): Extension<CurrentUser>,
    Json(req): Json<CreateNoteRequest>,
) -> impl IntoResponse {
    info!(target = "http", route = "/notes#create", user_id = %user_id, title = %req.title, tags = ?req.tags, "incoming create note");
    match note_service::create_note(&state, user_id, req).await {
        Ok(note) => (axum::http::StatusCode::CREATED, Json(note)).into_response(),
        Err(e) => error_response(e),
    }
}

async fn list(
    State(state): State<AppState>,
    Extension(CurrentUser(user_id)): Extension<CurrentUser>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let tag = params.get("tag").cloned();
    let q = params.get("q").cloned();
    info!(target = "http", route = "/notes#list", user_id = %user_id, tag = ?tag, q = ?q, "incoming list notes");
    match note_service::list_notes(&state, user_id, tag, q).await {
        Ok(notes) => (axum::http::StatusCode::OK, Json(notes)).into_response(),
        Err(e) => error_response(e),
    }
}

async fn get_one(
    State(state): State<AppState>,
    Extension(CurrentUser(user_id)): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    info!(target = "http", route = "/notes#get", user_id = %user_id, id = %id, "incoming get note");
    match note_service::get_note(&state, user_id, id).await {
        Ok(note) => (axum::http::StatusCode::OK, Json(note)).into_response(),
        Err(e) => error_response(e),
    }
}

async fn update(
    State(state): State<AppState>,
    Extension(CurrentUser(user_id)): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateNoteRequest>,
) -> impl IntoResponse {
    info!(target = "http", route = "/notes#update", user_id = %user_id, id = %id, "incoming update note");
    match note_service::update_note(&state, user_id, id, req).await {
        Ok(note) => (axum::http::StatusCode::OK, Json(note)).into_response(),
        Err(e) => error_response(e),
    }
}

async fn remove(
    State(state): State<AppState>,
    Extension(CurrentUser(user_id)): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    info!(target = "http", route = "/notes#delete", user_id = %user_id, id = %id, "incoming delete note");
    match note_service::delete_note(&state, user_id, id).await {
        Ok(()) => (axum::http::StatusCode::NO_CONTENT, "").into_response(),
        Err(e) => error_response(e),
    }
}

fn error_response<E: std::fmt::Display>(e: E) -> axum::response::Response {
    error!(target = "http", error = %e, "notes route error");
    (
        axum::http::StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": e.to_string() })),
    )
        .into_response()
}
