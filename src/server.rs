use crate::processing;
use crate::processing::ScrapMangaRequest;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use std::env;
use tokio::net::TcpListener;
use tracing::info;

#[derive(Clone)]
struct AppState {
    port: u16,
    chrome_max_count: u16,
}

#[derive(Debug)]
pub enum ConfigErrorType {
    ParseEnv(env::VarError),
    ParseInt(std::num::ParseIntError),
}

#[derive(Debug)]
pub enum Error {
    Config(ConfigErrorType),
    ServerError(std::io::Error),
}

pub async fn serve() -> Result<(), Error> {
    let port = env::var("APP_PORT")
        .map_err(|err| Error::Config(ConfigErrorType::ParseEnv(err)))?
        .parse::<u16>()
        .map_err(|err| Error::Config(ConfigErrorType::ParseInt(err)))?;
    let chrome_max_count = env::var("CHROME_MAX_COUNT")
        .map_err(|err| Error::Config(ConfigErrorType::ParseEnv(err)))?
        .parse::<u16>()
        .map_err(|err| Error::Config(ConfigErrorType::ParseInt(err)))?;

    let state = AppState {
        port,
        chrome_max_count,
    };
    let address = &format!("0.0.0.0:{}", state.port.clone());
    let listener = TcpListener::bind(address)
        .await
        .map_err(Error::ServerError)?;
    let router: Router = Router::new()
        .route("/scrap-manga", post(scrap_manga))
        .route("/scrap-manga/", post(scrap_manga))
        .with_state(state)
        .fallback(handle_404);

    info!("Web server is up: {address}");
    axum::serve(listener, router)
        .await
        .map_err(Error::ServerError)?;

    Ok(())
}

async fn scrap_manga(
    State(state): State<AppState>,
    Json(payload): Json<ScrapMangaRequest>,
) -> (StatusCode, Json<Value>) {
    tokio::spawn(async move {
        let _ignored_result = processing::process(state.chrome_max_count, payload).await;
    });

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "message": "Manga was sent successfully"
        })),
    )
}

async fn handle_404() -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "success": false,
            "message": "Route not found"
        })),
    )
}
