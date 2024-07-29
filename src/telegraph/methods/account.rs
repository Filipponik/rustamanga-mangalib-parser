#![allow(dead_code)]
#![allow(unused_variables)]

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::telegraph::methods::{Error, ErrorResult, FieldToChange};
use crate::telegraph::methods::page::PageResult;

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub short_name: String,
    pub author_name: Option<String>,
    pub author_url: Option<String>,
    pub access_token: Option<String>,
    pub auth_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RevokeTokenResult {
    ok: bool,
    result: Access,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Access {
    pub access_token: String,
    pub auth_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AccountResult {
    ok: bool,
    result: Account,
}

pub fn is_ok(value: &Value) -> Result<bool, Error> {
    return match value {
        Value::Object(object) => match object.get("ok") {
            Some(Value::Bool(ok_value)) => Ok(*ok_value),
            _ => Err(Error::StructParseError),
        },
        _ => Err(Error::StructParseError),
    };
}

pub async fn create(short_name: &str, author_name: Option<&str>, author_url: Option<&str>) -> Result<Account, Error> {
    let client = reqwest::Client::new();
    let response: Value = client
        .post("https://api.telegra.ph/createAccount")
        .json(&json!({
            "short_name": short_name,
            "author_name": author_name,
            "author_url": author_url
        }))
        .send()
        .await
        .map_err(|err| Error::RequestInternalError)?
        .json()
        .await
        .map_err(|err| Error::JsonParseError)?;

    match is_ok(&response)? {
        true => Ok(serde_json::from_value::<AccountResult>(response).map_err(|x| Error::StructParseError)?.result),
        false => Err(Error::BadResponse(serde_json::from_value::<ErrorResult>(response)
            .map_err(|x| Error::StructParseError)?)),
    }
}

pub async fn edit(
    access_token: &str,
    fields_to_change: Vec<FieldToChange>,
    short_name: &str,
    author_name: Option<&str>,
    author_url: Option<&str>,
) {
    todo!();
}

pub async fn get(access_token: &str, fields: Vec<FieldToChange>) -> Result<Account, Error> {
    let response: Value = reqwest::get("https://api.telegra.ph/createAccount")
        .await
        .map_err(|err| Error::RequestInternalError)?
        .json()
        .await
        .map_err(|err| Error::JsonParseError)?;

    match is_ok(&response)? {
        true => Ok(serde_json::from_value::<AccountResult>(response).map_err(|x| Error::StructParseError)?.result),
        false => Err(Error::BadResponse(serde_json::from_value::<ErrorResult>(response)
            .map_err(|x| Error::StructParseError)?)),
    }
}

pub async fn revoke_token(access_token: &str) -> Result<Access, Error> {
    let client = reqwest::Client::new();
    let response: Value = client
        .post("https://api.telegra.ph/revokeAccessToken")
        .json(&json!({
            "access_token": access_token
        }))
        .send()
        .await
        .map_err(|err| Error::RequestInternalError)?
        .json()
        .await
        .map_err(|err| Error::JsonParseError)?;

    match is_ok(&response)? {
        true => Ok(serde_json::from_value::<RevokeTokenResult>(response).map_err(|x| Error::StructParseError)?.result),
        false => Err(Error::BadResponse(serde_json::from_value::<ErrorResult>(response)
            .map_err(|x| Error::StructParseError)?)),
    }
}
