#![allow(dead_code)]
#![allow(unused_variables)]

#[cfg(feature = "browser_client")]
pub mod browser_client;
mod deserializers;
#[cfg(feature = "http_client")]
pub mod http_client;
pub mod search;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to parse by serde: {0}")]
    SerdeParse(#[from] serde_json::Error),
    #[error("Failed to create browser: {0}")]
    BrowserCreate(String),
    #[error("Failed to create browser launch builder: {0}")]
    BrowserCreateBuilder(String),
    #[error("Failed to create browser tab: {0}")]
    BrowserTabCreate(String),
    #[error("Failed to navigate to url: {0}")]
    BrowserNavigate(String),
    #[error("Failed to set browser user agent: {0}")]
    SetUserAgent(String),
    #[error("Browser wait navigate too long: {0}")]
    BrowserWaitNavigateTooLong(String),
    #[error("Browser wait element too long: {0}")]
    BrowserWaitElementTooLong(String),
    #[error("Failed to get page content: {0}")]
    BrowserGetContent(String),
    #[error("Network error for URL {url}: {source}")]
    ReqwestNetwork { source: reqwest::Error, url: String },
    #[error("Failed to read response from {url}: {source}")]
    ReqwestResponseRead { source: reqwest::Error, url: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MangaPreview {
    #[serde(rename(deserialize = "manga_type"))]
    pub r#type: String,
    pub name: String,
    pub url: String,
    pub slug: String,
    pub image_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct MangaChapter {
    #[serde(deserialize_with = "deserializers::to_string")]
    pub chapter_volume: String,
    #[serde(deserialize_with = "deserializers::to_string")]
    pub chapter_number: String,
}

impl MangaChapter {
    pub fn new<T: Into<String>>(volume: T, number: T) -> Self {
        Self {
            chapter_volume: volume.into(),
            chapter_number: number.into(),
        }
    }
}

pub trait Client {
    async fn get_manga_chapter_images(
        &self,
        slug: &str,
        manga_chapter: &MangaChapter,
        chapter_index: usize,
        total_chapters: usize,
    ) -> Result<Vec<String>, Error>;

    async fn get_manga_chapters(&self, slug: &str) -> Result<Vec<MangaChapter>, Error>;
}
