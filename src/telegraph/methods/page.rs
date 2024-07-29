#![allow(dead_code)]
#![allow(unused_variables)]

use crate::telegraph::methods::{Error, ErrorResult, ListPagesResult, Page};
use crate::telegraph::types::NodeElement;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub fn is_ok(value: &Value) -> Result<bool, Error> {
    return match value {
        Value::Object(object) => match object.get("ok") {
            Some(Value::Bool(ok_value)) => Ok(*ok_value),
            _ => Err(Error::StructParseError),
        },
        _ => Err(Error::StructParseError),
    };
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PageResult {
    ok: bool,
    pub result: Page,
}

pub async fn create(
    access_token: &str,
    title: &str,
    author_name: Option<&str>,
    author_url: Option<&str>,
    content: &[NodeElement],
) -> Result<Page, Error> {
    let json_content: Vec<Value> = content.iter().map(|x| x.to_json()).collect();
    let client = reqwest::Client::new();
    let response: Value = client
        .post("https://api.telegra.ph/createPage")
        .json(&json!({
            "access_token": access_token,
            "author_name": author_name,
            "title": title,
            "author_url": author_url,
            "content": json_content,
            "return_content": true,
        }))
        .send()
        .await
        .map_err(|err| Error::RequestInternalError)?
        .json()
        .await
        .map_err(|err| Error::JsonParseError)?;

    response_to_page(response)
}

pub async fn edit(
    access_token: &str,
    path: &str,
    title: &str,
    author_name: Option<&str>,
    author_url: Option<&str>,
    content: &[NodeElement],
) -> Result<Page, Error> {
    let json_content: Vec<Value> = content.iter().map(|x| x.to_json()).collect();
    let client = reqwest::Client::new();
    let response: Value = client
        .post("https://api.telegra.ph/editPage")
        .json(&json!({
            "access_token": access_token,
            "path": path,
            "title": title,
            "content": json_content,
            "author_name": author_name,
            "author_url": author_url,
            "return_content": true,
        }))
        .send()
        .await
        .map_err(|err| Error::RequestInternalError)?
        .json()
        .await
        .map_err(|err| Error::JsonParseError)?;

    response_to_page(response)
}

fn response_to_page(response: Value) -> Result<Page, Error> {
    match is_ok(&response)? {
        true => Ok(serde_json::from_value::<PageResult>(response)
            .map_err(|x| Error::StructParseError)?
            .result),
        false => Err(Error::BadResponse(
            serde_json::from_value::<ErrorResult>(response).map_err(|x| Error::StructParseError)?,
        )),
    }
}

pub async fn get(path: &str) -> Result<Page, Error> {
    let response: Value = reqwest::get(format!("https://api.telegra.ph/getPage?path={path}"))
        .await
        .map_err(|err| Error::RequestInternalError)?
        .json()
        .await
        .map_err(|err| Error::JsonParseError)?;

    response_to_page(response)
}

#[derive(Serialize, Deserialize, Debug)]
struct ListResult {
    ok: bool,
    result: ListPagesResult,
}

pub async fn get_list(
    access_token: &str,
    offset: u64,
    limit: u8,
) -> Result<ListPagesResult, Error> {
    let response: Value = reqwest::get(format!("https://api.telegra.ph/getPageList?access_token={access_token}&offset={offset}&limit={limit}"))
        .await
        .map_err(|err| Error::RequestInternalError)?
        .json()
        .await
        .map_err(|err| Error::JsonParseError)?;

    match is_ok(&response)? {
        true => Ok(serde_json::from_value::<ListResult>(response)
            .map_err(|x| Error::StructParseError)?
            .result),
        false => Err(Error::BadResponse(
            serde_json::from_value::<ErrorResult>(response).map_err(|x| Error::StructParseError)?,
        )),
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Views {
    views: u128,
}

#[derive(Serialize, Deserialize, Debug)]
struct ViewsResult {
    ok: bool,
    result: Views,
}

pub async fn get_views(
    path: &str,
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>,
    hour: Option<u8>,
) -> Result<u128, Error> {
    let response: Value = reqwest::get(format!("https://api.telegra.ph/getViews?path={path}"))
        .await
        .map_err(|err| Error::RequestInternalError)?
        .json()
        .await
        .map_err(|err| Error::JsonParseError)?;

    match is_ok(&response)? {
        true => Ok(serde_json::from_value::<ViewsResult>(response)
            .map_err(|x| Error::StructParseError)?
            .result
            .views),
        false => Err(Error::BadResponse(
            serde_json::from_value::<ErrorResult>(response).map_err(|x| Error::StructParseError)?,
        )),
    }
}
