use serde_json::Value;

pub fn url_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

pub fn url_decode(input: &str) -> Result<String, String> {
    let mut bytes = Vec::with_capacity(input.len());
    let mut chars = input.bytes().peekable();

    while let Some(b) = chars.next() {
        if b == b'%' {
            let h1 = chars.next().ok_or_else(|| {
                "Incomplete percent-encoding sequence at end of input".to_string()
            })?;
            let h2 = chars.next().ok_or_else(|| {
                "Incomplete percent-encoding sequence at end of input".to_string()
            })?;
            let hex_bytes = [h1, h2];
            let hex_str = std::str::from_utf8(&hex_bytes)
                .map_err(|_| "Invalid ASCII in percent-encoding escape".to_string())?;
            let byte = u8::from_str_radix(hex_str, 16)
                .map_err(|_| format!("Invalid hex escape '%{hex_str}'"))?;
            bytes.push(byte);
        } else if b == b'+' {
            bytes.push(b' ');
        } else {
            bytes.push(b);
        }
    }

    String::from_utf8(bytes).map_err(|e| format!("Decoded bytes are not valid UTF-8: {e}"))
}

pub fn hex_encode(input: &[u8]) -> String {
    input.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn hex_decode(input: &str) -> Result<Vec<u8>, String> {
    let trimmed = input.trim();
    let without_prefix = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);

    let cleaned: String = without_prefix
        .chars()
        .filter(|c| !c.is_ascii_whitespace() && *c != ':')
        .collect();

    if !cleaned.len().is_multiple_of(2) {
        return Err("Hex string must have an even number of hex digits".to_string());
    }

    let mut result = Vec::with_capacity(cleaned.len() / 2);
    for i in (0..cleaned.len()).step_by(2) {
        let chunk = &cleaned[i..i + 2];
        let byte = u8::from_str_radix(chunk, 16)
            .map_err(|_| format!("Invalid hex byte sequence '{chunk}'"))?;
        result.push(byte);
    }

    Ok(result)
}

pub fn hex_decode_to_string(input: &str) -> Result<String, String> {
    let bytes = hex_decode(input)?;
    String::from_utf8(bytes).map_err(|e| format!("Decoded hex bytes are not valid UTF-8: {e}"))
}

pub struct JwtInspection {
    pub header: Value,
    pub payload: Value,
    pub signature: String,
    pub issued_at: Option<i64>,
    pub not_before: Option<i64>,
    pub expires_at: Option<i64>,
}

pub fn inspect_jwt(token: &str) -> Result<JwtInspection, String> {
    let trimmed = token.trim();
    let stripped = if trimmed.len() >= 7 && trimmed[..7].eq_ignore_ascii_case("bearer ") {
        trimmed[7..].trim()
    } else {
        trimmed
    };

    let parts: Vec<&str> = stripped.split('.').collect();
    if parts.len() < 2 || parts.len() > 3 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(
            "Invalid JWT format: expected header.payload or header.payload.signature".to_string(),
        );
    }

    let decode_part = |name: &str, part: &str| -> Result<Value, String> {
        let decoded = crate::crypto::base64_decode(part)
            .map_err(|e| format!("Failed to decode JWT {name}: {e}"))?;
        serde_json::from_str(&decoded).map_err(|e| format!("JWT {name} is not valid JSON: {e}"))
    };

    let header = decode_part("header", parts[0])?;
    let payload = decode_part("payload", parts[1])?;
    let signature = if parts.len() == 3 {
        parts[2].to_string()
    } else {
        String::new()
    };

    let extract_ts = |key: &str| -> Option<i64> {
        payload
            .get(key)
            .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
    };

    Ok(JwtInspection {
        issued_at: extract_ts("iat"),
        not_before: extract_ts("nbf"),
        expires_at: extract_ts("exp"),
        header,
        payload,
        signature,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encode_and_decode() {
        let original = "Hello World! / 2026? &key=val+1";
        let encoded = url_encode(original);
        assert_eq!(
            encoded,
            "Hello%20World%21%20%2F%202026%3F%20%26key%3Dval%2B1"
        );

        let decoded = url_decode(&encoded).expect("Decode should succeed");
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_url_decode_plus_as_space() {
        let query = "Rust+Developer+Tool";
        let decoded = url_decode(query).expect("Decode should succeed");
        assert_eq!(decoded, "Rust Developer Tool");
    }

    #[test]
    fn test_url_decode_invalid() {
        let invalid = "invalid%2Gsequence";
        assert!(url_decode(invalid).is_err());
    }

    #[test]
    fn test_hex_encode_and_decode() {
        let raw = "Omnitool Fast CLI";
        let hex = hex_encode(raw.as_bytes());
        assert_eq!(hex, "4f6d6e69746f6f6c204661737420434c49");

        let decoded_str = hex_decode_to_string(&hex).expect("Hex decode should succeed");
        assert_eq!(decoded_str, raw);
    }

    #[test]
    fn test_hex_decode_with_prefix_and_spaces() {
        assert_eq!(hex_decode_to_string("0x4869").unwrap(), "Hi");
        assert_eq!(hex_decode_to_string("48 69").unwrap(), "Hi");
        assert_eq!(hex_decode_to_string("48:69").unwrap(), "Hi");
    }

    #[test]
    fn test_hex_decode_odd_length() {
        let odd = "123";
        assert!(hex_decode(odd).is_err());
    }

    #[test]
    fn test_hex_decode_invalid_chars() {
        let invalid = "zz";
        assert!(hex_decode(invalid).is_err());
    }

    #[test]
    fn test_inspect_jwt() {
        // Standard sample JWT:
        // Header: {"alg":"HS256","typ":"JWT"} -> eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9
        // Payload: {"sub":"1234567890","name":"John Doe","iat":1516239022} -> eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ
        // Signature: SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let inspection = inspect_jwt(token).expect("JWT should decode successfully");
        assert_eq!(inspection.header["alg"], "HS256");
        assert_eq!(inspection.payload["name"], "John Doe");
        assert_eq!(inspection.payload["sub"], "1234567890");
        assert_eq!(inspection.payload["iat"], 1516239022);
        assert_eq!(inspection.issued_at, Some(1516239022));
        assert_eq!(
            inspection.signature,
            "SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"
        );

        let with_bearer = format!("Bearer {token}");
        let inspection2 = inspect_jwt(&with_bearer).expect("Bearer prefix should work");
        assert_eq!(inspection2.payload["sub"], "1234567890");
    }

    #[test]
    fn test_inspect_jwt_invalid_format() {
        assert!(inspect_jwt("only-one-part").is_err());
        assert!(inspect_jwt(".abc").is_err());
    }
}
