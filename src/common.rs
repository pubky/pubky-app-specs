#[cfg(target_arch = "wasm32")]
use js_sys::Date;

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
