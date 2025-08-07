pub static VERSION: &str = "0.4.0";
pub static PUBLIC_PATH: &str = "/pub/";
pub static APP_PATH: &str = "pubky.app/";
pub static PROTOCOL: &str = "pubky://";

// Define the maximum blob/file size (100 MB) in bytes.
pub static MAX_SIZE: usize = 100 * (1 << 20); // 100MB

#[cfg(target_arch = "wasm32")]
use js_sys::Date;

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
