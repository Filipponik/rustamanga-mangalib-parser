use crate::mangalib::MangaPreview;
use async_stream::stream;
use futures::Stream;
use governor::{DefaultKeyedRateLimiter, Quota};
use serde::Serialize;
use std::num::NonZeroU32;
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
        rus_name: Option<String>,
        eng_name: Option<String>,
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
        filename: Option<String>,
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
                name: self
                    .rus_name
                    .unwrap_or_else(|| self.eng_name.unwrap_or(self.name)),
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
            fields: vec![
                "rate".to_string(),
                "rate_avg".to_string(),
                "userBookmark".to_string(),
            ],
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

    let response = client
        .get("https://api.lib.social/api/manga")
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
    let response = response.json::<response::Response>().await;

    match response {
        Ok(value) => {
            info!("Success parsing manga at page {}", query.page);

            Ok(value.into())
        }
        Err(err) => {
            error!(
                "Error while parsing manga at page {}: {:?}",
                query.page, err
            );

            Err(SendingError::Deserialize(err))
        }
    }
}

pub fn get_manga_iter() -> impl Stream<Item = MangaPreview> {
    stream! {
        let quota = Quota::per_minute(NonZeroU32::new(30).unwrap());
        let rate_limiter = DefaultKeyedRateLimiter::keyed(quota);
        let client = reqwest::Client::new();

        let mut page_num = 1;
        loop {
            rate_limiter.until_key_ready(&"default").await;
            if let Ok(page) = send(&client, &Query::new_only_page(page_num)).await {
                if page.is_empty() {
                    break;
                }

                for manga in page {
                    yield manga;
                }

                page_num += 1;
            } else {
                break;
            }
        }
    }
}
