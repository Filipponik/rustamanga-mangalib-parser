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

#[cfg(test)]
mod tests {
    use super::to_string;
    use serde_json::Deserializer;

    #[test]
    fn deserializes_number_to_string() {
        let mut de = Deserializer::from_str("123");
        let result = to_string(&mut de).unwrap();
        assert_eq!(result, "123");
    }

    #[test]
    fn deserializes_string_value() {
        let mut de = Deserializer::from_str("\"hello\"");
        let result = to_string(&mut de).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn rejects_unsupported_type() {
        let mut de = Deserializer::from_str("true");
        let err = to_string(&mut de).unwrap_err();
        assert!(
            format!("{err}").contains("expected a number or string"),
            "unexpected error message: {err}"
        );
    }
}
