use std::{env, net::SocketAddr};

pub struct Config {
    pub bind: SocketAddr,
    pub database_url: String,
    pub admin_token: String,
    pub origin: String,
    pub secure_cookies: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let bind = env::var("BIND_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:3000".into())
            .parse::<SocketAddr>()
            .map_err(|_| "BIND_ADDR must be a socket address")?;
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL is required; there is no memory fallback")?;
        let admin_token =
            env::var("ADMIN_REVIEW_TOKEN").map_err(|_| "ADMIN_REVIEW_TOKEN is required")?;
        if admin_token.trim().len() < 32 || admin_token.contains("replace-with") {
            return Err(
                "ADMIN_REVIEW_TOKEN must be a randomly generated credential of at least 32 bytes"
                    .into(),
            );
        }
        let origin = env::var("APP_ORIGIN").map_err(|_| "APP_ORIGIN is required")?;
        let uri = origin
            .parse::<axum::http::Uri>()
            .map_err(|_| "APP_ORIGIN must be an origin URL")?;
        if uri.authority().is_none()
            || !matches!(uri.scheme_str(), Some("http" | "https"))
            || uri.path() != "/"
            || uri.query().is_some()
            || origin.ends_with('/')
        {
            return Err(
                "APP_ORIGIN must contain scheme and authority only, without trailing slash".into(),
            );
        }
        let secure_cookies = match env::var("COOKIE_SECURE").as_deref() {
            Ok("false") => false,
            Ok("true") | Err(_) => true,
            _ => return Err("COOKIE_SECURE must be true or false".into()),
        };
        if !secure_cookies
            && (!bind.ip().is_loopback()
                || !matches!(uri.host(), Some("localhost" | "127.0.0.1" | "[::1]")))
        {
            return Err("Insecure cookies are allowed only for loopback development".into());
        }
        if secure_cookies && uri.scheme_str() != Some("https") {
            return Err("Secure cookies require an HTTPS APP_ORIGIN".into());
        }
        Ok(Self {
            bind,
            database_url,
            admin_token,
            origin,
            secure_cookies,
        })
    }
}
