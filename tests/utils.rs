use std::fs;

use rustamanga_mangalib_parser::mangalib::http_client::{AccessToken, RefreshToken, TokenPair};
use serde_json::Value;

/// Loads JSON fixture by name.
///
/// # Errors
/// - [`FixtureError::ReadError`] if the fixture cannot be read.
/// - [`FixtureError::ParseError`] if the fixture cannot be parsed.
#[allow(clippy::panic)]
pub fn load_fixture(fixture_name: &str) -> Result<Value, FixtureError> {
    let path = format!("tests/fixtures/{fixture_name}");
    let content = fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&content)?)
}

#[derive(Debug, thiserror::Error)]
pub enum FixtureError {
    #[error("Failed to read fixture {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse JSON fixture {0}")]
    ParseError(#[from] serde_json::Error),
}

#[must_use]
pub fn create_token_pair() -> TokenPair {
    TokenPair {
        access_token: AccessToken::new("test_access_token"),
        refresh_token: RefreshToken::new("test_refresh_token"),
    }
}
