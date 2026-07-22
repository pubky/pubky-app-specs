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
pub use common::validate_crockford_id;
#[doc(inline)]
pub use limits::*;
// Re-export domain types
pub use models::blob::PubkyAppBlob;
pub use models::bookmark::PubkyAppBookmark;
pub use models::feed::{PubkyAppFeed, PubkyAppFeedLayout, PubkyAppFeedReach, PubkyAppFeedSort};
pub use models::file::{PubkyAppFile, VALID_MIME_TYPES};
pub use models::follow::PubkyAppFollow;
pub use models::last_read::PubkyAppLastRead;
pub use models::mute::PubkyAppMute;
pub use models::post::{
    PubkyAppCollectionContent, PubkyAppCollectionLayout, PubkyAppPost, PubkyAppPostEmbed,
    PubkyAppPostKind,
};
pub use models::tag::PubkyAppTag;
pub use models::user::{PubkyAppUser, PubkyAppUserLink};
pub use models::PubkyAppObject;
pub use types::PubkyId;
#[doc(inline)]
pub use uri::{
    base_uri_builder, blob_uri_builder, bookmark_uri_builder, feed_uri_builder, file_uri_builder,
    follow_uri_builder, is_pubky_scheme, last_read_uri_builder, mute_uri_builder, post_uri_builder,
    tag_uri_builder, try_parse_pubky_path, user_uri_builder, ExtendedParsedUri, ParsedUri,
    PubkyPath, Resource,
};

// Our WASM module
#[cfg(target_arch = "wasm32")]
mod wasm;
// Re-export the Wasm functions so they're available to wasm-pack
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
