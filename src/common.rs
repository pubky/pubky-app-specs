use std::time::{SystemTime, UNIX_EPOCH};

pub static VERSION: &str = "0.2.1";
pub static APP_PATH: &str = "/pub/pubky.app/";
pub static PROTOCOL: &str = "pubky://";

/// Returns the current timestamp in microseconds since the UNIX epoch.
pub fn timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_micros() as i64
}
