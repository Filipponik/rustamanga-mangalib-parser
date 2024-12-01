#![allow(dead_code)]
#![allow(unused_variables)]
use headless_chrome::Browser;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use tracing::{debug, error};

const URL: &str = "https://mangalib.me";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36";
const ACCEPT_LANGUAGE: &str = "en-US,en;q=0.9,hi;q=0.8,es;q=0.7,lt;q=0.6";
const PLATFORM: &str = "macOS";

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

fn get_url() -> String {
    URL.to_string()
}

pub async fn get_manga_chapter_images(
    slug: &str,
    manga_chapter: &MangaChapter,
) -> Result<Vec<String>, MangalibError> {
    let browser = Browser::default().map_err(|_| MangalibError::BrowserCreateError)?;
    let tab = browser.new_tab().map_err(|_| MangalibError::BrowserTabCreateError)?;
    let web_url = format!(
        "{}/{slug}/v{}/c{}?page=1",
        get_url(),
        manga_chapter.chapter_volume,
        manga_chapter.chapter_number
    );

    debug!("Searching manga chapters at {web_url}");
    tab.set_user_agent(USER_AGENT, Some(ACCEPT_LANGUAGE), Some(PLATFORM)).map_err(|_| MangalibError::SetUserAgentError)?;
    tab.navigate_to(&web_url)
        .map_err(|_| MangalibError::BrowserNavigateError)?
        .wait_until_navigated()
        .map_err(|_| MangalibError::BrowserWaitNavigateTooLong)?;
    let reader_element = tab.wait_for_element(".reader-view")
        .map_err(|_| MangalibError::BrowserWaitElementTooLong)?;

    let js_obj = reader_element.call_js_fn(r#"
        function f() {
            return JSON.stringify(window.__pg.map(el => el.u).map(image => window.__info.servers[window.__info.img.server]+window.__info.img.url+image));
        }
    "#, vec![], false)
        .map_err(|_| MangalibError::BrowserFunctionError)?;

    let val = match js_obj.value {
        Some(v) => v,
        _ => return Err(MangalibError::BrowserFunctionError),
    };

    let res: Vec<String> = match val {
        Value::String(v) => serde_json::from_str(&v).map_err(|_| MangalibError::SerdeParseError)?,
        _ => return Err(MangalibError::BrowserFunctionError),
    };

    let _ = tab.close_target();

    Ok(res)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MangaPreview {
    manga_type: String,
    name: String,
    url: String,
    slug: String,
    image_url: String,
}

pub async fn search_manga(search_input: &str) -> Result<Vec<MangaPreview>, MangalibError> {
    let web_url = format!(
        "{}/manga-list?sort=rate&dir=desc&page=1&name={search_input}",
        get_url()
    );
    let browser = Browser::default().map_err(|_| MangalibError::BrowserCreateError)?;
    let tab = browser.new_tab().map_err(|_| MangalibError::BrowserTabCreateError)?;
    debug!("Searching manga at {web_url}");

    tab.set_user_agent(USER_AGENT, Some(ACCEPT_LANGUAGE), Some(PLATFORM)).map_err(|_| MangalibError::SetUserAgentError)?;
    tab.navigate_to(&web_url)
        .map_err(|_| MangalibError::BrowserNavigateError)?
        .wait_until_navigated()
        .map_err(|_| MangalibError::BrowserWaitNavigateTooLong)?;

    let start = std::time::Instant::now();

    tab.wait_for_element(".media-card")
        .map_err(|_| MangalibError::BrowserWaitElementTooLong)?;

    let func_result = tab.evaluate(
        r#"
        JSON.stringify(Array.from(document.querySelectorAll('.media-card')).map(function (el) {
            let subinfo = el.querySelector('.media-card__caption');
            return {
                manga_type: subinfo.querySelector('h5.media-card__subtitle').innerText,
                name: subinfo.querySelector('h3.media-card__title').innerText,
                url: el.href,
                slug: el.dataset.mediaSlug,
                image_url: el.dataset.src,
            }
        }))
    "#,
        false,
    )
        .map_err(|_| MangalibError::BrowserFunctionError)?;


    let val = match func_result.value {
        Some(v) => v,
        _ => return Err(MangalibError::BrowserFunctionError),
    };

    Ok(match val {
        Value::String(v) => serde_json::from_str(&v).map_err(|_| MangalibError::SerdeParseError)?,
        _ => return Err(MangalibError::BrowserFunctionError),
    })
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
    data: Vec<ChapterInner>
}

pub async fn get_manga_chapters(slug: &str) -> Result<Vec<MangaChapter>, MangalibError> {
    let web_url = &format!("https://api.mangalib.me/api/manga/{slug}/chapters");
    let browser = Browser::default().map_err(|_| MangalibError::BrowserCreateError)?;
    let tab = browser.new_tab().map_err(|_| MangalibError::BrowserTabCreateError)?;
    debug!("Searching manga chapters at {web_url}");

    tab.set_user_agent(USER_AGENT, Some(ACCEPT_LANGUAGE), Some(PLATFORM)).map_err(|_| MangalibError::SetUserAgentError)?;
    tab.navigate_to(&web_url)
        .map_err(|_| MangalibError::BrowserNavigateError)?
        .wait_until_navigated()
        .map_err(|_| MangalibError::BrowserWaitNavigateTooLong)?;

    let text = tab.wait_for_element("body > pre")
        .map_err(|_| MangalibError::BrowserWaitElementTooLong)?
        .get_inner_text()
        .map_err(|_| MangalibError::BrowserGetContentError)?;

    let chapter_inner_list: ChapterInnerList = serde_json::from_str(&text)
        .map_err(|_| MangalibError::SerdeParseError)?;

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
