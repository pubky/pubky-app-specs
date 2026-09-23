//! URI parsing and construction for `pubky://` paths.
//!
//! - **Build**: the `*_uri_builder` helpers assemble canonical URIs.
//! - **Parse**: [`ParsedUri`] classifies a path into visibility and resource.
//! - **Scheme**: [`is_pubky_scheme`] is the one string check shared with callers.

mod builders;
mod parsed;
mod resource;
mod scheme;

// Build
pub use builders::{
    bookmark_uri_builder, feed_uri_builder, file_uri_builder, follow_uri_builder,
    list_prefix_builder, mute_uri_builder, post_uri_builder, private_list_prefix_builder,
    tag_uri_builder, user_uri_builder,
};

// Strict
pub(crate) use parsed::{is_valid_label, media_stem, parse_version_leaf};
pub use parsed::{ParsedUri, Visibility};
pub use resource::Resource;

// Structure (advanced)
pub use scheme::is_pubky_scheme;
