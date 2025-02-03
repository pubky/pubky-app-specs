mod common;
mod models;
pub mod traits;
mod types;
mod uri_parser;
mod utils;

// Re-export domain types
pub use common::{APP_PATH, PROTOCOL, PUBLIC_PATH, VERSION};
pub use models::blob::PubkyAppBlob;
pub use models::bookmark::PubkyAppBookmark;
pub use models::feed::{PubkyAppFeed, PubkyAppFeedLayout, PubkyAppFeedReach, PubkyAppFeedSort};
pub use models::file::PubkyAppFile;
pub use models::follow::PubkyAppFollow;
pub use models::last_read::PubkyAppLastRead;
pub use models::mute::PubkyAppMute;
pub use models::post::{PubkyAppPost, PubkyAppPostEmbed, PubkyAppPostKind};
pub use models::tag::PubkyAppTag;
pub use models::user::{PubkyAppUser, PubkyAppUserLink};
pub use models::PubkyAppObject;
pub use types::PubkyId;
pub use uri_parser::{ParsedUri, Resource};
pub use utils::*;

// Our WASM module
#[cfg(target_arch = "wasm32")]
mod wasm;
// Re-export the Wasm functions so they're available to wasm-pack
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
