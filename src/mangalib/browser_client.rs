use crate::mangalib::{Client, Error, MangaChapter};
use headless_chrome::{Browser, LaunchOptions};
use serde::Deserialize;
use tracing::debug;

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36";
const ACCEPT_LANGUAGE: &str = "en-US,en;q=0.9,hi;q=0.8,es;q=0.7,lt;q=0.6";
const PLATFORM: &str = "macOS";
const IMAGE_SERVER_PREFIX: &str = "https://img33.imgslib.link";
const MANGALIB_DEFAULT_BASE_URL: &str = "https://api.cdnlibs.org";

#[derive(Default, Debug)]
pub struct Builder {
    user_agent: Option<String>,
    accept_language: Option<String>,
    platform: Option<String>,
    image_server_prefix: Option<String>,
    base_url: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct ImageInnerList {
    data: ImageInnerListData,
}

#[derive(Deserialize, Debug, Clone)]
struct ImageInnerListData {
    pages: Vec<ImageInner>,
}

#[derive(Deserialize, Debug, Clone)]
struct ImageInner {
    id: u128,
    image: String,
    height: u32,
    width: u32,
    url: String,
    #[serde(deserialize_with = "crate::mangalib::deserializers::to_string")]
    ratio: String,
}

#[derive(Deserialize, Debug, Clone)]
struct ChapterInner {
    id: u128,
    index: u128,
    item_number: u128,
    volume: String,
    number: String,
    number_secondary: Option<String>,
    name: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct ChapterInnerList {
    data: Vec<ChapterInner>,
}

impl Builder {
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    pub fn accept_language(mut self, accept_language: impl Into<String>) -> Self {
        self.accept_language = Some(accept_language.into());
        self
    }

    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    pub fn image_server_prefix(mut self, image_server_prefix: impl Into<String>) -> Self {
        self.image_server_prefix = Some(image_server_prefix.into());
        self
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn build(self) -> HeadlessBrowserClient {
        HeadlessBrowserClient {
            user_agent: self.user_agent.unwrap_or_else(|| USER_AGENT.to_string()),
            accept_language: self
                .accept_language
                .unwrap_or_else(|| ACCEPT_LANGUAGE.to_string()),
            platform: self.platform.unwrap_or_else(|| PLATFORM.to_string()),
            image_server_prefix: self
                .image_server_prefix
                .unwrap_or_else(|| IMAGE_SERVER_PREFIX.to_string()),
            base_url: self
                .base_url
                .unwrap_or_else(|| MANGALIB_DEFAULT_BASE_URL.to_string()),
        }
    }
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

    fn parse<T>(&self, url: &str) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let browser = Self::get_browser()?;
        let tab = browser
            .new_tab()
            .map_err(|err| Error::BrowserTabCreate(err.to_string()))?;

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
        let image_inner_list: ImageInnerList = self.parse(url)?;

        let images = image_inner_list
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
        let chapter_inner_list: ChapterInnerList = self.parse(url)?;

        debug!(
            manga_slug = slug,
            "Found {} chapters",
            chapter_inner_list.data.len()
        );

        let chapters = chapter_inner_list
            .data
            .into_iter()
            .map(|chapter_inner| MangaChapter::new(chapter_inner.volume, chapter_inner.number))
            .collect();

        Ok(chapters)
    }
}
