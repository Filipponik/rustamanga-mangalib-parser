use std::{ops::Add, sync::Arc, time::Duration};

use dashmap::DashMap;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{AcquireError, Semaphore},
    time::sleep,
};
use tracing::{error, info};

use crate::mangalib;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Mangalib error: {0}")]
    Mangalib(#[from] mangalib::Error),
    #[error("Chapter not found")]
    ChapterNotFound { chapter: mangalib::MangaChapter },
    #[error("Chapter not found for filter, {dto:?}")]
    ChapterNotFoundForFilter { dto: MangaScrappingParamsDto },
    #[error("Semaphore acquire error: {0}")]
    SemaphoreAcquire(#[from] AcquireError),
    #[error("Handle error")]
    Handle,
}

/// Manga slug to scrape, e.g. `solo-leveling`.
#[derive(Deserialize, Debug, utoipa::ToSchema)]
pub struct GetMangaWithOnlyChaptersParams {
    /// Manga slug to scrape.
    pub slug: String,
    /// URL where the scraped result will be published.
    pub callback_url: String,
}

/// Params for the `full` command: chapters with images.
#[derive(Deserialize, Debug, utoipa::ToSchema)]
pub struct GetMangaWithChaptersAndImagesParams {
    /// Manga slug to scrape.
    pub slug: String,
    /// URL where the scraped result will be published.
    pub callback_url: String,
    /// Only scrape chapters after this chapter number.
    pub after_chapter: Option<String>,
    /// Only scrape chapters after this volume number.
    pub after_volume: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishedManga {
    pub slug: String,
    pub chapters: Vec<PublishedMangaChapter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishedMangaChapter {
    pub chapter: String,
    pub volume: String,
    pub images_urls: Vec<String>,
}

/// Handle `full` manga request
///
/// # Errors
/// Any error can be returned while processing manga
pub async fn handle_full<T: mangalib::Client + 'static>(
    params: &GetMangaWithChaptersAndImagesParams,
    client: Arc<T>,
    sender: &reqwest::Client,
    semaphore_permits: usize,
) -> Result<(), Error> {
    let dto = MangaScrappingParamsDto {
        slug: params.slug.clone(),
        after_chapter: params.after_chapter.clone(),
        after_volume: params.after_volume.clone(),
    };

    let manga = get_manga_urls(&dto, semaphore_permits, client).await?;

    info!(callback_url = params.callback_url, "Sending manga",);
    let response = send_info_about_manga(&params.callback_url, &manga, sender).await;

    match response {
        Ok(body) => info!(body = body, "Successfully sent manga"),
        Err(err) => error!(manga_slug = dto.slug, "Error while sending manga: {err:?}"),
    }

    Ok(())
}

/// Handle `only_chapters` manga request (return only chapters without images)
///
/// # Errors
/// Any error can be returned while processing manga
pub async fn handle_only_chapters<T: mangalib::Client + 'static>(
    params: &GetMangaWithOnlyChaptersParams,
    client: Arc<T>,
    sender: &reqwest::Client,
) -> Result<(), Error> {
    let dto = MangaScrappingParamsDto {
        slug: params.slug.clone(),
        after_chapter: None,
        after_volume: None,
    };

    let manga = get_manga_chapters(&dto, client).await?;

    info!(callback_url = params.callback_url, "Sending manga",);
    let response = send_info_about_manga(&params.callback_url, &manga, sender).await;
    match response {
        Ok(body) => info!(body = body, "Successfully sent manga"),
        Err(err) => error!(manga_slug = dto.slug, "Error while sending manga: {err:?}"),
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct MangaScrappingParamsDto {
    pub slug: String,
    pub after_chapter: Option<String>,
    pub after_volume: Option<String>,
}

async fn get_manga_chapters<T: mangalib::Client + 'static>(
    dto: &MangaScrappingParamsDto,
    client: Arc<T>,
) -> Result<PublishedManga, Error> {
    let chapters = client.get_manga_chapters(&dto.slug).await?;
    let Some(chapters) = filter_chapters(chapters, dto) else {
        return Err(Error::ChapterNotFoundForFilter { dto: dto.clone() });
    };
    let chapters = chapters
        .into_iter()
        .map(|chapter| PublishedMangaChapter {
            chapter: chapter.chapter_number,
            volume: chapter.chapter_volume,
            images_urls: vec![],
        })
        .collect();

    Ok(PublishedManga {
        slug: dto.slug.clone(),
        chapters,
    })
}

async fn get_manga_urls<T: mangalib::Client + 'static>(
    dto: &MangaScrappingParamsDto,
    semaphore_permits: usize,
    client: Arc<T>,
) -> Result<PublishedManga, Error> {
    let chapter_urls_map: Arc<DashMap<mangalib::MangaChapter, Vec<String>>> =
        Arc::new(DashMap::new());
    let chapters = client.get_manga_chapters(&dto.slug).await?;
    let Some(chapters) = filter_chapters(chapters, dto) else {
        return Err(Error::ChapterNotFoundForFilter { dto: dto.clone() });
    };
    let semaphore = Arc::new(Semaphore::new(semaphore_permits));

    let mut handles = Vec::new();
    let chapters_len = chapters.len();
    for (index, chapter) in chapters.iter().enumerate() {
        let urls = Arc::clone(&chapter_urls_map);
        let slug = dto.slug.clone();
        let semaphore = semaphore.clone();
        let chapter = chapter.clone();
        let client = Arc::clone(&client);
        handles.push(tokio::spawn(async move {
            #[allow(unused_variables)]
            let permit = semaphore.acquire().await?;
            let result = retry(
                || async {
                    client
                        .get_manga_chapter_images(&slug, &chapter, index + 1, chapters_len)
                        .await
                },
                5,
                |err: &mangalib::Error| {
                    // Do NOT retry if throttling
                    matches!(err, mangalib::Error::Throttling)
                },
            )
            .await?;
            urls.insert(chapter, result);

            Ok::<(), Error>(())
        }));
    }

    for handle in handles {
        handle.await.map_err(|_| Error::Handle)??;
    }

    prepare_manga_for_publish(&dto.slug, &chapters, &chapter_urls_map)
}

fn filter_chapters(
    chapters: Vec<mangalib::MangaChapter>,
    dto: &MangaScrappingParamsDto,
) -> Option<Vec<mangalib::MangaChapter>> {
    let (chapter_num, volume_num) = match (&dto.after_chapter, &dto.after_volume) {
        (Some(c), Some(v)) => (c.clone(), v.clone()),
        _ => return Some(chapters),
    };

    let position = chapters.iter().position(|chapter| {
        chapter_num.eq(&chapter.chapter_number) && volume_num.eq(&chapter.chapter_volume)
    });

    position.map(|index| chapters.into_iter().skip(index + 1).collect())
}

fn prepare_manga_for_publish(
    slug: &str,
    input_chapters: &[mangalib::MangaChapter],
    chapter_urls_map: &DashMap<mangalib::MangaChapter, Vec<String>>,
) -> Result<PublishedManga, Error> {
    let mut output_chapters: Vec<PublishedMangaChapter> = vec![];
    for chapter in input_chapters {
        let Some(url_images) = chapter_urls_map.get(chapter) else {
            return Err(Error::ChapterNotFound {
                chapter: chapter.clone(),
            });
        };

        output_chapters.push(PublishedMangaChapter {
            chapter: chapter.chapter_number.clone(),
            volume: chapter.chapter_volume.clone(),
            images_urls: url_images.clone(),
        });
    }

    Ok(PublishedManga {
        slug: slug.to_string(),
        chapters: output_chapters,
    })
}

async fn send_info_about_manga(
    url: &str,
    manga: &PublishedManga,
    sender: &reqwest::Client,
) -> reqwest::Result<String> {
    sender.post(url).json(manga).send().await?.text().await
}

async fn retry<T, E: std::fmt::Debug, F>(
    decorated: impl Fn() -> F,
    max_retries: u32,
    should_not_retry: impl Fn(&E) -> bool,
) -> Result<T, E>
where
    F: Future<Output = Result<T, E>>,
{
    let mut backoff = Duration::from_secs(1);

    for attempt in 1..=max_retries {
        match decorated().await {
            Ok(value) => return Ok(value),
            Err(err) if attempt >= max_retries => {
                error!(attempt = attempt, err = ?err, "Last attempt failed, retries exceeded");
                return Err(err);
            }
            Err(err) => {
                error!(attempt = attempt, err = ?err, "Attempt failed");

                if should_not_retry(&err) {
                    error!(attempt = attempt, err = ?err, "Result marked as should not retry");
                    return Err(err);
                }

                sleep(backoff).await;

                backoff = backoff
                    .mul_f32(3.0)
                    .add(Duration::from_millis(rand::rng().random_range(0..1000)))
                    .min(Duration::from_mins(1));
            }
        }
    }

    unreachable!();
}
