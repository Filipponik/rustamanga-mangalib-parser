use crate::mangalib::{Client, Error, MangaChapter};
use serde::Deserialize;
use tracing::debug;

const IMAGE_SERVER_PREFIX: &str = "https://img33.imgslib.link";
const MANGALIB_DEFAULT_BASE_URL: &str = "https://api.cdnlibs.org";
const REFERRER_HEADER: &str = "https://mangalib.org/";
const SITE_ID_HEADER: &str = "1";

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

#[derive(Default, Debug)]
pub struct Builder {
    image_server_prefix: Option<String>,
    base_url: Option<String>,
    referrer_header: Option<String>,
    site_id_header: Option<String>,
    reqwest_client: Option<reqwest::Client>,
}

impl Builder {
    pub fn image_server_prefix(mut self, image_server_prefix: impl Into<String>) -> Self {
        self.image_server_prefix = Some(image_server_prefix.into());
        self
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn referrer_header(mut self, referrer_header: impl Into<String>) -> Self {
        self.referrer_header = Some(referrer_header.into());
        self
    }

    pub fn site_id_header(mut self, site_id_header: impl Into<String>) -> Self {
        self.site_id_header = Some(site_id_header.into());
        self
    }

    pub fn reqwest_client(mut self, reqwest_client: reqwest::Client) -> Self {
        self.reqwest_client = Some(reqwest_client);
        self
    }

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
            reqwest_client: self.reqwest_client.unwrap_or_default(),
        }
    }
}

pub struct HttpClient {
    image_server_prefix: String,
    base_url: String,
    referrer_header: String,
    site_id_header: String,
    reqwest_client: reqwest::Client,
}

impl HttpClient {
    pub fn builder() -> Builder {
        Builder::default()
    }

    async fn get<T>(&self, url: &str) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self
            .reqwest_client
            .get(url)
            .header("Referrer", &self.referrer_header)
            .header("Site-Id", &self.site_id_header)
            .send()
            .await
            .map_err(Error::ReqwestNetwork)?
            .text()
            .await
            .map_err(Error::ReqwestResponseRead)?;

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

        let image_inner_list: ImageInnerList = self.get(url).await?;
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
        let chapter_inner_list: ChapterInnerList = self.get(url).await?;

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
