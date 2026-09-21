mod canonicalize;
mod common;
mod constants;
pub mod limits;
mod mime;
mod models;
pub mod traits;
mod types;
mod uri;

// Re-export constants
pub use constants::{
    epoch_segment, namespace_path, social_path, PROTOCOL, SOCIAL_EPOCH, SOCIAL_NAMESPACE, VERSION,
};
// Re-export common utilities
pub use canonicalize::{
    canonicalize_external_uri, canonicalize_pubky_uri, canonicalize_target, canonicalize_universal,
    canonicalize_web_uri, validate_reference, AllowedSchemes,
};
pub use common::{
    ascii_fold, code_point_len, frozen_trim, is_frozen_whitespace, mint_timestamp_micros,
    validate_hash_id_format, validate_safe_json_int, validate_timestamp_id_format,
    FROZEN_WHITESPACE, MAX_SAFE_JSON_INT,
};
// Re-export the frozen MIME map
#[doc(inline)]
pub use limits::*;
pub use mime::{essence, mime_to_ext, MIME_TO_EXT, STRIP_SET};
pub use traits::{Root, ValidationCtx, ValidationError, PUB_CTX};
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
pub use models::post::lifecycle::{
    plan_delete, plan_publish, plan_unpublish, DeletePlan, PublishPlan, UnpublishPlan,
};
pub use models::post::{
    MintedVersion, PubkySocialArticleContent, PubkySocialAttachment, PubkySocialCollectionContent,
    PubkySocialCollectionItem, PubkySocialCollectionLayout, PubkySocialPost, PubkySocialPostKind,
};
pub use models::tag::{sanitize_tag_label, validate_tag_label, PubkySocialTag};
pub use models::user::{PubkySocialUser, PubkySocialUserLink};
pub use models::PubkySocialObject;
pub use types::PubkyId;
#[doc(inline)]
pub use uri::{
    blob_uri_builder, bookmark_uri_builder, feed_uri_builder, file_uri_builder, follow_uri_builder,
    is_pubky_scheme, list_prefix_builder, mute_uri_builder, post_uri_builder,
    private_list_prefix_builder, tag_uri_builder, user_uri_builder, ParsedUri, Resource,
    Visibility,
};

// Our WASM module
#[cfg(target_arch = "wasm32")]
mod wasm;
// Re-export the Wasm functions so they're available to wasm-pack
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
