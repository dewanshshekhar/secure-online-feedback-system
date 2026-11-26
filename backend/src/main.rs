#![forbid(unsafe_code)]

use std::error::Error;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let config = feedback_backend::config::Config::from_env().map_err(std::io::Error::other)?;
    let app = feedback_backend::persistent_app(&config.database_url, &config)
        .await
        .map_err(|_| {
            std::io::Error::other("Cannot connect to feedback database or schema is missing")
        })?;
    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    tracing::info!(address = %listener.local_addr()?, "API listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            if let Err(error) = tokio::signal::ctrl_c().await {
                tracing::error!(%error, "Failed to listen for shutdown signal");
            }
        })
        .await?;
    Ok(())
}
