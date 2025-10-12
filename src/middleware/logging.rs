use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
};
use tracing::info;

pub async fn request_logger(req: Request<Body>, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let query = uri.query().unwrap_or("");
    let content_type = req
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    info!(target: "http", "request: method={} path={} query={} ct={}", method, uri.path(), query, content_type);
    let res = next.run(req).await;
    let status = res.status();
    info!(target: "http", "response: status={} reason={}", status.as_u16(), status.canonical_reason().unwrap_or(""));
    res
}