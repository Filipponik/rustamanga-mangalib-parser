#![allow(dead_code, unused_variables)]

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
    #[error("Failed to set browser user agent: {0}")]
    SetUserAgent(String),
    #[error("Network error for URL {url}: {source}")]
    ReqwestNetwork { source: reqwest::Error, url: String },
    #[error("Failed to read response from {url}: {source}")]
    ReqwestResponseRead { source: reqwest::Error, url: String },
    #[error("Throttling error")]
    Throttling,
    #[error("Bad response status for URL {url}: {status}")]
    ReqwestResponseStatus {
        status: reqwest::StatusCode,
        url: String,
    },
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MangaListItem {
    pub slug: String,
    pub index: Option<u32>,
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

pub trait Client: Clone + Send + Sync {
    fn get_manga_chapter_images(
        &self,
        slug: &str,
        manga_chapter: &MangaChapter,
        chapter_index: usize,
        total_chapters: usize,
    ) -> impl std::future::Future<Output = Result<Vec<String>, Error>> + std::marker::Send;

    fn get_manga_chapters(
        &self,
        slug: &str,
    ) -> impl std::future::Future<Output = Result<Vec<MangaChapter>, Error>> + std::marker::Send;

    fn get_user_list(
        &self,
        token: &str,
        user_id: u32,
    ) -> impl std::future::Future<Output = Result<Vec<MangaListItem>, Error>> + std::marker::Send;
}
