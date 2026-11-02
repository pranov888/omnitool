use base64::prelude::*;
use md5::{Digest, Md5};
use rand::Rng;
use sha2::Sha256;
use uuid::Uuid;

pub fn hash_sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn hash_md5(input: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn base64_encode(input: &str) -> String {
    BASE64_STANDARD.encode(input.as_bytes())
}

pub fn base64_decode(input: &str) -> Result<String, String> {
    let trimmed = input.trim();

    // 1. Try standard base64 directly
    if let Ok(bytes) = BASE64_STANDARD.decode(trimmed) {
        return String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 output: {e}"));
    }

    // 2. Try URL-safe base64 directly
    if let Ok(bytes) = BASE64_URL_SAFE.decode(trimmed) {
        return String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 output: {e}"));
    }

    // 3. Fallback for unpadded inputs (common in JWTs / URL query params)
    let padded = match trimmed.len() % 4 {
        2 => format!("{trimmed}=="),
        3 => format!("{trimmed}="),
        _ => trimmed.to_string(),
    };

    let bytes = BASE64_STANDARD
        .decode(&padded)
        .or_else(|_| BASE64_URL_SAFE.decode(&padded))
        .map_err(|e| format!("Base64 decoding failed: {e}"))?;

    String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 output: {e}"))
}

pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

pub fn generate_password(length: usize, include_symbols: bool) -> String {
    const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const NUMBERS: &[u8] = b"0123456789";
    const SYMBOLS: &[u8] = b"!@#$%^&*()-_=+[]{}|;:,.<>?";

    let mut pool = Vec::new();
    pool.extend_from_slice(LOWERCASE);
    pool.extend_from_slice(UPPERCASE);
    pool.extend_from_slice(NUMBERS);
    if include_symbols {
        pool.extend_from_slice(SYMBOLS);
    }

    let mut rng = rand::thread_rng();
    let length = length.max(6);
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..pool.len());
            pool[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        assert_eq!(
            hash_sha256("hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_md5() {
        assert_eq!(hash_md5("hello"), "5d41402abc4b2a76b9719d911017c592");
    }

    #[test]
    fn test_base64_roundtrip() {
        let original = "Rust all-in-one developer toolkit";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).expect("Decode should succeed");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_base64_url_safe_and_unpadded() {
        // "subjects?_d=1" encoded in url-safe without padding
        let input = "c3ViamVjdHM_X2Q9MQ";
        let decoded = base64_decode(input).expect("Should decode URL-safe unpadded string");
        assert_eq!(decoded, "subjects?_d=1");
    }

    #[test]
    fn test_uuid() {
        let id = generate_uuid();
        assert_eq!(id.len(), 36);
    }

    #[test]
    fn test_password_length() {
        let pass = generate_password(16, true);
        assert_eq!(pass.len(), 16);
    }
}
