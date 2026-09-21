//! The 0.x reader, frozen at the 0.8.0 pin so a v0 ingest verdict never moves.
//!
//! Every file under this module is the 0.8.0 source, copied unchanged except for import
//! paths and the stripped JS bindings. Its `url::Url` parsing, its `mime` gate, its
//! `.trim()`/`to_lowercase()` and its error messages ARE the verdict real v0 data was
//! accepted or rejected under, so they are deliberately outside the v1 rules and are never
//! edited, improved or aligned. Indexers read un-migrated data through it forever and the
//! migration transforms read v0 objects through it.
//!
//! One thing is shared rather than copied: [`PubkyId`] is the crate's id type. Two id types
//! would fork every signature a consumer writes, and the acceptance set is the same, a
//! 52-character z-base32 host.

mod common;
mod constants;
pub mod limits;
mod models;
pub mod traits;
mod uri;

pub use crate::types::PubkyId;
pub use common::validate_crockford_id;
pub use constants::{APP_PATH, PROTOCOL, PUBLIC_PATH};
pub use limits::{ValidationLimits, VALIDATION_LIMITS};
pub use models::blob::PubkyAppBlob;
pub use models::bookmark::PubkyAppBookmark;
pub use models::feed::{
    PubkyAppFeed, PubkyAppFeedConfig, PubkyAppFeedLayout, PubkyAppFeedReach, PubkyAppFeedSort,
};
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
pub use uri::{
    base_uri_builder, blob_uri_builder, bookmark_uri_builder, feed_uri_builder, file_uri_builder,
    follow_uri_builder, is_pubky_scheme, last_read_uri_builder, mute_uri_builder, post_uri_builder,
    tag_uri_builder, try_parse_pubky_path, user_uri_builder, ExtendedParsedUri, ParsedUri,
    PubkyPath, Resource,
};
