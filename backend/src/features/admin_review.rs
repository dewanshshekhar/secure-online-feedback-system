use super::{
    admin_session::AdminAuth,
    feedback_store::{FeedbackRecord, FeedbackStore},
};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct ReviewState {
    store: FeedbackStore,
    auth: AdminAuth,
}

pub fn routes(store: FeedbackStore, auth: AdminAuth) -> Router {
    if !auth.enabled() {
        return Router::new();
    }
    Router::new()
        .route("/admin/feedback", get(list_feedback))
        .with_state(ReviewState { store, auth })
}

#[derive(Serialize)]
struct FeedbackPage {
    items: Vec<FeedbackRecord>,
    has_more: bool,
    next_offset: Option<usize>,
}

async fn list_feedback(
    State(state): State<ReviewState>,
    headers: HeaderMap,
    page: Result<Query<Pagination>, axum::extract::rejection::QueryRejection>,
) -> Result<Json<FeedbackPage>, StatusCode> {
    if !state.auth.authorized(&headers) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let Query(page) = page.map_err(|_| StatusCode::BAD_REQUEST)?;
    let limit = page.limit.unwrap_or(20);
    let offset = page.offset.unwrap_or_default();
    if !(1..=100).contains(&limit) || offset > 1_000_000 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let mut items = state
        .store
        .list_page(offset, limit + 1)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let has_more = items.len() > limit;
    items.truncate(limit);
    Ok(Json(FeedbackPage {
        items,
        has_more,
        next_offset: has_more.then_some(offset + limit),
    }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pagination {
    limit: Option<usize>,
    offset: Option<usize>,
}
