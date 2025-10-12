use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::{
    utils::jwt::validate_token,
    AppState,
};
use crate::db::redis::is_token_blacklisted;

#[derive(Clone, Copy)]
pub struct CurrentUser(pub uuid::Uuid);

pub async fn require_auth(State(state): State<AppState>, mut req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let auth = req.headers().get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !auth.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let token = auth.trim_start_matches("Bearer ").trim();

    let claims = match validate_token(token, &state.jwt_secret) {
        Ok(c) => c,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Check blacklist
    match is_token_blacklisted(&state.redis, &claims.jti).await {
        Ok(true) => return Err(StatusCode::UNAUTHORIZED),
        Ok(false) => {}
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }

    req.extensions_mut().insert(CurrentUser(claims.sub));
    Ok(next.run(req).await)
}