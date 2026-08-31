use crate::{
    traits::{HasIdPath, HasPath},
    PubkyId, PubkySocialBlob, PubkySocialBookmark, PubkySocialFeed, PubkySocialFile,
    PubkySocialFollow, PubkySocialMute, PubkySocialPost, PubkySocialTag, PubkySocialUser,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub enum Resource {
    User,
    Post(String),
    Follow(PubkyId),
    Mute(PubkyId),
    Bookmark(String),
    Tag(String),
    File(String),
    Blob(String),
    Feed(String),
    #[default]
    Unknown,
}

impl fmt::Display for Resource {
    /// Returns the resource name without any identifier.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use the associated constant for each resource type, trimming any trailing '/'
        let name = match self {
            Resource::User => PubkySocialUser::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Post(_) => PubkySocialPost::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Follow(_) => PubkySocialFollow::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Mute(_) => PubkySocialMute::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Bookmark(_) => PubkySocialBookmark::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Tag(_) => PubkySocialTag::PATH_SEGMENT.trim_end_matches('/'),
            Resource::File(_) => PubkySocialFile::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Blob(_) => PubkySocialBlob::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Feed(_) => PubkySocialFeed::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Unknown => "unknown",
        };
        write!(f, "{}", name)
    }
}

impl Resource {
    /// Returns the identifier as a `Some(String)` if the resource variant holds one,
    /// or `None` if there is no identifier.
    pub fn id(&self) -> Option<String> {
        match self {
            Resource::Post(id) => Some(id.clone()),
            Resource::Follow(id) => Some(id.to_string()),
            Resource::Mute(id) => Some(id.to_string()),
            Resource::Bookmark(id) => Some(id.clone()),
            Resource::Tag(id) => Some(id.clone()),
            Resource::File(id) => Some(id.clone()),
            Resource::Blob(id) => Some(id.clone()),
            Resource::Feed(id) => Some(id.clone()),
            // The following variants do not carry an id.
            Resource::User | Resource::Unknown => None,
        }
    }
}
