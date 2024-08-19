use mangalib::MangaChapter;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use telegraph::types::NodeElement;
use tokio::sync::Semaphore;

mod mangalib;
mod telegraph;

#[tokio::main]
async fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");
    rt.block_on(async {});
}

async fn get_manga_urls(slug: &str, telegraph_token: &str) -> Vec<String> {
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

async fn publish_manga(
    slug: &str,
    chapters: &[MangaChapter],
    chapter_urls_map: &HashMap<MangaChapter, Vec<String>>,
    telegraph_token: &str,
) -> Vec<String> {
    let mut telegraph_urls: Vec<String> = vec![];
    for chapter in chapters {
        let pages_nodes: Vec<NodeElement> = chapter_urls_map
            .get(chapter)
            .unwrap()
            .iter()
            .map(|x| NodeElement::img(x))
            .collect::<Vec<NodeElement>>();

        telegraph_urls
            .push(publish_manga_chapter(slug, &pages_nodes, chapter, telegraph_token).await);
        tokio::time::sleep(Duration::from_millis(1200)).await;
    }

    telegraph_urls
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
