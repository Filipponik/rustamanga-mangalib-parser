#![allow(dead_code)]
#![allow(unused_variables)]
use headless_chrome::Browser;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::error::Error;

const URL: &str = "https://mangalib.me";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36";
const ACCEPT_LANGUAGE: &str = "en-US,en;q=0.9,hi;q=0.8,es;q=0.7,lt;q=0.6";
const PLATFORM: &str = "macOS";

fn get_url() -> String {
    URL.to_string()
}

pub async fn get_manga_chapter_images(
    slug: &str,
    manga_chapter: &MangaChapter,
) -> Result<Vec<String>, Box<dyn Error>> {
    let browser = Browser::default().unwrap();
    let tab = browser.new_tab().unwrap();
    let web_url = format!(
        "{}/{slug}/v{}/c{}?page=1",
        get_url(),
        manga_chapter.chapter_volume,
        manga_chapter.chapter_number
    );
    println!("going to {web_url}");
    tab.set_user_agent(USER_AGENT, Some(ACCEPT_LANGUAGE), Some(PLATFORM))
        .unwrap();
    tab.navigate_to(&web_url)
        .unwrap()
        .wait_until_navigated()
        .unwrap();
    let reader_element = tab.wait_for_element(".reader-view").unwrap();
    let js_obj = reader_element.call_js_fn(r#"
        function f() {
            return JSON.stringify(window.__pg.map(el => el.u).map(image => window.__info.servers[window.__info.img.server]+window.__info.img.url+image));
        }
    "#, vec![], false)?;

    let res: Vec<String> = match js_obj.value.unwrap() {
        Value::String(v) => serde_json::from_str(&v).unwrap(),
        _ => panic!("shit happens!"),
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

pub async fn search_manga(search_input: &str) -> Result<Vec<MangaPreview>, Box<dyn Error>> {
    let web_url = &format!(
        "{}/manga-list?sort=rate&dir=desc&page=1&name={search_input}",
        get_url()
    );
    let browser = Browser::default().unwrap();
    let tab = browser.new_tab().unwrap();
    println!("going to {web_url}");
    tab.set_user_agent(USER_AGENT, Some(ACCEPT_LANGUAGE), Some(PLATFORM))
        .unwrap();
    tab.navigate_to(web_url)
        .unwrap()
        .wait_until_navigated()
        .unwrap();

    let start = std::time::Instant::now();
    tab.wait_for_element(".media-card").unwrap();
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
    )?;

    Ok(match func_result.value.unwrap() {
        Value::String(v) => serde_json::from_str(&v).unwrap(),
        _ => panic!("shit happens!"),
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

pub async fn get_manga_chapters(slug: &str) -> Result<Vec<MangaChapter>, Box<dyn Error>> {
    let web_url = &format!("{}/{slug}?section=chapters", get_url());
    let browser = Browser::default().unwrap();
    let tab = browser.new_tab().unwrap();
    println!("going to {web_url}");
    tab.set_user_agent(USER_AGENT, Some(ACCEPT_LANGUAGE), Some(PLATFORM))
        .unwrap();
    tab.navigate_to(web_url)
        .unwrap()
        .wait_until_navigated()
        .unwrap();
    let elem = tab.wait_for_element(".media-chapter__name.text-truncate a");
    let elem = match elem {
        Ok(value) => value,
        Err(err) => {
            panic!("{}\n{}", err, tab.get_content().unwrap())
        }
    };
    let js_obj = elem.call_js_fn(
        r#"
        function f() {
            return JSON.stringify(window.__DATA__.chapters.list);
        }
    "#,
        vec![],
        false,
    )?;

    Ok(match js_obj.value.unwrap() {
        Value::String(v) => serde_json::from_str(&v).unwrap(),
        _ => panic!("shit happens!"),
    })
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
