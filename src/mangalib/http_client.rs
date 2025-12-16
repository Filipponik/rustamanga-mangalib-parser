use std::time::Duration;

use crate::mangalib::{Client, Error, MangaChapter};
use serde::{Deserialize, de::DeserializeOwned};
use tracing::debug;

const IMAGE_SERVER_PREFIX: &str = "https://img33.imgslib.link";
const MANGALIB_DEFAULT_BASE_URL: &str = "https://api.cdnlibs.org";
const REFERRER_HEADER: &str = "https://mangalib.org/";
const SITE_ID_HEADER: &str = "1";
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Deserialize, Debug)]
struct ApiResponse<T> {
    data: T,
}

#[derive(Deserialize, Debug)]
struct ImageResponse {
    pages: Vec<Image>,
}

#[derive(Deserialize, Debug)]
struct Image {
    id: u128,
    #[serde(rename = "image")]
    file_name: String,
    height: u32,
    width: u32,
    url: String,
    #[serde(deserialize_with = "crate::mangalib::deserializers::to_string")]
    ratio: String,
}

#[derive(Deserialize, Debug)]
struct Chapter {
    id: u128,
    index: u128,
    item_number: u128,
    volume: String,
    number: String,
    number_secondary: Option<String>,
    name: Option<String>,
}

impl From<Chapter> for MangaChapter {
    fn from(chapter: Chapter) -> Self {
        Self::new(chapter.volume, chapter.number)
    }
}

#[derive(Default)]
pub struct Builder {
    image_server_prefix: Option<String>,
    base_url: Option<String>,
    referrer_header: Option<String>,
    site_id_header: Option<String>,
    timeout: Option<Duration>,
    reqwest_client: Option<reqwest::Client>,
}

impl Builder {
    #[must_use]
    pub fn image_server_prefix(mut self, image_server_prefix: impl Into<String>) -> Self {
        self.image_server_prefix = Some(image_server_prefix.into());
        self
    }

    #[must_use]
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    #[must_use]
    pub fn referrer_header(mut self, referrer_header: impl Into<String>) -> Self {
        self.referrer_header = Some(referrer_header.into());
        self
    }

    #[must_use]
    pub fn site_id_header(mut self, site_id_header: impl Into<String>) -> Self {
        self.site_id_header = Some(site_id_header.into());
        self
    }

    #[must_use]
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    #[must_use]
    pub fn reqwest_client(mut self, reqwest_client: reqwest::Client) -> Self {
        self.reqwest_client = Some(reqwest_client);
        self
    }

    #[must_use]
    pub fn build(self) -> HttpClient {
        HttpClient {
            image_server_prefix: self
                .image_server_prefix
                .unwrap_or_else(|| IMAGE_SERVER_PREFIX.to_string()),
            base_url: self
                .base_url
                .unwrap_or_else(|| MANGALIB_DEFAULT_BASE_URL.to_string()),
            referrer_header: self
                .referrer_header
                .unwrap_or_else(|| REFERRER_HEADER.to_string()),
            site_id_header: self
                .site_id_header
                .unwrap_or_else(|| SITE_ID_HEADER.to_string()),
            timeout: self.timeout.unwrap_or(DEFAULT_REQUEST_TIMEOUT),
            reqwest_client: self.reqwest_client.unwrap_or_default(),
        }
    }
}

#[derive(Clone)]
pub struct HttpClient {
    image_server_prefix: String,
    base_url: String,
    referrer_header: String,
    site_id_header: String,
    timeout: Duration,
    reqwest_client: reqwest::Client,
}

impl HttpClient {
    #[must_use]
    pub fn builder() -> Builder {
        Builder::default()
    }

    async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T, Error> {
        let response = self
            .reqwest_client
            .get(url)
            .header("Referrer", &self.referrer_header)
            .header("Site-Id", &self.site_id_header)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| Error::ReqwestNetwork {
                source: e,
                url: url.to_string(),
            })?
            .text()
            .await
            .map_err(|e| Error::ReqwestResponseRead {
                source: e,
                url: url.to_string(),
            })?;

        Ok(serde_json::from_str(&response)?)
    }
}

impl Client for HttpClient {
    async fn get_manga_chapter_images(
        &self,
        slug: &str,
        manga_chapter: &MangaChapter,
        chapter_index: usize,
        total_chapters: usize,
    ) -> Result<Vec<String>, Error> {
        let url = &format!(
            "{}/api/manga/{slug}/chapter?number={}&volume={}",
            self.base_url, manga_chapter.chapter_number, manga_chapter.chapter_volume
        );
        debug!(
            chapter_index = chapter_index,
            total_chapters = total_chapters,
            url = url,
            "Searching manga chapter image urls",
        );

        let image_response: ApiResponse<ImageResponse> = self.get(url).await?;
        let images = image_response
            .data
            .pages
            .into_iter()
            .map(|item| format!("{}{}", self.image_server_prefix, item.url))
            .collect();

        Ok(images)
    }

    async fn get_manga_chapters(&self, slug: &str) -> Result<Vec<MangaChapter>, Error> {
        let url = &format!("{}/api/manga/{slug}/chapters", self.base_url);
        debug!(manga_slug = slug, url = url, "Searching manga chapters",);
        let chapters_response: ApiResponse<Vec<Chapter>> = self.get(url).await?;

        debug!(
            manga_slug = slug,
            "Found {} chapters",
            chapters_response.data.len()
        );

        let chapters = chapters_response
            .data
            .into_iter()
            .map(MangaChapter::from)
            .collect();

        Ok(chapters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_builder_empty() {
        let builder = Builder::default();
        assert!(builder.image_server_prefix.is_none());
        assert!(builder.base_url.is_none());
        assert!(builder.referrer_header.is_none());
        assert!(builder.site_id_header.is_none());
        assert!(builder.timeout.is_none());
        assert!(builder.reqwest_client.is_none());
    }

    #[tokio::test]
    async fn test_client_builder_change_state() {
        let builder = Builder::default()
            .image_server_prefix("test")
            .base_url("test2")
            .referrer_header("test3")
            .site_id_header("test4")
            .timeout(Duration::from_secs(12345))
            .reqwest_client(reqwest::Client::new());

        assert_eq!(builder.image_server_prefix.unwrap(), "test");
        assert_eq!(builder.base_url.unwrap(), "test2");
        assert_eq!(builder.referrer_header.unwrap(), "test3");
        assert_eq!(builder.site_id_header.unwrap(), "test4");
        assert_eq!(builder.timeout.unwrap(), Duration::from_secs(12345));
        assert!(builder.reqwest_client.is_some()); // i don't know how to check if reqwest_client is same
    }

    #[tokio::test]
    async fn test_client_builder_build_all_filled() {
        let client = Builder::default()
            .image_server_prefix("test")
            .base_url("test2")
            .referrer_header("test3")
            .site_id_header("test4")
            .timeout(Duration::from_secs(12345))
            .reqwest_client(reqwest::Client::new())
            .build();

        assert_eq!(client.image_server_prefix, "test");
        assert_eq!(client.base_url, "test2");
        assert_eq!(client.referrer_header, "test3");
        assert_eq!(client.site_id_header, "test4");
        assert_eq!(client.timeout, Duration::from_secs(12345));
        // assert!(client.reqwest_client.is_some()); // i don't know how to check if reqwest_client is same
    }

    #[tokio::test]
    async fn test_client_builder_build_all_default() {
        let client = Builder::default().build();

        assert_eq!(client.image_server_prefix, "https://img33.imgslib.link");
        assert_eq!(client.base_url, "https://api.cdnlibs.org");
        assert_eq!(client.referrer_header, "https://mangalib.org/");
        assert_eq!(client.site_id_header, "1");
        assert_eq!(client.timeout, Duration::from_secs(60));
        // assert!(client.reqwest_client.is_some()); // i don't know how to check if reqwest_client is same
    }
}
