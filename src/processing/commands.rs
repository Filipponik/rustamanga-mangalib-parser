use crate::processing::{
    get_manga_with_chapters_and_images::GetMangaWithChaptersAndImagesParams,
    get_manga_with_only_chapters::GetMangaWithOnlyChaptersParams, get_user_list::GetUserListParams,
};
use serde::Deserialize;
use serde_json::{Map, Value};

macro_rules! define_commands {
    ($($str:expr => $variant:path),*) => {
        macro_rules! command_match {
            ($command_name:expr, $object_payload:expr) => {
                match $command_name {
                    $(
                        $str => Ok($variant(
                            parse_params_from_object($object_payload)?,
                        )),
                    )*
                    c_name => Err(ParseError::InvalidCommand(c_name.to_string())),
                }
            };
        }
    };
}

define_commands! {
    "full" => Command::GetMangaWithChaptersAndImages,
    "only_chapters" => Command::GetMangaWithOnlyChapters,
    "get_user_list" => Command::GetMangaWithChaptersAndImages
}

#[derive(Deserialize, Debug)]
pub enum Command {
    GetMangaWithChaptersAndImages(GetMangaWithChaptersAndImagesParams),
    GetMangaWithOnlyChapters(GetMangaWithOnlyChaptersParams),
    GetUserList(GetUserListParams),
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

    command_match!(command_name.as_str(), &object_payload)
}

/// Parse params from object payload
///
/// # Errors
/// - [`ParseError::ParamsMustBeSet`] If params is not set (not existing key)
/// - [`ParseError::InvalidParams`] If params cannot be deserialized to needed struct
fn parse_params_from_object<T: serde::de::DeserializeOwned>(
    object: &Map<String, Value>,
) -> Result<T, ParseError> {
    let Some(params) = object.get("params") else {
        return Err(ParseError::ParamsMustBeSet);
    };

    serde_json::from_value(params.clone()).map_err(ParseError::InvalidParams)
}
