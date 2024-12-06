use reqwest::{Client};
use serde_json::Value;

const MANGALIB_STATIC_RESOURCE: &str = include_str!("../resource/json/mangalib_manga_list.json");

pub async fn send_resource(url: &str) {
    let resource_vec: Vec<Value> = serde_json::from_str(MANGALIB_STATIC_RESOURCE).unwrap();

    for res in resource_vec.into_iter() {
        let sending_result = Client::new()
            .post(url)
            .body(res.to_string())
            .send()
            .await;

        match sending_result {
            Ok(_) => {
                println!("Successfully sent")
            }
            Err(err) => {
                eprintln!("Failed to send resource: {}", err)
            }
        }
    }
}

