//! URI parsing and construction for `pubky://` paths.
//!
//! - **Build** — `*_uri_builder` functions
//! - **Parse (strict)** — [`ParsedUri`] for `pubky.app` spec paths
//! - **Parse (extended)** — [`ExtendedParsedUri`] for ingest (cross-app tags)
//! - **Parse (structure)** — [`try_parse_pubky_path`] / [`PubkyPath`] (advanced)

mod builders;
mod compat;
mod parsed;
mod path;
mod resource;
mod scheme;

// Build
pub use builders::{
    base_uri_builder, blob_uri_builder, bookmark_uri_builder, feed_uri_builder, file_uri_builder,
    follow_uri_builder, last_read_uri_builder, mute_uri_builder, post_uri_builder,
    tag_uri_builder, user_uri_builder,
};

// Strict
pub use parsed::ParsedUri;
pub use resource::Resource;

// Compat
pub use compat::ExtendedParsedUri;

// Structure (advanced)
pub use path::{try_parse_pubky_path, PubkyPath};
pub use scheme::is_pubky_scheme;
