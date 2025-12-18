#[derive(serde::Deserialize, Debug)]
pub struct GetUserListParams {
    pub callback_url: String,
    pub user_id: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

pub async fn handle(params: &GetUserListParams) -> Result<(), Error> {
    todo!();
}
