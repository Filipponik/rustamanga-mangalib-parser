#![allow(dead_code)]
#![allow(unused_variables)]

mod builder;
pub mod search;

use crate::mangalib::builder::Builder;
use headless_chrome::{Browser, LaunchOptions};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;
use tracing::debug;

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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ImageInnerList {
    data: ImageInnerListData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ImageInnerListData {
    pages: Vec<ImageInner>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ImageInner {
    id: u128,
    image: String,
    height: u32,
    width: u32,
    url: String,
    #[serde(deserialize_with = "to_string")]
    ratio: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MangaPreview {
    #[serde(rename(deserialize = "manga_type"))]
    r#type: String,
    name: String,
    url: String,
    slug: String,
    image_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct MangaChapter {
    #[serde(deserialize_with = "to_string")]
    pub chapter_volume: String,
    #[serde(deserialize_with = "to_string")]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChapterInner {
    id: u128,
    index: u128,
    item_number: u128,
    volume: String,
    number: String,
    number_secondary: Option<String>,
    name: Option<String>,
}

pub struct HeadlessBrowserClient {
    user_agent: String,
    accept_language: String,
    platform: String,
    image_server_prefix: String,
    base_url: String,
}

impl HeadlessBrowserClient {
    pub fn builder() -> Builder {
        Builder::default()
    }

    fn parse<T>(&self, url: &str, debug_message_prefix: &str) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let browser = Self::get_browser()?;
        let tab = browser
            .new_tab()
            .map_err(|err| Error::BrowserTabCreate(err.to_string()))?;
        debug!("{} {url}", &debug_message_prefix);

        tab.set_user_agent(
            &self.user_agent,
            Some(&self.accept_language),
            Some(&self.platform),
        )
        .map_err(|err| Error::SetUserAgent(err.to_string()))?;
        tab.navigate_to(url)
            .map_err(|err| Error::BrowserNavigate(err.to_string()))?
            .wait_until_navigated()
            .map_err(|err| Error::BrowserWaitNavigateTooLong(err.to_string()))?;

        let text = tab
            .wait_for_element("body > pre")
            .map_err(|err| Error::BrowserWaitElementTooLong(err.to_string()))?
            .get_inner_text()
            .map_err(|err| Error::BrowserGetContent(err.to_string()))?;

        Ok(serde_json::from_str(&text)?)
    }

    fn get_browser() -> Result<Browser, Error> {
        let options = LaunchOptions::default_builder()
            .sandbox(false)
            .build()
            .map_err(|err| Error::BrowserCreateBuilder(err.to_string()))?;

        Browser::new(options).map_err(|err| Error::BrowserCreate(err.to_string()))
    }
}

impl Client for HeadlessBrowserClient {
    fn get_manga_chapter_images(
        &self,
        slug: &str,
        manga_chapter: &MangaChapter,
        chapter_index: usize,
        total_chapters: usize,
    ) -> Result<Vec<String>, Error> {
        let image_inner_list: ImageInnerList = self.parse(
            &format!(
                "{}/api/manga/{slug}/chapter?number={}&volume={}",
                self.base_url, manga_chapter.chapter_number, manga_chapter.chapter_volume
            ),
            &format!("[{chapter_index}/{total_chapters}] Searching manga chapter urls"),
        )?;

        let images = image_inner_list
            .data
            .pages
            .into_iter()
            .map(|item| format!("{}{}", self.image_server_prefix, item.url))
            .collect();

        Ok(images)
    }

    fn get_manga_chapters(&self, slug: &str) -> Result<Vec<MangaChapter>, Error> {
        let chapter_inner_list: ChapterInnerList = self.parse(
            &format!("{}/api/manga/{slug}/chapters", self.base_url),
            slug,
        )?;

        debug!("Found {} chapters", chapter_inner_list.data.len());

        let chapters = chapter_inner_list
            .data
            .into_iter()
            .map(|chapter_inner| MangaChapter::new(chapter_inner.volume, chapter_inner.number))
            .collect();

        Ok(chapters)
    }
}

pub trait Client {
    fn get_manga_chapter_images(
        &self,
        slug: &str,
        manga_chapter: &MangaChapter,
        chapter_index: usize,
        total_chapters: usize,
    ) -> Result<Vec<String>, Error>;

    fn get_manga_chapters(&self, slug: &str) -> Result<Vec<MangaChapter>, Error>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChapterInnerList {
    data: Vec<ChapterInner>,
}

fn to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde::de::Unexpected;
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;

    match value {
        Value::Number(num) => Ok(num.to_string()),
        Value::String(s) => Ok(s),
        _ => Err(Error::invalid_type(
            Unexpected::Other("non-number/string value"),
            &"a number or string",
        )),
    }
}
