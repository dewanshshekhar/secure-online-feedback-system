use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use feedback_backend::{config::Config, persistent_app};
use tower::ServiceExt;

/// Run only against a dedicated disposable database whose name contains feedback_test.
#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing at a disposable feedback_test database"]
async fn submitted_feedback_survives_application_recreation() {
    let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL required");
    assert!(
        url.split('?').next().unwrap().ends_with("/feedback_test"),
        "use a dedicated feedback_test database"
    );
    let pool = sqlx::PgPool::connect(&url).await.unwrap();
    sqlx::raw_sql(include_str!("../migrations/0001_feedback.sql"))
        .execute(&pool)
        .await
        .unwrap();
    let config = Config {
        bind: "127.0.0.1:3000".parse().unwrap(),
        database_url: url.clone(),
        admin_token: "test-only-administrator-credential-123456".into(),
        origin: "http://127.0.0.1:8080".into(),
        secure_cookies: false,
    };
    let router = persistent_app(&url, &config).await.unwrap();
    let response = router
        .oneshot(
            Request::post("/api/feedback")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"category":"facilities","message":"Isolated persistence verification"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let receipt: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    let id: uuid::Uuid = receipt["id"].as_str().unwrap().parse().unwrap();
    let restarted = persistent_app(&url, &config).await.unwrap();
    let login = restarted
        .clone()
        .oneshot(
            Request::post("/api/admin/session")
                .header("origin", &config.origin)
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"credential":config.admin_token}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::NO_CONTENT);
    let cookie = login.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    let response = restarted
        .oneshot(
            Request::get("/api/admin/feedback")
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let page: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 16384).await.unwrap()).unwrap();
    assert!(page["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|record| record["id"] == id.to_string()
            && record["message"] == "Isolated persistence verification"));
    sqlx::query("DELETE FROM feedback WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
}
