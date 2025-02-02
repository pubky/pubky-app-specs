use url::Url;

use crate::{
    traits::{HasPath, HasPubkyIdPath},
    PubkyAppBlob, PubkyAppBookmark, PubkyAppFeed, PubkyAppFile, PubkyAppFollow, PubkyAppLastRead,
    PubkyAppMute, PubkyAppPost, PubkyAppTag, PubkyAppUser, PubkyId, APP_PATH, PROTOCOL,
    PUBLIC_PATH,
};
use std::convert::TryFrom;

#[derive(Debug, PartialEq, Default)]
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

#[derive(Debug, Default)]
pub struct ParsedUri {
    pub user_id: PubkyId,
    pub resource: Resource,
}

impl TryFrom<&str> for ParsedUri {
    type Error = String;
    fn try_from(uri: &str) -> Result<Self, Self::Error> {
        let mut parsed_uri = ParsedUri::default();

        // 0. Validate and sanitize the URL.
        let parsed_url = Url::parse(uri).map_err(|e| format!("Invalid URL: {}", e))?;
        let uri = parsed_url.as_str();

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

        // 4. Extract the resource type and id
        // Remove the base portion.
        let base = format!("{}{}", PUBLIC_PATH, APP_PATH);
        let after_app = uri.split(&base).nth(1).unwrap_or("");
        let parts: Vec<&str> = after_app.split('/').filter(|s| !s.is_empty()).collect();

        // Modularize resource detection using slice pattern matching.
        parsed_uri.resource = match parts.as_slice() {
            // No extra segments.
            [] => Ok(Resource::Unknown),
            // A single segment: it must exactly match the identifier-less routes.
            [segment] => match *segment {
                s if s == PubkyAppUser::PATH_SEGMENT => Ok(Resource::User),
                s if s == PubkyAppLastRead::PATH_SEGMENT => Ok(Resource::LastRead),
                _ => Ok(Resource::Unknown),
            },
            // Two or more segments: the first is the resource type and the second is its id.
            [res_type, id, ..] => {
                // Since our constants for these routes include a trailing slash, re-add it.
                let path_segment = format!("{}/", res_type);
                match path_segment.as_str() {
                    PubkyAppPost::PATH_SEGMENT => Ok(Resource::Post(id.to_string())),
                    PubkyAppFollow::PATH_SEGMENT => PubkyId::try_from(*id).map(Resource::Follow),
                    PubkyAppMute::PATH_SEGMENT => PubkyId::try_from(*id).map(Resource::Mute),
                    PubkyAppBookmark::PATH_SEGMENT => Ok(Resource::Bookmark(id.to_string())),
                    PubkyAppTag::PATH_SEGMENT => Ok(Resource::Tag(id.to_string())),
                    PubkyAppFile::PATH_SEGMENT => Ok(Resource::File(id.to_string())),
                    PubkyAppBlob::PATH_SEGMENT => Ok(Resource::Blob(id.to_string())),
                    PubkyAppFeed::PATH_SEGMENT => Ok(Resource::Feed(id.to_string())),
                    _ => Ok(Resource::Unknown),
                }
            }
        }?;

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
            parsed_uri.resource,
            Resource::Unknown,
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
            Resource::Bookmark("00".to_string()),
            "The provided URI has wrong id"
        );
    }

    #[test]
    fn test_user() {
        let uri =
            "pubky://phbhg3qgcttn95guepmbud1nzcxhg3xc5j5k4h7i8a4b6wb3nw1o/pub/pubky.app/profile.json";
        let parsed_uri = ParsedUri::try_from(uri).unwrap_or_default();
        assert_eq!(
            parsed_uri.resource,
            Resource::User,
            "The provided URI is not user resource type"
        );
    }
}
