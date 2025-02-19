#![allow(dead_code)]
#![allow(unused_variables)]
use headless_chrome::{Browser, LaunchOptions};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;
use tracing::debug;

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36";
const ACCEPT_LANGUAGE: &str = "en-US,en;q=0.9,hi;q=0.8,es;q=0.7,lt;q=0.6";
const PLATFORM: &str = "macOS";
const IMAGE_SERVER_PREFIX: &str = "https://img33.imgslib.link";
pub const MANGALIB_DEFAULT_BASE_URL: &str = "https://api.mangalib.me";

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

fn get_browser() -> Result<Browser, Error> {
    let options = LaunchOptions::default_builder()
        .sandbox(false)
        .build()
        .map_err(|err| Error::BrowserCreateBuilder(err.to_string()))?;

    Browser::new(options).map_err(|err| Error::BrowserCreate(err.to_string()))
}

pub fn get_manga_chapter_images(
    slug: &str,
    manga_chapter: &MangaChapter,
) -> Result<Vec<String>, Error> {
    let parser = Parser::new(
        format!(
            "https://api.mangalib.me/api/manga/{slug}/chapter?number={}&volume={}",
            manga_chapter.chapter_number, manga_chapter.chapter_volume
        ),
        "Searching manga chapter urls".to_owned(),
    );

    let image_inner_list = parser.parse::<ImageInnerList>()?;
    let images = image_inner_list
        .data
        .pages
        .into_iter()
        .map(|item| format!("{IMAGE_SERVER_PREFIX}{}", item.url))
        .collect();

    Ok(images)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MangaPreview {
    manga_type: String,
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

pub struct Mangalib;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChapterInnerList {
    data: Vec<ChapterInner>,
}

struct Parser {
    url: String,
    debug_message_prefix: String,
}

impl Parser {
    const fn new(url: String, debug_message_prefix: String) -> Self {
        Self {
            url,
            debug_message_prefix,
        }
    }

    fn parse<T>(&self) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let web_url = &self.url;
        let browser = get_browser()?;
        let tab = browser
            .new_tab()
            .map_err(|err| Error::BrowserTabCreate(err.to_string()))?;
        debug!("{} {web_url}", &self.debug_message_prefix);

        tab.set_user_agent(USER_AGENT, Some(ACCEPT_LANGUAGE), Some(PLATFORM))
            .map_err(|err| Error::SetUserAgent(err.to_string()))?;
        tab.navigate_to(web_url)
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
}

pub fn get_manga_chapters(slug: &str) -> Result<Vec<MangaChapter>, Error> {
    let parser = Parser::new(
        format!("https://api.mangalib.me/api/manga/{slug}/chapters"),
        slug.to_owned(),
    );

    let chapter_inner_list = parser.parse::<ChapterInnerList>()?;
    let chapters = chapter_inner_list
        .data
        .into_iter()
        .map(|chapter_inner| MangaChapter::new(chapter_inner.volume, chapter_inner.number))
        .collect();

    Ok(chapters)
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
