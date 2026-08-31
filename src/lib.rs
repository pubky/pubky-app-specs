mod common;
mod constants;
pub mod limits;
mod models;
pub mod traits;
mod types;
mod uri;

// Re-export constants
pub use constants::{APP_PATH, PROTOCOL, PUBLIC_PATH, VERSION};
// Re-export common utilities
pub use common::{
    ascii_fold, code_point_len, frozen_trim, is_frozen_whitespace, mint_timestamp_micros,
    validate_hash_id_format, validate_safe_json_int, validate_timestamp_id_format,
    FROZEN_WHITESPACE, MAX_SAFE_JSON_INT,
};
#[doc(inline)]
pub use limits::*;
// Re-export domain types
pub use models::blob::PubkySocialBlob;
pub use models::bookmark::PubkySocialBookmark;
pub use models::feed::{
    PubkySocialFeed, PubkySocialFeedConfig, PubkySocialFeedLayout, PubkySocialFeedReach,
    PubkySocialFeedSort,
};
pub use models::file::{PubkySocialFile, VALID_MIME_TYPES};
pub use models::follow::PubkySocialFollow;
pub use models::mute::PubkySocialMute;
pub use models::post::{
    PubkySocialCollectionContent, PubkySocialCollectionLayout, PubkySocialPost,
    PubkySocialPostEmbed, PubkySocialPostKind,
};
pub use models::tag::PubkySocialTag;
pub use models::user::{PubkySocialUser, PubkySocialUserLink};
pub use models::PubkySocialObject;
pub use types::PubkyId;
#[doc(inline)]
pub use uri::{
    base_uri_builder, blob_uri_builder, bookmark_uri_builder, feed_uri_builder, file_uri_builder,
    follow_uri_builder, is_pubky_scheme, mute_uri_builder, post_uri_builder, tag_uri_builder,
    try_parse_pubky_path, user_uri_builder, ExtendedParsedUri, ParsedUri, PubkyPath, Resource,
};

// Our WASM module
#[cfg(target_arch = "wasm32")]
mod wasm;
// Re-export the Wasm functions so they're available to wasm-pack
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
