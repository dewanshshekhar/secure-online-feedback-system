#![forbid(unsafe_code)]

pub mod config;
mod features;

use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderValue, Request},
    middleware::{self, Next},
    response::Response,
    Router,
};
use features::admin_session::AdminAuth;
use features::feedback_store::FeedbackStore;

pub const MAX_BODY_BYTES: usize = 16 * 1024;

#[cfg(test)]
pub fn app() -> Router {
    app_with_admin_token(None)
}

#[cfg(test)]
pub fn app_with_admin_token(admin_token: Option<String>) -> Router {
    let feedback_store = FeedbackStore::default();
    router(
        feedback_store,
        AdminAuth::new(admin_token, "http://127.0.0.1:8080".into(), false),
    )
}

pub async fn persistent_app(
    database_url: &str,
    config: &config::Config,
) -> Result<Router, sqlx::Error> {
    Ok(router(
        FeedbackStore::connect(database_url).await?,
        AdminAuth::new(
            Some(config.admin_token.clone()),
            config.origin.clone(),
            config.secure_cookies,
        ),
    ))
}

fn router(feedback_store: FeedbackStore, auth: AdminAuth) -> Router {
    Router::new()
        .nest(
            "/api",
            features::health::routes()
                .merge(features::feedback_api::routes(feedback_store.clone()))
                .merge(features::admin_review::routes(feedback_store, auth.clone()))
                .merge(features::admin_session::routes(auth)),
        )
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(middleware::from_fn(security_headers))
}

async fn security_headers(request: Request<axum::body::Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        header::HeaderName::from_static("x-request-id"),
        HeaderValue::from_str(&uuid::Uuid::new_v4().to_string()).expect("UUID is a valid header"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'; base-uri 'none'"),
    );
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
        routing::post,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_returns_json_liveness() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["content-type"], "application/json");
        assert_eq!(response.headers()["x-content-type-options"], "nosniff");
        assert_eq!(response.headers()["x-frame-options"], "DENY");
        assert_eq!(response.headers()["cache-control"], "no-store");
        assert!(response.headers()["x-request-id"].to_str().unwrap().len() == 36);
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
            serde_json::json!({"status": "ok"})
        );
    }

    #[tokio::test]
    async fn unknown_routes_return_not_found() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/api/not-a-route")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn feedback_submission_returns_an_opaque_receipt() {
        let response = app()
            .oneshot(
                Request::post("/api/feedback")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"category":"teaching","message":"Helpful examples"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        let receipt: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(receipt["id"].as_str().is_some());
    }

    #[tokio::test]
    async fn feedback_submission_rejects_empty_messages() {
        let response = app()
            .oneshot(
                Request::post("/api/feedback")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"category":"other","message":" "}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
            serde_json::json!({"error":"validation_failed","message":"message must not be empty"})
        );
    }

    #[tokio::test]
    async fn feedback_submission_rejects_non_json_content_types() {
        let response = app()
            .oneshot(
                Request::post("/api/feedback")
                    .header("content-type", "text/plain")
                    .body(Body::from("feedback"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
            serde_json::json!({"error":"unsupported_media_type","message":"content-type must be application/json"})
        );
    }

    #[tokio::test]
    async fn feedback_submission_rejects_malformed_json_without_echoing_body() {
        let response = app()
            .oneshot(
                Request::post("/api/feedback")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"message":"unterminated"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
            serde_json::json!({"error":"invalid_json","message":"request body must be valid JSON"})
        );
    }

    #[tokio::test]
    async fn feedback_submission_rejects_unexpected_fields() {
        let response = app()
            .oneshot(
                Request::post("/api/feedback")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"category":"other","message":"Useful","email":"student@example.com"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn submission_enforces_actual_body_limit() {
        let response = app().oneshot(Request::post("/api/feedback").header("content-type", "application/json")
            .body(Body::from(serde_json::json!({"category":"other", "message":"a".repeat(MAX_BODY_BYTES)}).to_string())).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn review_requires_session_and_returns_only_saved_records() {
        let router = app_with_admin_token(Some("test-only-administrator-credential-123456".into()));
        let login = router
            .clone()
            .oneshot(
                Request::post("/api/admin/session")
                    .header("content-type", "application/json")
                    .header("origin", "http://127.0.0.1:8080")
                    .body(Body::from(
                        r#"{"credential":"test-only-administrator-credential-123456"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let cookie = login.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned();
        let empty = router
            .clone()
            .oneshot(
                Request::get("/api/admin/feedback")
                    .header("cookie", &cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let value: serde_json::Value =
            serde_json::from_slice(&to_bytes(empty.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(value["items"], serde_json::json!([]));
        for text in [
            "First submitted message",
            "<script>alert('text only')</script>",
        ] {
            let response = router
                .clone()
                .oneshot(
                    Request::post("/api/feedback")
                        .header("content-type", "application/json")
                        .body(Body::from(
                            serde_json::json!({"category":"other", "message":text}).to_string(),
                        ))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::CREATED);
        }
        let response = router
            .clone()
            .oneshot(
                Request::get("/api/admin/feedback?limit=1")
                    .header("cookie", &cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let value: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(
            value["items"][0]["message"],
            "<script>alert('text only')</script>"
        );
        assert_eq!(value["has_more"], true);
        assert_eq!(value["next_offset"], 1);
        for query in ["limit=0", "limit=101", "offset=1000001", "limit=bad"] {
            let response = router
                .clone()
                .oneshot(
                    Request::get(format!("/api/admin/feedback?{query}"))
                        .header("cookie", &cookie)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }
        let legacy = router
            .oneshot(
                Request::get("/api/admin/feedback")
                    .header("x-admin-token", "test-only-administrator-credential-123456")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(legacy.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn admin_review_requires_a_configured_token() {
        let response = app_with_admin_token(Some("review-token-that-is-at-least-32-bytes".into()))
            .oneshot(
                Request::get("/api/admin/feedback")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn admin_review_route_is_disabled_without_a_token() {
        let response = app()
            .oneshot(
                Request::get("/api/admin/feedback")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn buffered_request_bodies_are_limited() {
        // Exercise the buffered-extractor boundary independently of validation.
        let router = Router::new()
            .route("/", post(|_: String| async { StatusCode::NO_CONTENT }))
            .layer(DefaultBodyLimit::max(MAX_BODY_BYTES));
        for (size, expected) in [
            (MAX_BODY_BYTES, StatusCode::NO_CONTENT),
            (MAX_BODY_BYTES + 1, StatusCode::PAYLOAD_TOO_LARGE),
        ] {
            let response = router
                .clone()
                .oneshot(
                    Request::post("/")
                        .body(Body::from("a".repeat(size)))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), expected);
        }
    }
}
