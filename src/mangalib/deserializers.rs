use serde::Deserializer;

pub fn to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::{Deserialize, de::Error, de::Unexpected};
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;

    match value {
        Value::Number(num) => Ok(num.to_string()),
        Value::String(s) => Ok(s),
        _ => Err(Error::invalid_type(
            Unexpected::Other("non-number/string value"),
            &"a number or string",
        )),
    }
}
