use crate::mangalib::Client;
use crate::mangalib::http_client::HttpClient;
use crate::processing::{self, Processor, commands};
use axum::extract::{OriginalUri, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};
use std::env;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{error, info};

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
    Server(#[from] std::io::Error),
    #[error("HTTP client build error {0}")]
    HttpClientBuild(#[from] reqwest::Error),
    #[error("Processor error {0}")]
    Processor(#[from] crate::processing::Error),
}

/// # Errors
/// - [`Error::Config`]: Error while parsing config
/// - [`Error::ServerError`]: Server error
pub async fn serve(
    port: u16,
    semaphore_permits: usize,
    proxy_str: Option<&str>,
) -> Result<(), Error> {
    let config = AppConfig::new(port, semaphore_permits);
    let mangalib_client = build_client(proxy_str)?;
    let state = Arc::new(AppState::new(
        config,
        Processor::new(mangalib_client, None)?,
    ));
    let address = state.config.address();
    let listener = TcpListener::bind(&address).await?;

    let router: Router = Router::new()
        .route("/async-command", post(do_async_command))
        .route("/async-command/", post(do_async_command))
        .route("/sync-command", post(do_sync_command))
        .route("/sync-command/", post(do_sync_command))
        .route("/health", get(health))
        .route("/health/", get(health))
        .route("/version", get(version))
        .route("/version/", get(version))
        .with_state(state)
        .fallback(handle_404);

    info!(address = address, "Web server is up");
    axum::serve(listener, router).await?;

    Ok(())
}

async fn do_async_command<TClient: Client + 'static>(
    State(state): State<Arc<AppState<TClient>>>,
    payload: String,
) -> (StatusCode, Json<Value>) {
    let command = match commands::parse_command(&payload) {
        Ok(command) => command,
        Err(err) => return parse_error_response(&err),
    };

    let processor = state.processor.clone();
    let semaphore_permits = state.config.semaphore_permits;

    tokio::spawn(async move {
        if let Err(err) = processor.process_command(command, semaphore_permits).await {
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

async fn do_sync_command<TClient: Client + 'static>(
    State(state): State<Arc<AppState<TClient>>>,
    payload: String,
) -> (StatusCode, Json<Value>) {
    let command = match commands::parse_command(&payload) {
        Ok(command) => command,
        Err(err) => return parse_error_response(&err),
    };

    let processor = state.processor.clone();
    let semaphore_permits = state.config.semaphore_permits;

    match processor.process_command(command, semaphore_permits).await {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "message": "Manga was sent successfully"
            })),
        ),
        Err(err) => {
            error!("Error while processing manga: {err:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "code": "PROCESSING_ERROR",
                    "message": format!("Failed to process command: {err:?}")
                })),
            )
        }
    }
}

fn build_client(proxy_str: Option<&str>) -> Result<HttpClient, Error> {
    let client_builder = match proxy_str {
        Some(proxy) => {
            let proxy = reqwest::Proxy::all(proxy).map_err(Error::HttpClientBuild)?;
            reqwest::ClientBuilder::new().proxy(proxy)
        }
        None => reqwest::ClientBuilder::new(),
    };

    let client = client_builder.build().map_err(Error::HttpClientBuild)?;

    Ok(HttpClient::builder().reqwest_client(client).build())
}

async fn handle_404(uri: OriginalUri) -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "success": false,
            "code": "NOT_FOUND",
            "message": format!("Route {} not found", uri.0)
        })),
    )
}

async fn health() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok"
        })),
    )
}

async fn version() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "version": env!("CARGO_PKG_VERSION")
        })),
    )
}

fn parse_error_response(err: &commands::ParseError) -> (StatusCode, Json<Value>) {
    match err {
        commands::ParseError::FirstParse(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "code": "INVALID_JSON",
                "message": format!("Failed to parse request body as JSON: {e}")
            })),
        ),
        commands::ParseError::PayloadMustBeObject => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "code": "PAYLOAD_MUST_BE_OBJECT",
                "message": "Request payload must be a JSON object"
            })),
        ),
        commands::ParseError::CommandMustBeString => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "code": "COMMAND_MUST_BE_STRING",
                "message": "Field 'command' must be a string"
            })),
        ),
        commands::ParseError::ParamsMustBeSet => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "code": "PARAMS_MUST_BE_SET",
                "message": "Field 'params' is required"
            })),
        ),
        commands::ParseError::InvalidParams(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "code": "INVALID_PARAMS",
                "message": format!("Invalid params: {e}")
            })),
        ),
        commands::ParseError::InvalidCommand(name) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "code": "COMMAND_NOT_FOUND",
                "message": format!("Unknown command: {name}")
            })),
        ),
    }
}
