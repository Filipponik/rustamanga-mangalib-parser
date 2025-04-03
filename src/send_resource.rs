use crate::mangalib::MangaPreview;
use reqwest::Client;
use thiserror::Error;
use tracing::{error, info};

const MANGALIB_STATIC_RESOURCE: &str = include_str!("../resource/json/mangalib_manga_list.json");

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error while parsing json: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Error sending resource: {0}")]
    Send(#[from] reqwest::Error),
}

pub async fn send_resource(url: &str) -> Result<(), Error> {
    let mangas: Vec<MangaPreview> =
        serde_json::from_str(MANGALIB_STATIC_RESOURCE).map_err(Error::Parse)?;

    let client = Client::new();
    for manga in mangas {
        match send_single_resource(client.clone(), url, manga).await {
            Ok(()) => info!("Successfully sent"),
            Err(err) => error!("Failed to send resource: {}", err),
        }
    }

    Ok(())
}

async fn send_single_resource(client: Client, url: &str, manga: MangaPreview) -> Result<(), Error> {
    client.post(url).json(&manga).send().await?;

    Ok(())
}
