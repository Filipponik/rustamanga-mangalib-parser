#![allow(dead_code)]

use utoipa::ToSchema;

use crate::processing::manga::{
    GetMangaWithChaptersAndImagesParams, GetMangaWithOnlyChaptersParams,
};
use crate::processing::user_list::GetUserListParams;

/// Request payload for `/async-command` and `/sync-command` endpoints.
///
/// Mirrors the wire format `{"command": "<name>", "params": {...}}`, where
/// the set of `params` fields depends on the chosen command.
#[derive(ToSchema)]
#[serde(tag = "command", content = "params", rename_all = "snake_case")]
#[schema(
    example = json!({
        "command": "full",
        "params": {
            "slug": "solo-leveling",
            "callback_url": "https://example.com/callback",
            "after_chapter": "1",
            "after_volume": "1"
        }
    })
)]
pub enum CommandRequest {
    /// Scrape manga with chapters and images.
    Full(GetMangaWithChaptersAndImagesParams),
    /// Scrape manga chapters without images.
    OnlyChapters(GetMangaWithOnlyChaptersParams),
    /// Get the user list.
    GetUserList(GetUserListParams),
}

/// Response returned when a command was accepted (or processed) successfully.
#[derive(ToSchema)]
pub struct CommandAcceptedResponse {
    pub success: bool,
    pub message: String,
}

/// Error response returned with 4xx and 5xx status codes.
#[derive(ToSchema)]
pub struct ErrorResponse {
    pub success: bool,
    pub code: String,
    pub message: String,
}

/// Health check response.
#[derive(ToSchema)]
pub struct HealthResponse {
    pub status: String,
}

/// Version response.
#[derive(ToSchema)]
pub struct VersionResponse {
    pub version: String,
}
