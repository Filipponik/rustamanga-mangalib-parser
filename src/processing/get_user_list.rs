#[derive(serde::Deserialize, Debug)]
pub struct GetUserListParams {
    pub callback_url: String,
    pub user_id: u32,
}

pub fn handle(params: &GetUserListParams) {
    todo!();
}
