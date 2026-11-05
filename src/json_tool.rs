use serde_json::Value;

pub fn format_json(raw: &str) -> Result<String, String> {
    let parsed: Value = serde_json::from_str(raw).map_err(|e| {
        format!(
            "Invalid JSON syntax at line {}:{}: {e}",
            e.line(),
            e.column()
        )
    })?;
    serde_json::to_string_pretty(&parsed).map_err(|e| format!("Formatting error: {e}"))
}

pub fn minify_json(raw: &str) -> Result<String, String> {
    let parsed: Value = serde_json::from_str(raw).map_err(|e| {
        format!(
            "Invalid JSON syntax at line {}:{}: {e}",
            e.line(),
            e.column()
        )
    })?;
    serde_json::to_string(&parsed).map_err(|e| format!("Minification error: {e}"))
}

pub fn validate_json(raw: &str) -> Result<String, String> {
    match serde_json::from_str::<Value>(raw) {
        Ok(Value::Object(map)) => Ok(format!(
            "Valid JSON Object with {} top-level keys",
            map.len()
        )),
        Ok(Value::Array(arr)) => Ok(format!("Valid JSON Array with {} elements", arr.len())),
        Ok(Value::String(_)) => Ok("Valid JSON primitive (string)".to_string()),
        Ok(Value::Number(_)) => Ok("Valid JSON primitive (number)".to_string()),
        Ok(Value::Bool(_)) => Ok("Valid JSON primitive (boolean)".to_string()),
        Ok(Value::Null) => Ok("Valid JSON null".to_string()),
        Err(e) => Err(format!(
            "Invalid JSON syntax at line {}:{}: {e}",
            e.line(),
            e.column()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_json() {
        let raw = r#"{"name":"omnitool","active":true}"#;
        let formatted = format_json(raw).expect("valid json");
        assert!(formatted.contains('\n'));
        assert!(formatted.contains(r#""name": "omnitool""#));
    }

    #[test]
    fn test_minify_json() {
        let pretty = "{\n  \"a\": 1,\n  \"b\": 2\n}";
        let minified = minify_json(pretty).expect("valid json");
        assert_eq!(minified, r#"{"a":1,"b":2}"#);
    }

    #[test]
    fn test_validate_json_error() {
        let invalid = r#"{"name": "broken",}"#;
        let res = validate_json(invalid);
        assert!(res.is_err());
    }
}
