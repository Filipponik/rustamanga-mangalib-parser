use crate::processing;
use crate::processing::ScrapMangaRequest;
use axum::extract::{OriginalUri, State};
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use std::env;
use std::sync::Arc;
use futures::TryFutureExt;
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{error, info};

const SCRAP_MANGA_ROUTE: &str = "/scrap-manga";

#[derive(Clone)]
struct AppState {
    config: AppConfig,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
}

#[derive(Clone)]
struct AppConfig {
    port: u16,
    chrome_max_count: u16,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigErrorType> {
        let port = env::var("APP_PORT")?
            .parse::<u16>()?;
        let chrome_max_count = env::var("CHROME_MAX_COUNT")?
            .parse::<u16>()?;

        Ok(Self { port, chrome_max_count })
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

pub async fn serve() -> Result<(), Error> {
    let config = AppConfig::from_env()?;
    let state = Arc::new(AppState::new(config));
    let address = state.config.address();
    let listener = TcpListener::bind(&address).await?;

    let router: Router = Router::new()
        .route(SCRAP_MANGA_ROUTE, post(scrap_manga))
        .route(&format!("{}/", SCRAP_MANGA_ROUTE), post(scrap_manga))
        .with_state(state)
        .fallback(handle_404);

    info!("Web server is up: {}", address);
    axum::serve(listener, router).await?;

    Ok(())
}

async fn scrap_manga(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ScrapMangaRequest>,
) -> (StatusCode, Json<Value>) {
    tokio::spawn(async move {
        if let Err(err) = processing::process(state.config.chrome_max_count, payload).await {
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
