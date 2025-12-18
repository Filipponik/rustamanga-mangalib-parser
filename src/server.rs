use crate::mangalib::Client;
use crate::mangalib::http_client::HttpClient;
use crate::processing::{self, Processor};
use axum::extract::{OriginalUri, State};
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{Value, json};
use std::env;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{error, info};

const SCRAP_MANGA_ROUTE: &str = "/scrap-manga";

#[derive(Clone)]
struct AppState<TClient: Client> {
    config: AppConfig,
    processor: processing::Processor<TClient>,
}

impl<TClient: Client> AppState<TClient> {
    pub const fn new(config: AppConfig, processor: processing::Processor<TClient>) -> Self {
        Self { config, processor }
    }
}

#[derive(Clone)]
struct AppConfig {
    port: u16,
    semaphore_permits: usize,
}

impl AppConfig {
    #[allow(dead_code)]
    pub fn from_env() -> Result<Self, ConfigErrorType> {
        let port = env::var("APP_PORT")?.parse::<u16>()?;
        let semaphore_permits = env::var("SEMAPHORE_PERMITS")?.parse::<usize>()?;

        Ok(Self::new(port, semaphore_permits))
    }

    pub const fn new(port: u16, semaphore_permits: usize) -> Self {
        Self {
            port,
            semaphore_permits,
        }
    }

    pub fn address(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }
}

#[derive(Debug, Error)]
pub enum ConfigErrorType {
    #[error("Error while parsing environment variable {0}")]
    ParseEnv(#[from] env::VarError),
    #[error("Error while parsing int variable {0}")]
    ParseInt(#[from] std::num::ParseIntError),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error while parsing config {0}")]
    Config(#[from] ConfigErrorType),
    #[error("Server error {0}")]
    ServerError(#[from] std::io::Error),
}

/// # Errors
/// - [`Error::Config`]: Error while parsing config
/// - [`Error::ServerError`]: Server error
pub async fn serve(port: u16, semaphore_permits: usize) -> Result<(), Error> {
    let config = AppConfig::new(port, semaphore_permits);
    let state = Arc::new(AppState::new(
        config,
        Processor::new(HttpClient::builder().build(), None),
    ));
    let address = state.config.address();
    let listener = TcpListener::bind(&address).await?;

    let router: Router = Router::new()
        .route(SCRAP_MANGA_ROUTE, post(scrap_manga))
        .route(&format!("{SCRAP_MANGA_ROUTE}/"), post(scrap_manga))
        .with_state(state)
        .fallback(handle_404);

    info!(address = address, "Web server is up");
    axum::serve(listener, router).await?;

    Ok(())
}

async fn scrap_manga<TClient: Client + 'static>(
    State(state): State<Arc<AppState<TClient>>>,
    Json(payload): Json<ScrapMangaRequest>,
) -> (StatusCode, Json<Value>) {
    let processor = state.processor.clone();
    let semaphore_permits = state.config.semaphore_permits;

    tokio::spawn(async move {
        if let Err(err) = processor.process(semaphore_permits, payload).await {
            error!("Error while processing manga: {err:?}");
        }
    });

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "message": "Manga was sent successfully"
        })),
    )
}

async fn handle_404(uri: OriginalUri) -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "success": false,
            "message": format!("Route {} not found", uri.0)
        })),
    )
}
