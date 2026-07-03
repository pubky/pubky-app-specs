// Application version (synced with Cargo.toml at compile time)
pub static VERSION: &str = env!("CARGO_PKG_VERSION");

// Path constants
pub static PUBLIC_PATH: &str = "/pub/";
pub static APP_PATH: &str = "pubky.app/";
pub static PROTOCOL: &str = "pubky://";
