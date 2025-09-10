use std::fs;

use serde_json::Value;

pub fn load_fixture(fixture_name: &str) -> Value {
    let path = format!("tests/fixtures/{}", fixture_name);
    let content =
        fs::read_to_string(path).expect(&format!("Failed to read fixture: {}", fixture_name));
    serde_json::from_str(&content)
        .expect(&format!("Failed to parse JSON fixture: {}", fixture_name))
}
