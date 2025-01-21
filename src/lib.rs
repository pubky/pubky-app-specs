mod bookmark;
mod common;
mod feed;
mod file;
mod file_blob;
mod follow;
mod last_read;
mod mute;
mod post;
mod tag;
pub mod traits;
mod user;

// Re-export domain types
pub use bookmark::PubkyAppBookmark;
pub use common::{APP_PATH, PROTOCOL, VERSION};
pub use feed::{PubkyAppFeed, PubkyAppFeedLayout, PubkyAppFeedReach, PubkyAppFeedSort};
pub use file::PubkyAppFile;
pub use file_blob::PubkyAppBlob;
pub use follow::PubkyAppFollow;
pub use last_read::PubkyAppLastRead;
pub use mute::PubkyAppMute;
pub use post::{PubkyAppPost, PubkyAppPostEmbed, PubkyAppPostKind};
pub use tag::PubkyAppTag;
pub use user::{PubkyAppUser, PubkyAppUserLink};

// Our FFI module
#[cfg(target_arch = "wasm32")]
mod ffi;
// Re-export the FFI functions so they're available to wasm-pack
#[cfg(target_arch = "wasm32")]
pub use ffi::*;
