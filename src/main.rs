use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use mangalib::MangaChapter;
use telegraph::types::NodeElement;

mod mangalib;
mod telegraph;

#[tokio::main]
async fn main() -> () {
}

async fn get_manga_urls(slug: &'static str, telegraph_token: &'static str) -> Vec<String> {
    let chapter_urls_map: Arc<Mutex<HashMap<MangaChapter, Vec<String>>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut chapters = mangalib::get_manga_chapters(slug).await.unwrap();
    chapters.reverse();
    let mut threads = vec![];

    for chapter in chapters.clone() {
        let urls = Arc::clone(&chapter_urls_map);
        let thread = thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(mangalib::get_manga_chapter_images(&slug, &chapter)).unwrap();

            let mut urls = urls.lock().unwrap();
            urls.insert(chapter.clone(), result);
        });

        threads.push(thread);
    }

    for thread in threads {
        thread.join().unwrap();
    }
    let chapter_urls_map = chapter_urls_map.lock().unwrap().clone();

    publish_manga(slug, &chapters, &chapter_urls_map, telegraph_token).await
}

async fn publish_manga(slug: &'static str, chapters: &[MangaChapter], chapter_urls_map: &HashMap<MangaChapter, Vec<String>>, telegraph_token: &'static str) -> Vec<String> {
    let mut telegraph_urls: Vec<String> = vec![];
    for chapter in chapters {
        let pages_nodes: Vec<NodeElement> = chapter_urls_map.get(&chapter).unwrap().iter().map(|x| NodeElement::img(x)).collect::<Vec<NodeElement>>();
        let telegraph_title: String = format!("{slug} v{}c{}", chapter.chapter_volume, chapter.chapter_number);
        let telegraph_url: String = telegraph::methods::create_page(
            telegraph_token,
            &telegraph_title,
            None,
            None,
            &pages_nodes
        ).await.unwrap().url;

        telegraph_urls.push(telegraph_url);
        tokio::time::sleep(Duration::from_millis(1500)).await;
    }

    telegraph_urls
}
