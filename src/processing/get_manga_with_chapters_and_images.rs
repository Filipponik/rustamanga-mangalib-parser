#[derive(serde::Deserialize, Debug)]
pub struct GetMangaWithChaptersAndImagesParams {
    pub slug: String,
    pub callback_url: String,
    pub after_chapter: Option<String>,
    pub after_volume: Option<String>,
}

pub fn handle(params: &GetMangaWithChaptersAndImagesParams) {
    todo!();
}
