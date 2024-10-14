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

#[derive(Debug)]
struct PublishedManga {
    slug: String,
    chapters: Vec<PublishedMangaChapter>,
}

#[derive(Debug)]
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
