#[cfg(target_arch = "wasm32")]
use js_sys::Date;

use base32::{decode, Alphabet};
use url::Url;

/// Returns the current timestamp in microseconds since the UNIX epoch.
#[cfg(target_arch = "wasm32")]
pub fn timestamp() -> i64 {
    let ms = Date::now() as i64;
    ms * 1_000
}

#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(not(target_arch = "wasm32"))]
pub fn timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as i64
}

/// Trims whitespace and normalizes a URL if valid and invalid URLs are preserved
/// (not discarded) so validation can catch them
pub fn sanitize_url(input: &str) -> String {
    let trimmed = input.trim();
    match Url::parse(trimmed) {
        Ok(parsed_url) => parsed_url.to_string(),
        Err(_) => trimmed.to_string(),
    }
}

/// Validates structural correctness of a Crockford Base32-encoded ID (13
/// characters, decodes to 8 bytes). Returns the decoded bytes on success.
pub fn validate_crockford_id(id: &str) -> Result<[u8; 8], String> {
    if id.len() != 13 {
        return Err("Validation Error: Invalid ID length: must be 13 characters".into());
    }

    let decoded_bytes = decode(Alphabet::Crockford, id)
        .ok_or("Validation Error: Invalid Crockford Base32 encoding")?;

    if decoded_bytes.len() != 8 {
        return Err("Validation Error: Invalid ID length after decoding".into());
    }

    Ok(decoded_bytes.try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_id_passes() {
        let id = "0000000000000";
        assert!(validate_crockford_id(id).is_ok());
    }

    #[test]
    fn wrong_length_fails() {
        assert!(validate_crockford_id("12345").is_err());
    }

    #[test]
    fn invalid_id_fails() {
        assert!(validate_crockford_id("UUUUUUUUUUUUU").is_err());
    }

}
