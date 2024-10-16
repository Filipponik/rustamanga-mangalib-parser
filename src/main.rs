use mangalib::MangaChapter;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::routing::{post};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use telegraph::types::NodeElement;
use tokio::sync::Semaphore;

mod mangalib;
mod telegraph;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:8081").await.unwrap();
    let router: Router = Router::new()
        .route("/scrap_manga", post(scrap_manga))
        .route("/scrap_manga/", post(scrap_manga))
        .fallback(handle_404);

    axum::serve(listener, router).await.unwrap();
}

#[derive(Deserialize)]
struct ScrapMangaRequest {
    slug: String,
    callback_url: String,
}

async fn scrap_manga(Json(payload): Json<ScrapMangaRequest>) -> (StatusCode, Json<Value>) {
    tokio::spawn(async move {
        let manga = get_manga_urls(&payload.slug, "token").await;
        send_info_about_manga(&payload.callback_url, &manga).await;
    });

    (StatusCode::OK, Json(json!({
        "success": true,
        "message": "Manga was sent successfully"
    })))
}

async fn handle_404() -> (StatusCode, Json<Value>) {
    (StatusCode::NOT_FOUND, Json(json!({
        "success": false,
        "message": "Route not found"
    })))
}

async fn get_manga_urls(slug: &str, telegraph_token: &str) -> PublishedManga {
    let chapter_urls_map: Arc<Mutex<HashMap<MangaChapter, Vec<String>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let mut chapters = mangalib::get_manga_chapters(slug).await.unwrap();
    chapters.reverse();
    let mut threads = vec![];
    let semaphore = Arc::new(Semaphore::new(8));
    for chapter in chapters.clone() {
        let urls = Arc::clone(&chapter_urls_map);
        let slug = slug.to_string();
        let semaphore = semaphore.clone();
        let thread = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let result = mangalib::get_manga_chapter_images(&slug, &chapter)
                .await
                .unwrap();

            let mut urls = urls.lock().unwrap();
            urls.insert(chapter.clone(), result);
        });

        threads.push(thread);
    }

    futures::future::join_all(threads).await;
    let chapter_urls_map = chapter_urls_map.lock().unwrap().clone();

    publish_manga(slug, &chapters, &chapter_urls_map, telegraph_token).await
}

#[derive(Debug, Serialize, Deserialize)]
struct PublishedManga {
    slug: String,
    chapters: Vec<PublishedMangaChapter>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PublishedMangaChapter {
    url: String,
    chapter: String,
    volume: String,
    images_urls: Vec<String>,
}

async fn publish_manga(
    slug: &str,
    chapters: &[MangaChapter],
    chapter_urls_map: &HashMap<MangaChapter, Vec<String>>,
    telegraph_token: &str,
) -> PublishedManga {
    let mut telegraph_urls: Vec<PublishedMangaChapter> = vec![];
    for chapter in chapters {
        let url_images = chapter_urls_map
            .get(chapter)
            .unwrap();
        let pages_nodes: Vec<NodeElement> = url_images
            .iter()
            .map(|x| NodeElement::img(x))
            .collect::<Vec<NodeElement>>();

        let chapter_url = publish_manga_chapter(slug, &pages_nodes, chapter, telegraph_token).await;
        telegraph_urls
            .push(PublishedMangaChapter {
                url: chapter_url,
                chapter: chapter.chapter_number.clone(),
                volume: chapter.chapter_volume.clone(),
                images_urls: url_images.clone(),
            });
        tokio::time::sleep(Duration::from_millis(1200)).await;
    }

    PublishedManga { slug: slug.to_string(), chapters: telegraph_urls }
}

async fn publish_manga_chapter(
    slug: &str,
    pages_nodes: &[NodeElement],
    chapter: &MangaChapter,
    telegraph_token: &str,
) -> String {
    let telegraph_title: String = format!(
        "{slug} v{}c{}",
        chapter.chapter_volume, chapter.chapter_number
    );

    telegraph::methods::create_page(telegraph_token, &telegraph_title, None, None, pages_nodes)
        .await
        .unwrap()
        .url
}

async fn send_info_about_manga(url: &str, manga: &PublishedManga) {
    reqwest::Client::new()
        .post(url)
        .json(manga)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
}