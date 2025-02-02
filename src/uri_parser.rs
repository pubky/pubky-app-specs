use crate::{
    traits::{HasPath, HasPubkyIdPath},
    PubkyAppBlob, PubkyAppBookmark, PubkyAppFeed, PubkyAppFile, PubkyAppFollow, PubkyAppMute,
    PubkyAppPost, PubkyAppTag, PubkyId, APP_PATH, PROTOCOL, PUBLIC_PATH,
};
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub enum ResourceId {
    Post(String),
    Follow(PubkyId),
    Mute(PubkyId),
    Bookmark(String),
    Tag(String),
    File(String),
    Blob(String),
    Feed(String),
}

#[derive(Debug, Default)]
pub struct ParsedUri {
    pub user_id: PubkyId,
    pub resource: Option<ResourceId>,
}

impl TryFrom<&str> for ParsedUri {
    type Error = String;
    fn try_from(uri: &str) -> Result<Self, Self::Error> {
        let mut parsed_uri = ParsedUri::default();

        // 1. Validate protocol
        if !uri.starts_with(PROTOCOL) {
            return Err(format!(
                "Invalid URI, must start with '{}',  {}",
                PROTOCOL, uri
            ));
        }

        // 2. Validate that the URI belongs to the correct app
        if let Some(app_segment) = extract_segment(uri, PUBLIC_PATH, "/") {
            if app_segment != APP_PATH.trim_end_matches('/') {
                return Err(format!(
                    "The Event URI does not belong to {},  {}",
                    APP_PATH, uri
                ));
            }
        } else {
            return Err(format!("The Event URI is malformed,  {}", uri));
        }

        // 3. Extract the user_id
        if let Some(user_id) = extract_segment(uri, PROTOCOL, PUBLIC_PATH) {
            parsed_uri.user_id = PubkyId::try_from(user_id)?;
        } else {
            return Err(format!("Uri Pubky ID is invalid,  {}", uri));
        }

        // 4. Remove the protocol, public path, and app parts, then split the remainder.
        //    For example, given:
        //      "pubky://<user_id>/pub/pubky.app/posts/123"
        //    we want to split the "posts/123" portion.
        let after_app = uri
            .split(&format!("{}{}", PUBLIC_PATH, APP_PATH))
            .nth(1)
            .unwrap_or("");
        let parts: Vec<&str> = after_app.split('/').filter(|s| !s.is_empty()).collect();

        parsed_uri.resource = if parts.len() >= 2 {
            // parts[0] is the resource type and parts[1] is the resource id.
            let resource_segment = format!("{}/", parts[0]);
            let resource_id = parts[1];
            match resource_segment.as_str() {
                PubkyAppPost::PATH_SEGMENT => Some(ResourceId::Post(resource_id.to_string())),
                PubkyAppFollow::PATH_SEGMENT => {
                    Some(ResourceId::Follow(PubkyId::try_from(resource_id)?))
                }
                PubkyAppMute::PATH_SEGMENT => {
                    Some(ResourceId::Mute(PubkyId::try_from(resource_id)?))
                }
                PubkyAppBookmark::PATH_SEGMENT => {
                    Some(ResourceId::Bookmark(resource_id.to_string()))
                }
                PubkyAppTag::PATH_SEGMENT => Some(ResourceId::Tag(resource_id.to_string())),
                PubkyAppFile::PATH_SEGMENT => Some(ResourceId::File(resource_id.to_string())),
                PubkyAppBlob::PATH_SEGMENT => Some(ResourceId::Blob(resource_id.to_string())),
                PubkyAppFeed::PATH_SEGMENT => Some(ResourceId::Feed(resource_id.to_string())),
                other => return Err(format!("Unknown resource type: {}", other)),
            }
        } else {
            None
        };

        Ok(parsed_uri)
    }
}

fn extract_segment<'a>(uri: &'a str, start_pattern: &str, end_pattern: &str) -> Option<&'a str> {
    let start_idx = uri.find(start_pattern)? + start_pattern.len();
    let end_idx = uri[start_idx..]
        .find(end_pattern)
        .map(|i| i + start_idx)
        .unwrap_or_else(|| uri.len());

    Some(&uri[start_idx..end_idx])
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_empty_bookmark_uri() {
        let uri =
            "pubky://phbhg3qgcttn95guepmbud1nzcxhg3xc5j5k4h7i8a4b6wb3nw1o/pub/pubky.app/bookmarks/";
        let parsed_uri = ParsedUri::try_from(uri).unwrap_or_default();
        assert_eq!(
            parsed_uri.resource, None,
            "The provided URI has bookmark_id"
        );
    }

    #[test]
    fn test_some_bookmark_uri() {
        let uri =
            "pubky://phbhg3qgcttn95guepmbud1nzcxhg3xc5j5k4h7i8a4b6wb3nw1o/pub/pubky.app/bookmarks/00";
        let parsed_uri = ParsedUri::try_from(uri).unwrap_or_default();
        assert_eq!(
            parsed_uri.resource,
            Some(ResourceId::Bookmark("00".to_string())),
            "The provided URI has wrong"
        );
    }
}
