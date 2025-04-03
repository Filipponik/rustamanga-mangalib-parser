use crate::mangalib::MangaPreview;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, info};

mod response {
    use crate::mangalib::MangaPreview;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct Response {
        data: Vec<Manga>,
        meta: Meta,
    }

    #[derive(Debug, Deserialize)]
    struct Manga {
        id: u32,
        name: String,
        rus_name: String,
        eng_name: String,
        slug: String,
        slug_url: String,
        model: String,
        site: u32,
        cover: Images,
        #[serde(rename = "ageRestriction")]
        age_restriction: AgeRestriction,
        r#type: Type,
        rating: Rating,
        status: Status,
    }

    #[derive(Debug, Deserialize)]
    struct Images {
        filename: String,
        thumbnail: String,
        default: String,
        md: String,
    }

    #[derive(Debug, Deserialize)]
    struct Type {
        id: u32,
        label: String,
    }

    #[derive(Debug, Deserialize)]
    struct AgeRestriction {
        id: u32,
        label: String,
    }

    #[derive(Debug, Deserialize)]
    struct Status {
        id: u32,
        label: String,
    }

    #[derive(Debug, Deserialize)]
    struct Rating {
        average: String,
        #[serde(rename = "averageFormated")]
        average_formatted: String,
        votes: u32,
        #[serde(rename = "votesFormated")]
        votes_formatted: String,
    }

    #[derive(Debug, Deserialize)]
    struct Meta {
        current_page: u32,
        from: Option<u32>,
        path: String,
        per_page: u32,
        to: Option<u32>,
        page: u32,
        has_next_page: bool,
        seed: String,
    }

    impl Into<MangaPreview> for Manga {
        fn into(self) -> MangaPreview {
            MangaPreview {
                manga_type: self.r#type.label,
                name: self.rus_name,
                url: format!("https://mangalib.me/{}", self.slug_url),
                slug: self.slug,
                image_url: self.cover.default,
            }
        }
    }

    impl Into<Vec<MangaPreview>> for Response {
        fn into(self) -> Vec<MangaPreview> {
            self.data.into_iter().map(|m| m.into()).collect()
        }
    }
}

#[derive(Debug, Serialize)]
struct Query {
    fields: Vec<String>,
    site_ids: Vec<u32>,
    page: u32,
}

impl Query {
    fn to_reqwest_format(&self) -> Vec<(String, String)> {
        let mut formatted = vec![];
        for field in &self.fields {
            formatted.push(("fields[]".to_string(), field.to_string()));
        }
        for site_id in &self.site_ids {
            formatted.push(("site_id[]".to_string(), site_id.to_string()));
        }

        formatted.push(("page".to_string(), self.page.to_string()));

        formatted
    }

    fn new_only_page(page: u32) -> Self {
        Self {
            fields: vec!["rate".to_string(), "rate_avg".to_string(), "userBookmark".to_string()],
            site_ids: vec![1],
            page,
        }
    }
}

#[derive(Debug, Error)]
pub enum SendingError {
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Deserialize error: {0}")]
    Deserialize(reqwest::Error),
    #[error("Runtime create error: {0}")]
    RuntimeCreate(#[from] std::io::Error),
}

async fn send(client: &reqwest::Client, query: &Query) -> Result<Vec<MangaPreview>, SendingError> {
    debug!("Requesting {} page", query.page);

    let response = client.get("https://api.lib.social/api/manga")
        .query(&query.to_reqwest_format().as_slice())
        .send()
        .await;

    let response = match response {
        Ok(response) => {
            info!("Success requesting manga at page {}", query.page);
            debug!("Response: {response:?}");

            Ok(response)
        }
        Err(err) => {
            error!("Error while requesting manga at page {}", query.page);

            Err(err)
        }
    }?;

    debug!("Parsing page {}", query.page);
    let response = response
        .json::<response::Response>()
        .await;

    match response {
        Ok(value) => {
            info!("Success parsing manga at page {}", query.page);

            Ok(value.into())
        }
        Err(err) => {
            error!("Error while parsing manga at page {}", query.page);

            Err(SendingError::Deserialize(err))
        }
    }
}

fn send_sync(client: &reqwest::Client, query: &Query) -> Result<Vec<MangaPreview>, SendingError> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        send(&client, &query).await
    })
}

pub fn get_manga_iter() -> GetAllMangaIterator {
    GetAllMangaIterator::new(None)
}

pub struct GetAllMangaIterator {
    current_page: u32,
    current_vec: Vec<MangaPreview>,
    current_index: usize,
    client: reqwest::Client,
}

impl GetAllMangaIterator {
    fn new(client: Option<reqwest::Client>) -> Self {
        Self {
            current_page: 0,
            current_vec: vec![],
            current_index: 0,
            client: client.unwrap_or_else(|| reqwest::Client::new()),
        }
    }
}

impl Iterator for GetAllMangaIterator {
    type Item = MangaPreview;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_vec.get(self.current_index).is_none() {
            self.current_page += 1;
            let request_result = send_sync(&self.client, &Query::new_only_page(self.current_page));
            self.current_vec = if let Ok(r) = request_result { r } else { return None };
            self.current_index = 0;
        }

        let res = self.current_vec.get(self.current_index).cloned();
        self.current_index += 1;

        res
    }
}