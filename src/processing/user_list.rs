#![allow(unused, dead_code, clippy::unused_async, clippy::missing_errors_doc)]

#[derive(serde::Deserialize, Debug)]
pub struct GetUserListParams {
    pub callback_url: String,
    pub user_id: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

/// Extension point for the `get_user_list` command, the feature itself is not implemented yet.
///
/// # Errors
/// Never returns an error while the feature is not implemented.
///
/// # Panics
/// Always panics with `unimplemented!` until the feature is implemented.
pub async fn handle(params: &GetUserListParams) -> Result<(), Error> {
    unimplemented!("get_user_list command is not implemented yet");
}
