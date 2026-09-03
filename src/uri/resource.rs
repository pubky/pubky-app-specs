use crate::models::{
    blob::PubkySocialBlob, bookmark::PubkySocialBookmark, feed::PubkySocialFeed,
    file::PubkySocialFile, follow::PubkySocialFollow, mute::PubkySocialMute, post::PubkySocialPost,
    tag::PubkySocialTag, user::PubkySocialUser,
};
use crate::traits::{HasIdPath, HasPath};
use crate::types::PubkyId;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A resource inside the `social/v1` namespace, classified from a path. Failed id or format
/// checks classify as `Unknown`; paths outside the namespace classify as `Foreign`; a
/// `social/vN` the crate does not speak yet is `UnsupportedVersion`. None of those is an error.
#[non_exhaustive]
#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub enum Resource {
    User,
    /// `version: None` is the versionless reference (a logical identity, never a stored file);
    /// `Some` is a stored version. `label` is the optional readable tail of the version leaf
    /// (`{editId}-{label}.json`); it carries no identity and is `Some` only with a version.
    Post {
        id: String,
        version: Option<String>,
        label: Option<String>,
    },
    Follow(PubkyId),
    Mute(PubkyId),
    Bookmark(String),
    Tag(String),
    /// The stripped id. The media collapse re-decides the payload when extensions vary.
    File(String),
    /// v0-shaped raw bytes under the v1 epoch; the media collapse removes it.
    Blob(String),
    Feed(String),
    /// A path under another namespace: valid, classified, skipped by social readers.
    Foreign {
        namespace: String,
        version: Option<String>,
        rest: Vec<String>,
    },
    /// A `social/vN` epoch this crate does not speak: a reader's "upgrade me" signal.
    UnsupportedVersion {
        version: String,
    },
    #[default]
    Unknown,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Resource::User => PubkySocialUser::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Post { .. } => PubkySocialPost::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Follow(_) => PubkySocialFollow::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Mute(_) => PubkySocialMute::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Bookmark(_) => PubkySocialBookmark::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Tag(_) => PubkySocialTag::PATH_SEGMENT.trim_end_matches('/'),
            Resource::File(_) => PubkySocialFile::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Blob(_) => PubkySocialBlob::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Feed(_) => PubkySocialFeed::PATH_SEGMENT.trim_end_matches('/'),
            Resource::Foreign { .. } => "foreign",
            Resource::UnsupportedVersion { .. } => "unsupported_version",
            Resource::Unknown => "unknown",
        };
        write!(f, "{}", name)
    }
}

impl Resource {
    /// The resource's id, when it has one.
    pub fn id(&self) -> Option<String> {
        match self {
            Resource::Post { id, .. } => Some(id.clone()),
            Resource::Follow(id) => Some(id.to_string()),
            Resource::Mute(id) => Some(id.to_string()),
            Resource::Bookmark(id) => Some(id.clone()),
            Resource::Tag(id) => Some(id.clone()),
            Resource::File(id) => Some(id.clone()),
            Resource::Blob(id) => Some(id.clone()),
            Resource::Feed(id) => Some(id.clone()),
            Resource::User
            | Resource::Foreign { .. }
            | Resource::UnsupportedVersion { .. }
            | Resource::Unknown => None,
        }
    }
}
