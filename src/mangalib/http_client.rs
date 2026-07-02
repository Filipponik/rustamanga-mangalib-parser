use crate::mangalib::{Client, Error, MangaChapter, MangaListItem};
use axum::http::StatusCode;
use reqwest::{Method, RequestBuilder};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::time::Duration;
use tracing::debug;

const IMAGE_SERVER_PREFIX: &str = "https://img3.cdnlibs.org";
const MANGALIB_DEFAULT_BASE_URL: &str = "https://api.cdnlibs.org";
const REFERER_HEADER: &str = "https://mangalib.org/";
const SITE_ID_HEADER: &str = "1";
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Deserialize, Debug)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Deserialize, Debug)]
pub struct ApiResponseWithPagination<T> {
    pub data: T,
    pub meta: Meta,
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub current_page: u32,
    pub from: Option<u32>,
    pub to: Option<u32>,
    pub page: u32,
    pub next_page_url: bool,
}

#[derive(Deserialize, Debug)]
struct ImageResponse {
    pages: Vec<Image>,
}

#[derive(serde::Deserialize)]
pub struct BookmarkItem {
    pub meta: Option<BookmarkMeta>,
    pub media: BookmarkMedia,
}

#[derive(serde::Deserialize)]
pub struct BookmarkMedia {
    pub slug: String,
}

#[derive(serde::Deserialize)]
pub struct BookmarkMeta {
    pub item_number: Option<u32>,
}

impl From<BookmarkItem> for MangaListItem {
    fn from(item: BookmarkItem) -> Self {
        Self {
            slug: item.media.slug,
            index: item.meta.and_then(|meta| meta.item_number),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessToken(String);

impl AccessToken {
    #[must_use]
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RefreshToken(String);

impl RefreshToken {
    #[must_use]
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenPair {
    pub access_token: AccessToken,
    pub refresh_token: RefreshToken,
}

impl TokenPair {
    #[must_use]
    pub const fn new(access_token: AccessToken, refresh_token: RefreshToken) -> Self {
        Self {
            access_token,
            refresh_token,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewTokenPair {
    pub access_token: AccessToken,
    pub refresh_token: RefreshToken,
    pub expires_in: u64,
}

impl From<NewTokenPair> for TokenPair {
    fn from(pair: NewTokenPair) -> Self {
        Self {
            access_token: pair.access_token,
            refresh_token: pair.refresh_token,
        }
    }
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
    referer_header: Option<String>,
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
    pub fn referer_header(mut self, referer_header: impl Into<String>) -> Self {
        self.referer_header = Some(referer_header.into());
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
            referer_header: self
                .referer_header
                .unwrap_or_else(|| REFERER_HEADER.to_string()),
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
    referer_header: String,
    site_id_header: String,
    timeout: Duration,
    reqwest_client: reqwest::Client,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BookmarkListItem {
    pub id: u32,
    pub name: String,
    pub site_ids: Vec<u32>,
}

impl HttpClient {
    #[must_use]
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// # Errors
    /// - [`Error::ReqwestNetwork`]
    /// - [`Error::ReqwestResponseRead`]
    /// - [`Error::SerdeParse`]
    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T, Error> {
        let response = self.build_request(Method::GET, url).send().await;

        self.parse_response(url, response).await
    }

    async fn get_with_bearer<T: DeserializeOwned>(
        &self,
        url: &str,
        bearer: &str,
    ) -> Result<T, Error> {
        let response = self
            .build_request(Method::GET, url)
            .header("Authorization", format!("Bearer {bearer}"))
            .send()
            .await;

        self.parse_response(url, response).await
    }

    async fn post_with_bearer<T: DeserializeOwned>(
        &self,
        url: &str,
        bearer: &str,
        body: &Value,
    ) -> Result<T, Error> {
        let response = self
            .build_request(Method::POST, url)
            .json(body)
            .header("Authorization", format!("Bearer {bearer}"))
            .send()
            .await;

        self.parse_response(url, response).await
    }

    fn build_request(&self, method: Method, url: &str) -> RequestBuilder {
        self.reqwest_client
            .request(method, url)
            .header("Referer", &self.referer_header)
            .header("Site-Id", &self.site_id_header)
            .timeout(self.timeout)
    }

    /// # Errors
    /// - [`Error::ReqwestNetwork`] if network error occurs
    /// - [`Error::ReqwestResponseRead`] if cannot read response body
    /// - [`Error::SerdeParse`] if cannot parse response body to needed struct
    /// - [`Error::Throttling`] if throttling found
    /// - [`Error::ReqwestResponseStatus`] if response status code is not success (except 429 HTTP)
    async fn parse_response<T: DeserializeOwned>(
        &self,
        url: &str,
        reqwest_response: reqwest::Result<reqwest::Response>,
    ) -> Result<T, Error> {
        let response = reqwest_response.map_err(|e| Error::ReqwestNetwork {
            source: e,
            url: url.to_string(),
        })?;

        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            return Err(Error::Throttling);
        }

        if !response.status().is_success() {
            return Err(Error::ReqwestResponseStatus {
                status: response.status(),
                url: url.to_string(),
            });
        }

        let response_body = response
            .text()
            .await
            .map_err(|e| Error::ReqwestResponseRead {
                source: e,
                url: url.to_string(),
            })?;

        Ok(serde_json::from_str(&response_body)?)
    }

    /// # Errors
    /// - [`Error::ReqwestNetwork`]
    /// - [`Error::ReqwestResponseRead`]
    /// - [`Error::SerdeParse`]
    pub async fn get_bookmark_folders(
        &self,
        token: &str,
        user_id: u32,
    ) -> Result<Vec<BookmarkListItem>, Error> {
        let url = &format!("https://api.cdnlibs.org/api/bookmarks/folder/{user_id}");

        Ok(self
            .get_with_bearer::<ApiResponse<Vec<BookmarkListItem>>>(url, token)
            .await?
            .data)
    }

    /// # Errors
    /// - [`Error::ReqwestNetwork`]
    /// - [`Error::ReqwestResponseRead`]
    /// - [`Error::SerdeParse`]
    pub async fn get_bookmarks(
        &self,
        page: u32,
        bookmark_folder_id: u32,
        token: &str,
        user_id: u32,
    ) -> Result<ApiResponseWithPagination<Vec<BookmarkItem>>, Error> {
        let url = &format!(
            "https://api.cdnlibs.org/api/bookmarks?page={page}&sort_by=name&sort_type=desc&status={bookmark_folder_id}&user_id={user_id}"
        );

        debug!(
            user_id = user_id,
            page = page,
            bookmark_folder_id = bookmark_folder_id,
            url = url,
            "Getting bookmarks for user",
        );

        self.get_with_bearer::<ApiResponseWithPagination<Vec<BookmarkItem>>>(url, token)
            .await
    }

    /// # Errors
    /// - [`Error::ReqwestNetwork`]
    /// - [`Error::ReqwestResponseRead`]
    /// - [`Error::SerdeParse`]
    pub async fn refresh_access_token(
        &self,
        token_pair: &TokenPair,
    ) -> Result<NewTokenPair, Error> {
        let url = "https://api.cdnlibs.org/api/auth/oauth/token";
        debug!(url = url, "Refreshing access token");

        self.post_with_bearer(
            url,
            &token_pair.access_token.0,
            &json!({
                "grant_type": "refresh_token",
                "client_id": "1",
                "refresh_token": &token_pair.refresh_token.0,
                "scope": ""
            }),
        )
        .await
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

    async fn get_user_list(&self, token: &str, user_id: u32) -> Result<Vec<MangaListItem>, Error> {
        let folders: Vec<BookmarkListItem> = self
            .get_bookmark_folders(token, user_id)
            .await?
            .into_iter()
            .filter(|folder| folder.site_ids.contains(&1))
            .collect();

        let mut user_list = Vec::new();
        for folder in folders {
            let mut page = 1;
            loop {
                debug!(
                    user_id = user_id,
                    folder_id = folder.id,
                    folder_name = folder.name,
                    page = page,
                    current_count = user_list.len(),
                    "Requesting bookmarks for folder"
                );
                let response = self.get_bookmarks(page, folder.id, token, user_id).await?;
                let list_items: Vec<MangaListItem> =
                    response.data.into_iter().map(MangaListItem::from).collect();

                user_list.extend(list_items);
                if response.meta.next_page_url {
                    page += 1;
                } else {
                    break;
                }
            }
        }

        Ok(user_list)
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
        assert!(builder.referer_header.is_none());
        assert!(builder.site_id_header.is_none());
        assert!(builder.timeout.is_none());
        assert!(builder.reqwest_client.is_none());
    }

    #[tokio::test]
    async fn test_client_builder_change_state() {
        let builder = Builder::default()
            .image_server_prefix("test")
            .base_url("test2")
            .referer_header("test3")
            .site_id_header("test4")
            .timeout(Duration::from_secs(12345))
            .reqwest_client(reqwest::Client::new());

        assert_eq!(builder.image_server_prefix.unwrap(), "test");
        assert_eq!(builder.base_url.unwrap(), "test2");
        assert_eq!(builder.referer_header.unwrap(), "test3");
        assert_eq!(builder.site_id_header.unwrap(), "test4");
        assert_eq!(builder.timeout.unwrap(), Duration::from_secs(12345));
        assert!(builder.reqwest_client.is_some()); // i don't know how to check if reqwest_client is same
    }

    #[tokio::test]
    async fn test_client_builder_build_all_filled() {
        let client = Builder::default()
            .image_server_prefix("test")
            .base_url("test2")
            .referer_header("test3")
            .site_id_header("test4")
            .timeout(Duration::from_secs(12345))
            .reqwest_client(reqwest::Client::new())
            .build();

        assert_eq!(client.image_server_prefix, "test");
        assert_eq!(client.base_url, "test2");
        assert_eq!(client.referer_header, "test3");
        assert_eq!(client.site_id_header, "test4");
        assert_eq!(client.timeout, Duration::from_secs(12345));
        // assert!(client.reqwest_client.is_some()); // i don't know how to check if reqwest_client is same
    }

    #[tokio::test]
    async fn test_client_builder_build_all_default() {
        let client = Builder::default().build();

        assert_eq!(client.image_server_prefix, "https://img33.imgslib.link");
        assert_eq!(client.base_url, "https://api.cdnlibs.org");
        assert_eq!(client.referer_header, "https://mangalib.org/");
        assert_eq!(client.site_id_header, "1");
        assert_eq!(client.timeout, Duration::from_secs(60));
        // assert!(client.reqwest_client.is_some()); // i don't know how to check if reqwest_client is same
    }
}
