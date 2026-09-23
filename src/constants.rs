// Application version (synced with Cargo.toml at compile time)
pub static VERSION: &str = env!("CARGO_PKG_VERSION");

pub static PROTOCOL: &str = "pubky://";
pub const PUBLIC_ROOT: &str = "pub"; // world-readable, anonymous GET
pub const PRIVATE_ROOT: &str = "priv"; // owner-only, excluded from public /events/
pub const SOCIAL_NAMESPACE: &str = "social"; // the shared domain every app writes under the same rules
pub const SOCIAL_EPOCH: u8 = 1; // data-format epoch; path segment "v1"

/// The epoch path segment ("v1"). Consumers never hardcode it.
pub fn epoch_segment() -> String {
    format!("v{SOCIAL_EPOCH}")
}

/// "/{root}/{namespace}/v1/{leaf}". The single path-assembly point; the epoch is this
/// crate's, whatever the namespace.
pub fn namespace_path(root: crate::traits::Root, namespace: &str, leaf: &str) -> String {
    [
        "/",
        root.segment(),
        "/",
        namespace,
        "/",
        &epoch_segment(),
        "/",
        leaf,
    ]
    .concat()
}

/// "/{root}/social/v1/{leaf}".
pub fn social_path(root: crate::traits::Root, leaf: &str) -> String {
    namespace_path(root, SOCIAL_NAMESPACE, leaf)
}
