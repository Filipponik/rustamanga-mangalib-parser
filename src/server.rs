use crate::processing::ScrapMangaRequest;
use crate::{mangalib, processing};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tracing::{error, info};

#[derive(Clone)]
struct AppState {
    port: u16,
    chrome_max_count: u16,
}

pub async fn serve() {
    let state = AppState {
        port: env::var("APP_PORT").unwrap().parse::<u16>().unwrap(),
        chrome_max_count: env::var("CHROME_MAX_COUNT")
            .unwrap()
            .parse::<u16>()
            .unwrap(),
    };
    let address = &format!("0.0.0.0:{}", state.port.clone());
    let listener = TcpListener::bind(address).await.unwrap();
    let router: Router = Router::new()
        .route("/scrap-manga", post(scrap_manga))
        .route("/scrap-manga/", post(scrap_manga))
        .with_state(state)
        .fallback(handle_404);

    info!("Web server is up: {address}");
    axum::serve(listener, router).await.unwrap();
}

async fn scrap_manga(
    State(state): State<AppState>,
    Json(payload): Json<ScrapMangaRequest>,
) -> (StatusCode, Json<Value>) {
    tokio::spawn(async move { processing::process(state.chrome_max_count, payload) });

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
