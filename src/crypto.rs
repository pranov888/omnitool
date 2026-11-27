use base64::prelude::*;
use md5::{Digest, Md5};
use rand::seq::SliceRandom;
use rand::Rng;
use sha2::{Sha256, Sha512};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use uuid::Uuid;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ChecksumAlgorithm {
    Sha256,
    Sha512,
    Md5,
}

pub struct ChecksumResult {
    pub algorithm: String,
    pub digest: String,
    pub bytes_read: u64,
}

fn hash_reader<D: Digest>(mut reader: BufReader<File>) -> Result<(String, u64), std::io::Error> {
    let mut hasher = D::new();
    let mut buffer = [0u8; 8192];
    let mut total_bytes = 0u64;

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        total_bytes += count as u64;
    }

    let digest = hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    Ok((digest, total_bytes))
}

pub fn compute_file_checksum<P: AsRef<Path>>(
    path: P,
    algorithm: ChecksumAlgorithm,
) -> Result<ChecksumResult, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let (algo_name, (digest, bytes_read)) = match algorithm {
        ChecksumAlgorithm::Sha256 => ("SHA-256", hash_reader::<Sha256>(reader)?),
        ChecksumAlgorithm::Sha512 => ("SHA-512", hash_reader::<Sha512>(reader)?),
        ChecksumAlgorithm::Md5 => ("MD5", hash_reader::<Md5>(reader)?),
    };

    Ok(ChecksumResult {
        algorithm: algo_name.to_string(),
        digest,
        bytes_read,
    })
}

pub fn hash_sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn hash_sha512(input: &str) -> String {
    let mut hasher = Sha512::new();
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
    // Strip surrounding and embedded ASCII whitespace (common in multiline MIME/CLI inputs)
    let cleaned: String = input.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    if cleaned.is_empty() {
        return Ok(String::new());
    }

    // 1. Try standard base64 directly
    if let Ok(bytes) = BASE64_STANDARD.decode(&cleaned) {
        return String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 output: {e}"));
    }

    // 2. Try URL-safe base64 directly
    if let Ok(bytes) = BASE64_URL_SAFE.decode(&cleaned) {
        return String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 output: {e}"));
    }

    // 3. Fallback for unpadded inputs (common in JWTs / URL query params)
    let padded = match cleaned.len() % 4 {
        2 => format!("{cleaned}=="),
        3 => format!("{cleaned}="),
        _ => cleaned,
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

pub fn try_generate_password(length: usize, include_symbols: bool) -> Result<String, String> {
    if length == 0 {
        return Err("Password length must be at least 1".to_string());
    }

    const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const NUMBERS: &[u8] = b"0123456789";
    const SYMBOLS: &[u8] = b"!@#$%^&*()-_=+[]{}|;:,.<>?";

    let mut categories: Vec<&[u8]> = vec![LOWERCASE, UPPERCASE, NUMBERS];
    if include_symbols {
        categories.push(SYMBOLS);
    }

    let mut pool = Vec::new();
    for cat in &categories {
        pool.extend_from_slice(cat);
    }

    let mut rng = rand::thread_rng();
    let mut chars = Vec::with_capacity(length);

    // Guarantee at least one character from each enabled character class when length permits
    if length >= categories.len() {
        for cat in &categories {
            let idx = rng.gen_range(0..cat.len());
            chars.push(cat[idx]);
        }
    }

    while chars.len() < length {
        let idx = rng.gen_range(0..pool.len());
        chars.push(pool[idx]);
    }

    chars.shuffle(&mut rng);
    Ok(chars.into_iter().map(|b| b as char).collect())
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
    fn test_sha512() {
        assert_eq!(
            hash_sha512("hello"),
            "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043"
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
    fn test_base64_multiline_whitespace() {
        let encoded = "ZGV2\nZWxv\ncGVy";
        assert_eq!(base64_decode(encoded).unwrap(), "developer");
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
    fn test_password_length_and_diversity() {
        let pass = try_generate_password(16, true).expect("valid password");
        assert_eq!(pass.len(), 16);
        assert!(pass.chars().any(|c| c.is_ascii_lowercase()));
        assert!(pass.chars().any(|c| c.is_ascii_uppercase()));
        assert!(pass.chars().any(|c| c.is_ascii_digit()));
        assert!(pass.chars().any(|c| !c.is_ascii_alphanumeric()));

        let alnum = try_generate_password(12, false).expect("alnum password");
        assert_eq!(alnum.len(), 12);
        assert!(alnum.chars().all(|c| c.is_ascii_alphanumeric()));

        assert!(try_generate_password(0, true).is_err());
        assert_eq!(try_generate_password(4, false).unwrap().len(), 4);
    }

    #[test]
    fn test_compute_file_checksum() {
        use std::io::Write;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("omnitool_checksum_test.txt");
        let content = b"hello";
        let mut f = File::create(&test_file).expect("Create test file");
        f.write_all(content).expect("Write test file");

        let sha256_res =
            compute_file_checksum(&test_file, ChecksumAlgorithm::Sha256).expect("SHA256 checksum");
        assert_eq!(sha256_res.digest, hash_sha256("hello"));
        assert_eq!(sha256_res.bytes_read, 5);

        let sha512_res =
            compute_file_checksum(&test_file, ChecksumAlgorithm::Sha512).expect("SHA512 checksum");
        assert_eq!(sha512_res.digest, hash_sha512("hello"));
        assert_eq!(sha512_res.bytes_read, 5);

        let md5_res =
            compute_file_checksum(&test_file, ChecksumAlgorithm::Md5).expect("MD5 checksum");
        assert_eq!(md5_res.digest, hash_md5("hello"));
        assert_eq!(md5_res.bytes_read, 5);

        let _ = std::fs::remove_file(test_file);
    }
}
