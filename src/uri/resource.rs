use crate::{
    traits::{HasIdPath, HasPath},
    PubkyAppBlob, PubkyAppBookmark, PubkyAppFeed, PubkyAppFile, PubkyAppFollow, PubkyAppLastRead,
    PubkyAppMute, PubkyAppPost, PubkyAppTag, PubkyAppUser, PubkyId,
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
    LastRead,
    #[default]
    Unknown,
}

impl fmt::Display for Resource {
    /// Returns the resource name without any identifier.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use the associated constant for each resource type, trimming any trailing '/'
        let name = match self {
            Resource::User => PubkyAppUser::PATH_SEGMENT.trim_end_matches('/'),
            Resource::LastRead => PubkyAppLastRead::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Post(_) => PubkyAppPost::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Follow(_) => PubkyAppFollow::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Mute(_) => PubkyAppMute::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Bookmark(_) => PubkyAppBookmark::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Tag(_) => PubkyAppTag::PATH_SEGMENT.trim_end_matches('/'),
            Resource::File(_) => PubkyAppFile::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Blob(_) => PubkyAppBlob::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Feed(_) => PubkyAppFeed::PATH_SEGMENT.trim_end_matches('/'),
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
            Resource::User | Resource::LastRead | Resource::Unknown => None,
        }
    }
}
