use std::fs;

use serde_json::Value;

/// Loads JSON fixture by name.
///
/// # Panics
/// Panics if the fixture cannot be read or parsed.
#[must_use]
#[allow(clippy::panic)]
pub fn load_fixture(fixture_name: &str) -> Value {
    let path = format!("tests/fixtures/{fixture_name}");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("Failed to read fixture {fixture_name}: {err}"));
    serde_json::from_str(&content)
        .unwrap_or_else(|err| panic!("Failed to parse JSON fixture {fixture_name}: {err}"))
}
