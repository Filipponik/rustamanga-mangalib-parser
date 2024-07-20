#![allow(dead_code)]
#![allow(unused_variables)]

use crate::telegraph::methods::{Error, ErrorResult, PageResult};
use crate::telegraph::types::NodeElement;
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

pub async fn create(
    access_token: &str,
    title: &str,
    author_name: Option<&str>,
    author_url: Option<&str>,
    content: &[NodeElement],
) -> Result<PageResult, Error> {
    let json_content: Vec<Value> = content.iter().map(|x| x.to_json()).collect();
    let client = reqwest::Client::new();
    let response: Value = client
        .post("https://api.telegra.ph/createPage")
        .json(&json!({
            "access_token": access_token,
            "author_name": author_name,
            // "title": title,
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

    match is_ok(&response)? {
        true => {
            let result: PageResult =
                serde_json::from_value(response).map_err(|x| Error::StructParseError)?;

            Ok(result)
        }
        false => {
            let result: ErrorResult =
                serde_json::from_value(response).map_err(|x| Error::StructParseError)?;

            Err(Error::BadResponse(result))
        }
    }
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
