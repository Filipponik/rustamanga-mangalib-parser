use crate::telegraph::types::NodeElement;

mod account;
mod page;

pub enum FieldToChange {
    ShortName,
    AuthorName,
    AuthorUrl,
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

pub async fn create_page(
    access_token: &str,
    title: &str,
    author_name: Option<&str>,
    author_url: Option<&str>,
    content: &[NodeElement],
    return_content: bool,
) {
    page::create(
        access_token,
        title,
        author_name,
        author_url,
        content,
        return_content,
    )
    .await
}

pub async fn edit_page(
    access_token: &str,
    path: &str,
    title: &str,
    author_name: Option<&str>,
    author_url: Option<&str>,
    content: &[NodeElement],
    return_content: bool,
) {
    page::edit(
        access_token,
        path,
        title,
        author_name,
        author_url,
        content,
        return_content,
    )
    .await
}

pub async fn get_page(path: &str, return_content: bool) {
    page::get(path, return_content).await
}

pub async fn get_page_list(access_token: &str, offset: u64, limit: u8) {
    page::get_list(access_token, offset, limit).await
}

pub async fn get_views(
    path: &str,
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>,
    hour: Option<u8>,
) {
    page::get_views(path, year, month, day, hour).await
}
