use reqwest::Client;
use serde_json::Value;
use tracing::{error, info};

const MANGALIB_STATIC_RESOURCE: &str = include_str!("../resource/json/mangalib_manga_list.json");

pub async fn send_resource(url: &str) {
    let resource_vec: Vec<Value> = serde_json::from_str(MANGALIB_STATIC_RESOURCE).unwrap();

    for res in resource_vec.into_iter() {
        let sending_result = Client::new().post(url).json(&res).send().await;

        match sending_result {
            Ok(_) => {
                info!("Successfully sent")
            }
            Err(err) => {
                error!("Failed to send resource: {}", err)
            }
        }
    }
}
