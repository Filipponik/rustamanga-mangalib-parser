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
pub struct Command<T> {
    command: CommandName,
    params: T,
}

pub trait CommandHandler {
    fn handle(&self);
}

pub struct GetMangaWithOnlyChaptersParams {
    slug: String,
    callback_url: String,
}

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
    #[error("Params must be an object")]
    ParamsMustBeObject,
    #[error("Invalid command: {0}")]
    InvalidCommand(serde_json::Error),
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
    /// - [`ParseError::ParamsMustBeObject`]
    /// - [`ParseError::InvalidCommand`]
    pub fn parse_command<T>(command: &str) -> Result<Command<T>, ParseError> {
        let value: Value = serde_json::from_str(command).map_err(ParseError::FirstParse)?;

        let Value::Object(object_payload) = value else {
            return Err(ParseError::PayloadMustBeObject);
        };

        let Some(Value::String(command_name)) = object_payload.get("command") else {
            return Err(ParseError::CommandMustBeString);
        };

        let command_enum: CommandName =
            serde_json::from_str(command_name).map_err(ParseError::InvalidCommand)?;

        let Some(Value::Object(params)) = object_payload.get("params") else {
            return Err(ParseError::ParamsMustBeObject);
        };

        todo!();
    }

    #[allow(clippy::missing_errors_doc)]
    #[allow(clippy::unused_async)]
    pub async fn process<T: CommandHandler>(&self, command: Command<T>) -> Result<(), Error> {
        command.params.handle();

        todo!();
    }
}
