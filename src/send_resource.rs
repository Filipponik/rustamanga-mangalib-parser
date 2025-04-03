use reqwest::Client;
use thiserror::Error;
use tokio::task::JoinHandle;
use tracing::{error, info};
use crate::mangalib::MangaPreview;

const MANGALIB_STATIC_RESOURCE: &str = include_str!("../resource/json/mangalib_manga_list.json");

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error while parsing json: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Error sending resource: {0}")]
    Send(#[from] reqwest::Error),
}

pub async fn send_resource(url: &str) -> Result<(), Error> {
    let resource_vec: Vec<MangaPreview> =
        serde_json::from_str(MANGALIB_STATIC_RESOURCE).map_err(Error::Parse)?;

    let client = Client::new();
    let handlers = resource_vec
        .into_iter()
        .map(|res| process_single_resource(url.to_string(), res, client.clone()));

    futures::future::join_all(handlers).await;

    Ok(())
}

fn process_single_resource(url: String, res: MangaPreview, client: Client) -> JoinHandle<()> {
    tokio::spawn(async move {
        match send_single_resource(client, &url, res).await {
            Ok(()) => info!("Successfully sent"),
            Err(err) => error!("Failed to send resource: {}", err),
        };
    })
}

async fn send_single_resource(client: Client, url: &str, res: MangaPreview) -> Result<(), Error> {
    client.post(url).json(&res).send().await?;

    Ok(())
}
