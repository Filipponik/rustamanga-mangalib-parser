#![allow(unused, dead_code, clippy::unused_async)]

use crate::mangalib;
use std::sync::Arc;

#[derive(serde::Deserialize, Debug)]
pub struct GetUserListParams {
    pub callback_url: String,
    pub user_id: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

pub async fn handle<T: mangalib::Client>(
    params: &GetUserListParams,
    client: Arc<T>,
    sender: &reqwest::Client,
) -> Result<(), Error> {
    todo!();
}
