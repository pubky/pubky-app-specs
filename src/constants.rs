// Application version
pub static VERSION: &str = "0.4.0";

// Path constants
pub static PUBLIC_PATH: &str = "/pub/";
pub static APP_PATH: &str = "pubky.app/";
pub static PROTOCOL: &str = "pubky://";

// Size limits
/// Maximum blob/file size (100 MB) in bytes
pub static MAX_SIZE: usize = 100 * (1 << 20); // 100MB

// Tag validation constants (shared across models)
pub const MAX_TAG_LABEL_LENGTH: usize = 20;
pub const MIN_TAG_LABEL_LENGTH: usize = 1;
/// Disallowed characters, in addition to whitespace chars
pub const INVALID_TAG_CHARS: &[char] = &[',', ':'];
