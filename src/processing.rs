use crate::mangalib;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use tracing::{error, info};

macro_rules! retry {
    ($f:expr, $count:expr) => {{
        let mut tries = 0;
        let result = loop {
            let result = $f;
            tries += 1;
            if result.is_ok() || tries >= $count {
                break result;
            }
        };
        result
    }};
    ($f:expr) => {
        retry!($f, 5)
    };
}

#[derive(Deserialize)]
pub struct ScrapMangaRequest {
    slug: String,
    callback_url: String,
    after_chapter: Option<String>,
    after_volume: Option<String>,
}

struct MangaScrappingParamsDto {
    slug: String,
    after_chapter: Option<String>,
    after_volume: Option<String>,
}

pub async fn process(chrome_max_count: u16, payload: ScrapMangaRequest) {
    let dto = MangaScrappingParamsDto {
        slug: payload.slug,
        after_chapter: payload.after_chapter,
        after_volume: payload.after_volume,
    };
    let manga = get_manga_urls(&dto, chrome_max_count).await;
    info!("Sending manga to {}", payload.callback_url);
    let response = send_info_about_manga(&payload.callback_url, &manga).await;
    match response {
        Ok(body) => info!("Successfully sent manga: {body}"),
        Err(err) => error!("Error while sending manga: {err:?}"),
    }
}

async fn get_manga_urls(dto: &MangaScrappingParamsDto, chrome_max_count: u16) -> PublishedManga {
    let chapter_urls_map: Arc<Mutex<HashMap<mangalib::MangaChapter, Vec<String>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let chapters = mangalib::get_manga_chapters(&dto.slug).await.unwrap();
    let chapters = filter_chapters(chapters, dto);
    let mut threads = vec![];
    let semaphore = Arc::new(Semaphore::new(chrome_max_count as usize));
    for chapter in chapters.clone() {
        let urls = Arc::clone(&chapter_urls_map);
        let slug = dto.slug.to_string();
        let semaphore = semaphore.clone();
        let thread = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let result = retry!(mangalib::get_manga_chapter_images(&slug, &chapter).await).unwrap();
            let mut urls = urls.lock().unwrap();
            urls.insert(chapter.clone(), result);
        });

        threads.push(thread);
    }

    futures::future::join_all(threads).await;
    let chapter_urls_map = chapter_urls_map.lock().unwrap().clone();

    publish_manga(&dto.slug, &chapters, &chapter_urls_map).await
}

fn filter_chapters(chapters: Vec<mangalib::MangaChapter>, dto: &MangaScrappingParamsDto) -> Vec<mangalib::MangaChapter> {
    let (chapter_num, volume_num) = match (&dto.after_chapter, &dto.after_volume) {
        (Some(c), Some(v)) => (c.to_string(), v.to_string()),
        _ => return chapters,
    };

    let position = chapters.iter().position(|chapter| {
        chapter_num.eq(&chapter.chapter_number) && volume_num.eq(&chapter.chapter_volume)
    });

    match position {
        None => chapters,
        Some(index) => chapters.into_iter().skip(index + 1).collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishedManga {
    pub slug: String,
    pub chapters: Vec<PublishedMangaChapter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishedMangaChapter {
    pub url: Option<String>,
    pub chapter: String,
    pub volume: String,
    pub images_urls: Vec<String>,
}

async fn publish_manga(
    slug: &str,
    chapters: &[mangalib::MangaChapter],
    chapter_urls_map: &HashMap<mangalib::MangaChapter, Vec<String>>,
) -> PublishedManga {
    let mut telegraph_urls: Vec<PublishedMangaChapter> = vec![];
    for chapter in chapters {
        let url_images = chapter_urls_map.get(chapter).unwrap();
        telegraph_urls.push(PublishedMangaChapter {
            url: None,
            chapter: chapter.chapter_number.clone(),
            volume: chapter.chapter_volume.clone(),
            images_urls: url_images.clone(),
        });
    }

    PublishedManga {
        slug: slug.to_string(),
        chapters: telegraph_urls,
    }
}

async fn send_info_about_manga(url: &str, manga: &PublishedManga) -> reqwest::Result<String> {
    reqwest::Client::new()
        .post(url)
        .json(manga)
        .send()
        .await?
        .text()
        .await
}
