use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use subtle::ConstantTimeEq;

const TTL: Duration = Duration::from_secs(1800);

#[derive(Clone)]
pub struct AdminAuth {
    token: Option<String>,
    origin: String,
    secure: bool,
    state: Arc<Mutex<AuthState>>,
}

#[derive(Default)]
struct AuthState {
    sessions: HashMap<String, Instant>,
    attempts: VecDeque<Instant>,
}

impl AdminAuth {
    pub fn new(token: Option<String>, origin: String, secure: bool) -> Self {
        Self {
            token: token.filter(|value| value.trim().len() >= 32),
            origin,
            secure,
            state: Default::default(),
        }
    }

    pub fn enabled(&self) -> bool {
        self.token.is_some()
    }
    fn cookie_name(&self) -> &'static str {
        if self.secure {
            "__Host-feedback_session"
        } else {
            "feedback_session"
        }
    }
    fn session_id<'a>(&self, headers: &'a HeaderMap) -> Option<&'a str> {
        headers
            .get(header::COOKIE)?
            .to_str()
            .ok()?
            .split(';')
            .find_map(|cookie| {
                let (name, value) = cookie.trim().split_once('=')?;
                (name == self.cookie_name()).then_some(value)
            })
    }
    pub fn authorized(&self, headers: &HeaderMap) -> bool {
        let Some(id) = self.session_id(headers) else {
            return false;
        };
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        state.sessions.retain(|_, issued| issued.elapsed() < TTL);
        state.sessions.contains_key(id)
    }
    fn check_origin(&self, headers: &HeaderMap) -> Result<(), StatusCode> {
        if headers
            .get(header::ORIGIN)
            .and_then(|value| value.to_str().ok())
            != Some(&self.origin)
        {
            return Err(StatusCode::FORBIDDEN);
        }
        Ok(())
    }
    fn cookie(&self, value: &str, age: u32) -> String {
        format!(
            "{}={value}; Path=/; HttpOnly; SameSite=Strict; Max-Age={age}{}",
            self.cookie_name(),
            if self.secure { "; Secure" } else { "" }
        )
    }
}

pub fn routes(auth: AdminAuth) -> Router {
    Router::new()
        .route("/admin/session", post(login).get(session).delete(logout))
        .with_state(auth)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Login {
    credential: String,
}

async fn login(
    State(auth): State<AdminAuth>,
    headers: HeaderMap,
    Json(login): Json<Login>,
) -> Result<(StatusCode, [(header::HeaderName, String); 1]), StatusCode> {
    auth.check_origin(&headers)?;
    let Some(expected) = &auth.token else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let mut state = auth
        .state
        .lock()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    state
        .attempts
        .retain(|attempt| attempt.elapsed() < Duration::from_secs(60));
    if state.attempts.len() >= 20 {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    state.attempts.push_back(Instant::now());
    if !constant_time_equal(login.credential.as_bytes(), expected.as_bytes()) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    state.sessions.retain(|_, issued| issued.elapsed() < TTL);
    if state.sessions.len() >= 128 {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    if let Some(previous) = auth.session_id(&headers) {
        state.sessions.remove(previous);
    }
    let id = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    state.sessions.insert(id.clone(), Instant::now());
    Ok((
        StatusCode::NO_CONTENT,
        [(header::SET_COOKIE, auth.cookie(&id, TTL.as_secs() as u32))],
    ))
}

async fn session(State(auth): State<AdminAuth>, headers: HeaderMap) -> StatusCode {
    if auth.authorized(&headers) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::UNAUTHORIZED
    }
}

async fn logout(
    State(auth): State<AdminAuth>,
    headers: HeaderMap,
) -> Result<(StatusCode, [(header::HeaderName, String); 1]), StatusCode> {
    auth.check_origin(&headers)?;
    if let Some(id) = auth.session_id(&headers) {
        auth.state
            .lock()
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
            .sessions
            .remove(id);
    }
    Ok((
        StatusCode::NO_CONTENT,
        [(header::SET_COOKIE, auth.cookie("", 0))],
    ))
}

fn constant_time_equal(left: &[u8], right: &[u8]) -> bool {
    // Compare fixed-size digests with a reviewed primitive instead of a hand-written
    // byte loop that a compiler could turn into a short-circuit comparison.
    Sha256::digest(left).ct_eq(&Sha256::digest(right)).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    const TOKEN: &str = "test-only-administrator-credential-123456";
    fn auth() -> AdminAuth {
        AdminAuth::new(Some(TOKEN.into()), "https://feedback.test".into(), true)
    }
    fn request(origin: &str, credential: &str) -> Request<Body> {
        Request::post("/admin/session")
            .header("origin", origin)
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"credential": credential}).to_string(),
            ))
            .unwrap()
    }

    #[tokio::test]
    async fn login_sets_protected_cookie_and_logout_revokes_it() {
        let auth = auth();
        let router = routes(auth.clone());
        let response = router
            .clone()
            .oneshot(request("https://feedback.test", TOKEN))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let cookie = response.headers()[header::SET_COOKIE].to_str().unwrap();
        for flag in [
            "HttpOnly",
            "Secure",
            "SameSite=Strict",
            "Max-Age=1800",
            "__Host-feedback_session=",
        ] {
            assert!(cookie.contains(flag));
        }
        let cookie = cookie.split(';').next().unwrap().to_owned();
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, cookie.parse().unwrap());
        assert!(auth.authorized(&headers));
        let response = router
            .oneshot(
                Request::delete("/admin/session")
                    .header("origin", "https://feedback.test")
                    .header("cookie", cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert!(!auth.authorized(&headers));
    }

    #[tokio::test]
    async fn login_rejects_foreign_origins_and_wrong_credentials() {
        let router = routes(auth());
        assert_eq!(
            router
                .clone()
                .oneshot(request("https://attacker.test", TOKEN))
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            router
                .clone()
                .oneshot(request("https://feedback.test", "wrong"))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            router
                .oneshot(request("https://feedback.test", &format!("{TOKEN}extra")))
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn login_attempts_are_bounded() {
        let router = routes(auth());
        for _ in 0..20 {
            router
                .clone()
                .oneshot(request("https://feedback.test", "wrong"))
                .await
                .unwrap();
        }
        assert_eq!(
            router
                .oneshot(request("https://feedback.test", TOKEN))
                .await
                .unwrap()
                .status(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[test]
    fn expired_sessions_are_rejected() {
        let auth = auth();
        auth.state
            .lock()
            .unwrap()
            .sessions
            .insert("expired".into(), Instant::now() - TTL);
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            "__Host-feedback_session=expired".parse().unwrap(),
        );
        assert!(!auth.authorized(&headers));
    }
}
