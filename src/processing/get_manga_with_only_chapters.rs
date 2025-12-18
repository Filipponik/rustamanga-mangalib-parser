#[derive(serde::Deserialize, Debug)]
pub struct GetMangaWithOnlyChaptersParams {
    pub slug: String,
    pub callback_url: String,
}

pub fn handle(params: &GetMangaWithOnlyChaptersParams) {
    todo!();
}
