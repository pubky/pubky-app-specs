pub static VERSION: &str = "0.3.0";
pub static APP_PATH: &str = "/pub/pubky.app/";
pub static PROTOCOL: &str = "pubky://";

#[cfg(target_arch = "wasm32")]
use js_sys::Date;

/// Returns the current timestamp in microseconds since the UNIX epoch.
#[cfg(target_arch = "wasm32")]
pub fn timestamp() -> i64 {
    // Use JS Date.now() which returns ms since Unix epoch
    let ms = Date::now() as i64;
    // Convert to microseconds if you like
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
