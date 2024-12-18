use reqwest::Client;
use serde_json::Value;
use tracing::{error, info};

const MANGALIB_STATIC_RESOURCE: &str = include_str!("../resource/json/mangalib_manga_list.json");

#[derive(Debug)]
pub enum Error {
    Parse(serde_json::Error),
}

pub async fn send_resource(url: &str) -> Result<(), Error> {
    let resource_vec: Vec<Value> =
        serde_json::from_str(MANGALIB_STATIC_RESOURCE).map_err(Error::Parse)?;

    let mut handlers = Vec::new();
    for res in resource_vec {
        let res_cloned = res.clone();
        let url_cloned = url.to_string();
        let handler = tokio::spawn(async move {
            let sending_result = Client::new()
                .post(&url_cloned)
                .json(&res_cloned)
                .send()
                .await;

            match sending_result {
                Ok(_) => {
                    info!("Successfully sent");
                }
                Err(err) => {
                    error!("Failed to send resource: {}", err);
                }
            }
        });

        handlers.push(handler);
    }

    futures::future::join_all(handlers).await;

    Ok(())
}
