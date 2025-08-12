use crate::mangalib;
use crate::mangalib::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::sync::{AcquireError, Semaphore};
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

#[derive(Debug, Error)]
pub enum Error {
    #[error("Mangalib error: {0}")]
    Mangalib(#[from] mangalib::Error),
    #[error("Chapter not found")]
    ChapterNotFound { chapter: mangalib::MangaChapter },
    #[error("Chapter not found for filter, {dto:?}")]
    ChapterNotFoundForFilter { dto: MangaScrappingParamsDto },
    #[error("Can't get mutex lock")]
    MutexLock,
    #[error("Semaphore acquire error: {0}")]
    SemaphoreAcquire(#[from] AcquireError),
    #[error("Handle error")]
    Handle,
}

#[derive(Deserialize)]
pub struct ScrapMangaRequest {
    slug: String,
    callback_url: String,
    after_chapter: Option<String>,
    after_volume: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MangaScrappingParamsDto {
    pub slug: String,
    pub after_chapter: Option<String>,
    pub after_volume: Option<String>,
}

pub async fn process(chrome_max_count: u16, payload: ScrapMangaRequest) -> Result<(), Error> {
    let dto = MangaScrappingParamsDto {
        slug: payload.slug,
        after_chapter: payload.after_chapter,
        after_volume: payload.after_volume,
    };
    let manga = get_manga_urls(&dto, chrome_max_count).await?;
    info!(callback_url = payload.callback_url, "Sending manga",);
    let response = send_info_about_manga(&payload.callback_url, &manga).await;
    match response {
        Ok(body) => info!(body = body, "Successfully sent manga"),
        Err(err) => error!(manga_slug = dto.slug, "Error while sending manga: {err:?}"),
    }

    Ok(())
}

async fn get_manga_urls(
    dto: &MangaScrappingParamsDto,
    chrome_max_count: u16,
) -> Result<PublishedManga, Error> {
    let chapter_urls_map: Arc<Mutex<HashMap<mangalib::MangaChapter, Vec<String>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let chapters = mangalib::HeadlessBrowserClient::builder()
        .build()
        .get_manga_chapters(&dto.slug)?;
    let chapters = match filter_chapters(chapters, dto) {
        None => return Err(Error::ChapterNotFoundForFilter { dto: dto.clone() }),
        Some(c) => c,
    };
    let semaphore = Arc::new(Semaphore::new(chrome_max_count as usize));

    let mut handles = Vec::new();
    let chapters_len = chapters.len();
    for (index, chapter) in chapters.iter().enumerate() {
        let urls = Arc::clone(&chapter_urls_map);
        let slug = dto.slug.to_string();
        let semaphore = semaphore.clone();
        let chapter = chapter.clone();
        handles.push(tokio::spawn(async move {
            let _permit = semaphore.acquire().await?;
            let result = retry!(
                mangalib::HeadlessBrowserClient::builder()
                    .build()
                    .get_manga_chapter_images(&slug, &chapter, index + 1, chapters_len)
            )?;
            urls.lock()
                .map_err(|_| Error::MutexLock)?
                .insert(chapter, result);
            Ok::<(), Error>(())
        }));
    }

    for handle in handles {
        handle.await.map_err(|_| Error::Handle)??; // Двойной `?` для JoinError и вашей Error
    }

    let chapter_urls_map = chapter_urls_map
        .lock()
        .map_err(|_| Error::MutexLock)?
        .clone();

    prepare_manga_for_publish(&dto.slug, &chapters, &chapter_urls_map)
}

fn filter_chapters(
    chapters: Vec<mangalib::MangaChapter>,
    dto: &MangaScrappingParamsDto,
) -> Option<Vec<mangalib::MangaChapter>> {
    let (chapter_num, volume_num) = match (&dto.after_chapter, &dto.after_volume) {
        (Some(c), Some(v)) => (c.to_string(), v.to_string()),
        _ => return Some(chapters),
    };

    let position = chapters.iter().position(|chapter| {
        chapter_num.eq(&chapter.chapter_number) && volume_num.eq(&chapter.chapter_volume)
    });

    position.map(|index| chapters.into_iter().skip(index + 1).collect())
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

fn prepare_manga_for_publish(
    slug: &str,
    chapters: &[mangalib::MangaChapter],
    chapter_urls_map: &HashMap<mangalib::MangaChapter, Vec<String>>,
) -> Result<PublishedManga, Error> {
    let mut telegraph_urls: Vec<PublishedMangaChapter> = vec![];
    for chapter in chapters {
        let Some(url_images) = chapter_urls_map.get(chapter) else {
            return Err(Error::ChapterNotFound {
                chapter: chapter.clone(),
            });
        };

        telegraph_urls.push(PublishedMangaChapter {
            url: None,
            chapter: chapter.chapter_number.clone(),
            volume: chapter.chapter_volume.clone(),
            images_urls: url_images.clone(),
        });
    }

    Ok(PublishedManga {
        slug: slug.to_string(),
        chapters: telegraph_urls,
    })
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
