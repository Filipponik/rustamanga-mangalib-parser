#![allow(dead_code)]
#![allow(unused_variables)]
use crate::telegraph::types::NodeElement;

pub async fn create(
    access_token: &str,
    title: &str,
    author_name: Option<&str>,
    author_url: Option<&str>,
    content: &[NodeElement],
    return_content: bool,
) {
    todo!()
}

pub async fn edit(
    access_token: &str,
    path: &str,
    title: &str,
    author_name: Option<&str>,
    author_url: Option<&str>,
    content: &[NodeElement],
    return_content: bool,
) {
    todo!()
}

pub async fn get(path: &str, return_content: bool) {
    todo!()
}

pub async fn get_list(access_token: &str, offset: u64, limit: u8) {
    todo!()
}

pub async fn get_views(
    path: &str,
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>,
    hour: Option<u8>,
) {
    todo!()
}
