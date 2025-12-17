use crate::mangalib;
use dashmap::DashMap;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{AcquireError, Semaphore};
use tokio::time::sleep;
use tracing::{error, info};

pub mod commands;

#[derive(Debug, Error)]
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

#[derive(Deserialize, Debug)]
pub struct ScrapMangaRequest {
    #[serde(default)]
    mode: ScrapMangaMode,
    slug: String,
    callback_url: String,
    after_chapter: Option<String>,
    after_volume: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub enum ScrapMangaMode {
    #[serde(rename = "full")]
    #[default]
    Full,
    #[serde(rename = "only_chapters")]
    OnlyChapters,
}

#[derive(Debug, Clone)]
pub struct MangaScrappingParamsDto {
    pub slug: String,
    pub after_chapter: Option<String>,
    pub after_volume: Option<String>,
}

#[derive(Clone)]
pub struct Processor<TClient: mangalib::Client> {
    client: Arc<TClient>,
    sender: reqwest::Client,
}

impl<TClient: mangalib::Client + 'static> Processor<TClient> {
    pub fn new(client: TClient, sender: Option<reqwest::Client>) -> Self {
        Self {
            client: Arc::new(client),
            sender: sender.unwrap_or_default(),
        }
    }

    /// Processes an incoming scrap request using the configured client and sender.
    ///
    /// # Errors
    /// Returns an error if fetching chapters, filtering chapters, or sending the processed manga fails.
    pub async fn process(
        &self,
        chrome_max_count: u16,
        payload: ScrapMangaRequest,
    ) -> Result<(), Error> {
        let dto = MangaScrappingParamsDto {
            slug: payload.slug,
            after_chapter: payload.after_chapter,
            after_volume: payload.after_volume,
        };

        let manga = match payload.mode {
            ScrapMangaMode::Full => self.get_manga_urls(&dto, chrome_max_count).await?,
            ScrapMangaMode::OnlyChapters => self.get_manga_chapters(&dto).await?,
        };

        info!(callback_url = payload.callback_url, "Sending manga",);
        let response = self
            .send_info_about_manga(&payload.callback_url, &manga)
            .await;
        match response {
            Ok(body) => info!(body = body, "Successfully sent manga"),
            Err(err) => error!(manga_slug = dto.slug, "Error while sending manga: {err:?}"),
        }

        Ok(())
    }

    async fn get_manga_chapters(
        &self,
        dto: &MangaScrappingParamsDto,
    ) -> Result<PublishedManga, Error> {
        let chapters = self.client.get_manga_chapters(&dto.slug).await?;
        let Some(chapters) = Self::filter_chapters(chapters, dto) else {
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

    async fn get_manga_urls(
        &self,
        dto: &MangaScrappingParamsDto,
        chrome_max_count: u16,
    ) -> Result<PublishedManga, Error> {
        let chapter_urls_map: Arc<DashMap<mangalib::MangaChapter, Vec<String>>> =
            Arc::new(DashMap::new());
        let chapters = self.client.get_manga_chapters(&dto.slug).await?;
        let Some(chapters) = Self::filter_chapters(chapters, dto) else {
            return Err(Error::ChapterNotFoundForFilter { dto: dto.clone() });
        };
        let semaphore = Arc::new(Semaphore::new(chrome_max_count as usize));

        let mut handles = Vec::new();
        let chapters_len = chapters.len();
        for (index, chapter) in chapters.iter().enumerate() {
            let urls = Arc::clone(&chapter_urls_map);
            let slug = dto.slug.clone();
            let semaphore = semaphore.clone();
            let chapter = chapter.clone();
            let client = Arc::clone(&self.client);
            handles.push(tokio::spawn(async move {
                #[allow(unused_variables)]
                let permit = semaphore.acquire().await?;
                let result = Self::retry(
                    || async {
                        client
                            .get_manga_chapter_images(&slug, &chapter, index + 1, chapters_len)
                            .await
                    },
                    5,
                )
                .await?;
                urls.insert(chapter, result);

                Ok::<(), Error>(())
            }));
        }

        for handle in handles {
            handle.await.map_err(|_| Error::Handle)??;
        }

        Self::prepare_manga_for_publish(&dto.slug, &chapters, &chapter_urls_map)
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
        &self,
        url: &str,
        manga: &PublishedManga,
    ) -> reqwest::Result<String> {
        self.sender.post(url).json(manga).send().await?.text().await
    }

    async fn retry<T, E: std::fmt::Debug, F>(
        decorated: impl Fn() -> F,
        max_retries: u32,
    ) -> Result<T, E>
    where
        F: Future<Output = Result<T, E>>,
    {
        let mut backoff = Duration::from_millis(500);

        for attempt in 1..=max_retries {
            match decorated().await {
                Ok(value) => return Ok(value),
                Err(err) if attempt >= max_retries => return Err(err),
                Err(err) => {
                    error!(attempt = attempt, err = ?err, "Attempt failed");
                    sleep(backoff).await;

                    backoff = backoff
                        .mul_f32(2.0)
                        .add(Duration::from_millis(rand::rng().random_range(0..100)))
                        .min(Duration::from_secs(30));
                }
            }
        }

        unreachable!();
    }
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
