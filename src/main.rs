use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use mangalib::MangaChapter;
use telegraph::types::{img, NodeElement};

mod mangalib;
mod telegraph;

#[tokio::main]
async fn main() -> () {
}

async fn get_manga_urls(slug: &'static str, telegraph_token: &'static str) -> Vec<String> {
    let urls: Arc<Mutex<HashMap<MangaChapter, Vec<String>>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut chapters = mangalib::get_manga_chapters(slug).await.unwrap();
    chapters.reverse();
    let mut threads = vec![];

    for chapter in chapters.clone() {
        let urls = Arc::clone(&urls);
        let thread = thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(mangalib::get_manga_chapter_images_by_js(&slug, &chapter)).unwrap();

            let mut urls = urls.lock().unwrap();
            urls.insert(chapter.clone(), result);
        });

        threads.push(thread);
    }

    for thread in threads {
        thread.join().unwrap();
    }

    let urls: HashMap<MangaChapter, Vec<String>> = urls.lock().unwrap().clone();

    let mut urls1 = vec![];
    for chapter in chapters {
        let pages = urls.get(&chapter).unwrap().iter().map(|x| img(x)).collect::<Vec<NodeElement>>();
        let name = format!("{slug} v{}c{}", chapter.chapter_volume, chapter.chapter_number);
        let url = telegraph::methods::create_page(
            telegraph_token,
            &name,
            None,
            None,
            &pages
        ).await.unwrap().result.url;

        urls1.push(url);
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    urls1
}
