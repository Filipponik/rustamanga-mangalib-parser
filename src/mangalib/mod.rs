#![allow(dead_code)]
#![allow(unused_variables)]
use headless_chrome::Browser;
use serde::{Deserialize, Deserializer, Serialize};
use tracing::debug;

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36";
const ACCEPT_LANGUAGE: &str = "en-US,en;q=0.9,hi;q=0.8,es;q=0.7,lt;q=0.6";
const PLATFORM: &str = "macOS";
const IMAGE_SERVER_PREFIX: &str = "https://img33.imgslib.link";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MangalibError {
    SerdeParseError,
    BrowserCreateError,
    BrowserTabCreateError,
    BrowserNavigateError,
    SetUserAgentError,
    BrowserWaitNavigateTooLong,
    BrowserWaitElementTooLong,
    BrowserFunctionError,
    BrowserGetContentError,
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
    ratio: String,
}

pub async fn get_manga_chapter_images(
    slug: &str,
    manga_chapter: &MangaChapter,
) -> Result<Vec<String>, MangalibError> {
    let browser = Browser::default().map_err(|_| MangalibError::BrowserCreateError)?;
    let tab = browser
        .new_tab()
        .map_err(|_| MangalibError::BrowserTabCreateError)?;
    let web_url = format!(
        "https://api.mangalib.me/api/manga/{slug}/chapter?number={}&volume={}",
        manga_chapter.chapter_number, manga_chapter.chapter_volume,
    );

    debug!("Searching manga chapters at {web_url}");
    tab.set_user_agent(USER_AGENT, Some(ACCEPT_LANGUAGE), Some(PLATFORM))
        .map_err(|_| MangalibError::SetUserAgentError)?;
    tab.navigate_to(&web_url)
        .map_err(|_| MangalibError::BrowserNavigateError)?
        .wait_until_navigated()
        .map_err(|_| MangalibError::BrowserWaitNavigateTooLong)?;

    let text = tab
        .wait_for_element("body > pre")
        .map_err(|_| MangalibError::BrowserWaitElementTooLong)?
        .get_inner_text()
        .map_err(|_| MangalibError::BrowserGetContentError)?;

    let chapter_inner_list: ImageInnerList =
        serde_json::from_str(&text).map_err(|_| MangalibError::SerdeParseError)?;

    let images = chapter_inner_list
        .data
        .pages
        .iter()
        .fold(Vec::new(), |mut acc, item| {
            let mut absolute_url = IMAGE_SERVER_PREFIX.to_string();
            absolute_url.push_str(&item.url.clone());
            acc.push(absolute_url);

            acc
        });

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
        MangaChapter {
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
    number_secondary: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChapterInnerList {
    data: Vec<ChapterInner>,
}

pub async fn get_manga_chapters(slug: &str) -> Result<Vec<MangaChapter>, MangalibError> {
    let web_url = &format!("https://api.mangalib.me/api/manga/{slug}/chapters");
    let browser = Browser::default().map_err(|_| MangalibError::BrowserCreateError)?;
    let tab = browser
        .new_tab()
        .map_err(|_| MangalibError::BrowserTabCreateError)?;
    debug!("Searching manga chapters at {web_url}");

    tab.set_user_agent(USER_AGENT, Some(ACCEPT_LANGUAGE), Some(PLATFORM))
        .map_err(|_| MangalibError::SetUserAgentError)?;
    tab.navigate_to(web_url)
        .map_err(|_| MangalibError::BrowserNavigateError)?
        .wait_until_navigated()
        .map_err(|_| MangalibError::BrowserWaitNavigateTooLong)?;

    let text = tab
        .wait_for_element("body > pre")
        .map_err(|_| MangalibError::BrowserWaitElementTooLong)?
        .get_inner_text()
        .map_err(|_| MangalibError::BrowserGetContentError)?;

    let chapter_inner_list: ChapterInnerList =
        serde_json::from_str(&text).map_err(|_| MangalibError::SerdeParseError)?;

    let chapters = chapter_inner_list
        .data
        .iter()
        .fold(Vec::new(), |mut acc, chapter_inner| {
            let chapter = MangaChapter {
                chapter_volume: chapter_inner.volume.to_string(),
                chapter_number: chapter_inner.number.clone(),
            };

            acc.push(chapter);
            acc
        });

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
