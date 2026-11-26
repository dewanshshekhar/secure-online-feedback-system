use axum::{routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub fn routes() -> Router {
    Router::new().route(
        "/health",
        get(|| async { Json(HealthResponse { status: "ok" }) }),
    )
}
