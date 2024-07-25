use std::fmt::Display;
use crate::telegraph::types::NodeElement;
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod account;
mod page;

#[derive(Serialize, Deserialize, Debug)]
pub struct ListPagesResult {
    pub total_count: u128,
    pub pages: Vec<Page>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Page {
    pub path: String,
    pub url: String,
    pub title: String,
    pub description: String,
    pub content: Option<Value>,
    pub views: u128,
    pub can_edit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FieldToChange {
    ShortName,
    AuthorName,
    AuthorUrl,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResult {
    pub error: String,
}

impl ErrorResult {
    pub fn new(error: String) -> Self {
        ErrorResult { error }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    RequestInternalError,
    JsonParseError,
    StructParseError,
    BadResponse(ErrorResult),
}

pub async fn create_account(short_name: &str, author_name: Option<&str>, author_url: Option<&str>) {
    account::create(short_name, author_name, author_url).await
}

pub async fn edit_account_info(
    access_token: &str,
    fields_to_change: Vec<FieldToChange>,
    short_name: &str,
    author_name: Option<&str>,
    author_url: Option<&str>,
) {
    account::edit(
        access_token,
        fields_to_change,
        short_name,
        author_name,
        author_url,
    )
    .await
}

pub async fn get_account_info(access_token: &str, fields: Vec<FieldToChange>) {
    account::get(access_token, fields).await
}

pub async fn revoke_access_token(access_token: &str) {
    account::revoke_token(access_token).await
}

pub async fn create_page<T: Into<String> + Serialize>(
    access_token: T,
    title: T,
    author_name: Option<T>,
    author_url: Option<T>,
    content: &[NodeElement],
) -> Result<Page, Error> {
    page::create(access_token, title, author_name, author_url, content).await
}

pub async fn edit_page<T: Into<String> + Serialize>(
    access_token: T,
    path: T,
    title: Option<T>,
    author_name: Option<T>,
    author_url: Option<T>,
    content: &[NodeElement],
) {
    page::edit(
        access_token,
        path,
        title,
        author_name,
        author_url,
        content,
    )
    .await
}

pub async fn get_page<T: Into<String> + Display>(path: T) -> Result<Page, Error> {
    page::get(path).await
}

pub async fn get_page_list<T: Into<String> + Display>(access_token: T, offset: u64, limit: u8) -> Result<ListPagesResult, Error> {
    page::get_list(access_token, offset, limit).await
}

pub async fn get_views<T: Into<String> + Display>(
    path: T,
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>,
    hour: Option<u8>,
) -> Result<u128, Error> {
    page::get_views(path, year, month, day, hour).await
}
