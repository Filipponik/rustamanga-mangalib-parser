use std::{fs, process};

use serde_json::Value;

/// Loads JSON fixture by name.
#[must_use]
pub fn load_fixture(fixture_name: &str) -> Value {
    let path = format!("tests/fixtures/{fixture_name}");
    let content = fs::read_to_string(&path).unwrap_or_else(|err| {
        eprintln!("Failed to read fixture {fixture_name}: {err}");
        process::exit(1);
    });
    serde_json::from_str(&content).unwrap_or_else(|err| {
        eprintln!("Failed to parse JSON fixture {fixture_name}: {err}");
        process::exit(1);
    })
}
