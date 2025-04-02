use serde::{Deserialize, Serialize};
use crate::mangalib::MangaPreview;

mod response {
    use serde::Deserialize;
    use crate::mangalib::MangaPreview;

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

        formatted
    }
}

pub async fn get() -> Vec<MangaPreview> {
    let client = reqwest::Client::new();
    let query = Query {
        fields: vec!["rate".to_string(), "rate_avg".to_string(), "userBookmark".to_string()],
        site_ids: vec![1],
    };

    let res_text = client.get("https://api.lib.social/api/manga")
        .query(&query.to_reqwest_format().as_slice())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    serde_json::from_str::<response::Response>(&res_text).unwrap().into()
}

