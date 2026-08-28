use crate::mangalib::Client;
use crate::mangalib::http_client::HttpClient;
use crate::openapi::{
    CommandAcceptedResponse, CommandRequest, ErrorResponse, HealthResponse, VersionResponse,
};
use crate::processing::{self, Processor, commands};
use axum::extract::{OriginalUri, State};
use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};
use std::env;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{error, info};
use utoipa::OpenApi;

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
        .route("/docs", get(docs))
        .route("/docs/", get(docs))
        .route("/api-docs/openapi.json", get(openapi_json))
        .with_state(state)
        .fallback(handle_404);

    info!(address = address, "Web server is up");
    axum::serve(listener, router).await?;

    Ok(())
}

/// Process command asynchronously
///
/// Accepts a scraping command payload, validates it and schedules processing
/// in a background task. Responds immediately with `200 OK` when the payload
/// is valid, or with `400`/`404` when validation fails.
#[utoipa::path(
    post,
    path = "/async-command",
    tag = "Command",
    request_body(
        content = CommandRequest,
        description = "Command payload to process"
    ),
    responses(
        (status = 200, description = "Command accepted for processing", body = CommandAcceptedResponse),
        (status = 400, description = "Invalid payload", body = ErrorResponse),
        (status = 404, description = "Unknown command", body = ErrorResponse)
    )
)]
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

/// Process command synchronously
///
/// Accepts a scraping command payload and processes it synchronously: the
/// response is returned only after the command is fully processed. Returns
/// `200 OK` on success, `400`/`404` for validation errors and `500` when
/// processing fails.
#[utoipa::path(
    post,
    path = "/sync-command",
    tag = "Command",
    request_body(
        content = CommandRequest,
        description = "Command payload to process"
    ),
    responses(
        (status = 200, description = "Command processed successfully", body = CommandAcceptedResponse),
        (status = 400, description = "Invalid payload", body = ErrorResponse),
        (status = 404, description = "Unknown command", body = ErrorResponse),
        (status = 500, description = "Command processing failed", body = ErrorResponse)
    )
)]
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

/// Health check
///
/// Reports server liveness. Handy for Docker healthchecks and Kubernetes
/// liveness probes.
#[utoipa::path(
    get,
    path = "/health",
    tag = "AppState",
    responses(
        (status = 200, description = "Server is healthy", body = HealthResponse)
    )
)]
async fn health() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok"
        })),
    )
}

/// Get server version
///
/// Returns the current server version from `Cargo.toml`.
#[utoipa::path(
    get,
    path = "/version",
    tag = "AppState",
    responses(
        (status = 200, description = "Server version", body = VersionResponse)
    )
)]
async fn version() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "version": env!("CARGO_PKG_VERSION")
        })),
    )
}

/// `OpenAPI` documentation of the HTTP API served by the `serve` command.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Rustamanga Mangalib parser",
        version = env!("CARGO_PKG_VERSION"),
        description = "Mangalib scraping service: accepts scraping commands over HTTP and \
                       publishes results to the provided callback URL",
        license(name = "MIT")
    ),
    paths(
        health,
        version,
        do_async_command,
        do_sync_command,
    ),
    tags(
        (name = "Command", description = "Scraping commands: submit manga and user list jobs"),
        (name = "AppState", description = "Server liveness and version info"),
    ),
    components(schemas(
        CommandRequest,
        CommandAcceptedResponse,
        ErrorResponse,
        HealthResponse,
        VersionResponse,
    ))
)]
struct ApiDoc;

/// Serves the interactive API documentation page powered by Scalar.
async fn docs() -> Html<&'static str> {
    Html(DOCS_HTML)
}

/// Serves the raw `OpenAPI` specification consumed by the docs page.
async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

const DOCS_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Rustamanga Mangalib parser — API Reference</title>
    <style>body { margin: 0; padding: 0; }</style>
</head>
<body>
    <script id="api-reference" data-url="/api-docs/openapi.json"></script>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
</body>
</html>
"#;

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
