#![allow(dead_code)]
#![allow(unused)]
use crate::{mangalib, processing::Error};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::sync::Arc;

#[derive(Deserialize, Debug, Default)]
pub enum CommandName {
    #[serde(rename = "full")]
    #[default]
    GetMangaWithChaptersAndImages,
    #[serde(rename = "only_chapters")]
    GetMangaWithOnlyChapters,
}

#[derive(Deserialize, Debug)]
pub enum Command {
    GetMangaWithChaptersAndImages(GetMangaWithChaptersAndImagesParams),
    GetMangaWithOnlyChapters(GetMangaWithOnlyChaptersParams),
}

impl Command {
    pub fn handle(&self) -> Result<(), Error> {
        todo!()
    }
}

#[derive(Deserialize, Debug)]
pub struct GetMangaWithOnlyChaptersParams {
    slug: String,
    callback_url: String,
}

#[derive(Deserialize, Debug)]
pub struct GetMangaWithChaptersAndImagesParams {
    slug: String,
    callback_url: String,
    after_chapter: Option<String>,
    after_volume: Option<String>,
}

#[derive(Clone)]
pub struct Processor<TClient: mangalib::Client> {
    client: Arc<TClient>,
    sender: reqwest::Client,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid command: {0}")]
    FirstParse(serde_json::Error),
    #[error("Payload must be an object")]
    PayloadMustBeObject,
    #[error("Command must be a string")]
    CommandMustBeString,
    #[error("Params must be set")]
    ParamsMustBeSet,
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("Invalid params: {0}")]
    InvalidParams(serde_json::Error),
}

impl<TClient: mangalib::Client> Processor<TClient> {
    pub const fn new(client: Arc<TClient>, sender: reqwest::Client) -> Self {
        Self { client, sender }
    }

    /// Parse command from string payload
    ///
    /// # Errors
    /// - [`ParseError::FirstParse`]
    /// - [`ParseError::PayloadMustBeObject`]
    /// - [`ParseError::CommandMustBeString`]
    /// - [`ParseError::ParamsMustBeSet`]
    /// - [`ParseError::InvalidCommand`]
    /// - [`ParseError::InvalidParams`]
    pub fn parse_command(command: &str) -> Result<Command, ParseError> {
        let value: Value = serde_json::from_str(command).map_err(ParseError::FirstParse)?;

        let Value::Object(object_payload) = value else {
            return Err(ParseError::PayloadMustBeObject);
        };

        let Some(Value::String(command_name)) = object_payload.get("command") else {
            return Err(ParseError::CommandMustBeString);
        };

        match command_name.as_str() {
            "full" => Ok(Command::GetMangaWithChaptersAndImages(
                Self::parse_params_from_object(&object_payload)?,
            )),
            "only_chapters" => Ok(Command::GetMangaWithOnlyChapters(
                Self::parse_params_from_object(&object_payload)?,
            )),
            c_name => Err(ParseError::InvalidCommand(c_name.to_string())),
        }
    }

    fn parse_params_from_object<T: serde::de::DeserializeOwned>(
        object: &Map<String, Value>,
    ) -> Result<T, ParseError> {
        let Some(params) = object.get("params") else {
            return Err(ParseError::ParamsMustBeSet);
        };

        serde_json::from_value(params.clone()).map_err(ParseError::InvalidParams)
    }

    #[allow(clippy::missing_errors_doc)]
    #[allow(clippy::unused_async)]
    pub async fn process(&self, command: Command) -> Result<(), Error> {
        command.handle();

        todo!();
    }
}
