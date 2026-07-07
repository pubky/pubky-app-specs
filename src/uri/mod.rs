//! Pubky URI parsing and construction.
//!
//! Core modules define the `pubky.app` spec. See [`compat`] for admission rules
//! when ingesting URIs beyond strict spec paths.

mod builders;
mod compat;
mod parsed;
mod path;
mod resource;
mod scheme;

pub use builders::*;
pub use compat::CompatParsedUri;
pub use parsed::ParsedUri;
pub use path::{try_parse_pubky_path, PubkyPath};
pub use resource::Resource;
