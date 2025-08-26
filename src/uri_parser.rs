use crate::{
    traits::{HasIdPath, HasPath},
    PubkyAppBlob, PubkyAppBookmark, PubkyAppFeed, PubkyAppFile, PubkyAppFollow, PubkyAppLastRead,
    PubkyAppMute, PubkyAppPost, PubkyAppTag, PubkyAppUser, PubkyId, APP_PATH, PROTOCOL,
    PUBLIC_PATH,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use url::Url;

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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ParsedUri {
    pub user_id: PubkyId,
    pub resource: Resource,
}

impl TryFrom<&str> for ParsedUri {
    type Error = String;
    fn try_from(uri: &str) -> Result<Self, Self::Error> {
        // 0. Validate and sanitize the URL.
        let parsed_url = Url::parse(uri).map_err(|e| format!("Invalid URL: {}", e))?;

        // 1. Validate the scheme (using PROTOCOL without the "://")
        if parsed_url.scheme() != PROTOCOL.trim_end_matches("://") {
            return Err(format!(
                "Invalid URI, must start with '{}': {}",
                PROTOCOL, uri
            ));
        }

        // 2. Extract the user_id from the host.
        let user_id_str = parsed_url
            .host_str()
            .ok_or_else(|| format!("Missing user ID in URI: {}", uri))?;
        let user_id = PubkyId::try_from(user_id_str)?;

        // 3. Get the path segments.
        // Expected URI structure:
        // pubky://<user_id>/<public>/<app>/<resource>[/<id>]
        let segments: Vec<&str> = parsed_url
            .path_segments()
            .ok_or_else(|| format!("Cannot parse path segments from URI: {}", uri))?
            .collect();
        if segments.len() < 2 {
            return Err(format!("Not enough path segments in URI: {}", uri));
        }
        if segments[0] != PUBLIC_PATH.trim_matches('/') {
            return Err(format!(
                "Expected public path '{}' but got '{}' in URI: {}",
                PUBLIC_PATH, segments[0], uri
            ));
        }
        if segments[1] != APP_PATH.trim_matches('/') {
            return Err(format!(
                "Expected app path '{}' but got '{}' in URI: {}",
                APP_PATH, segments[1], uri
            ));
        }

        // 4. Determine the resource from the remaining segments.
        let resource = match segments[2..] {
            // No extra segments.
            [] => Resource::Unknown,
            // A single segment: must exactly match an identifier-less route.
            [segment] => match segment {
                PubkyAppUser::PATH_SEGMENT => Resource::User,
                PubkyAppLastRead::PATH_SEGMENT => Resource::LastRead,
                _ => Resource::Unknown,
            },
            // Two or more segments and the id is not empty.
            [res_type, id, ..] if !id.is_empty() => {
                let resource_type = format!("{}/", res_type);
                match resource_type.as_str() {
                    PubkyAppPost::PATH_SEGMENT => Resource::Post(id.to_string()),
                    PubkyAppFollow::PATH_SEGMENT => PubkyId::try_from(id).map(Resource::Follow)?,
                    PubkyAppMute::PATH_SEGMENT => PubkyId::try_from(id).map(Resource::Mute)?,
                    PubkyAppBookmark::PATH_SEGMENT => Resource::Bookmark(id.to_string()),
                    PubkyAppTag::PATH_SEGMENT => Resource::Tag(id.to_string()),
                    PubkyAppFile::PATH_SEGMENT => Resource::File(id.to_string()),
                    PubkyAppBlob::PATH_SEGMENT => Resource::Blob(id.to_string()),
                    PubkyAppFeed::PATH_SEGMENT => Resource::Feed(id.to_string()),
                    _ => Resource::Unknown,
                }
            }
            // If the identifier is empty or doesn't match the expected pattern.
            _ => Resource::Unknown,
        };

        Ok(ParsedUri { user_id, resource })
    }
}

impl TryFrom<String> for ParsedUri {
    type Error = String;

    fn try_from(uri: String) -> Result<Self, Self::Error> {
        ParsedUri::try_from(uri.as_str())
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::*;

    use super::*;

    const USER_ID: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";

    #[test]
    fn test_empty_bookmark_uri() {
        let uri = bookmark_uri_builder(
            "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".into(),
            "".into(),
        );
        let parsed_uri = ParsedUri::try_from(uri).unwrap_or_default();
        assert_eq!(
            parsed_uri.resource,
            Resource::Unknown,
            "The provided URI has bookmark_id"
        );
    }

    #[test]
    fn test_some_bookmark_uri() {
        let uri = bookmark_uri_builder(
            "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".into(),
            "00".into(),
        );
        let parsed_uri = ParsedUri::try_from(uri).unwrap_or_default();
        assert_eq!(
            parsed_uri.resource,
            Resource::Bookmark("00".to_string()),
            "The provided URI has wrong id"
        );
    }

    #[test]
    fn test_user() {
        let uri = user_uri_builder("operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".into());
        let parsed_uri = ParsedUri::try_from(uri).unwrap_or_default();
        assert_eq!(
            parsed_uri.resource,
            Resource::User,
            "The provided URI is not user resource type"
        );
    }

    // Successful cases

    #[test]
    fn test_valid_user_uri() {
        // A valid user URI ends with profile.json.
        let uri = user_uri_builder("operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".into());
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse valid user URI");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(parsed.resource, Resource::User);
    }

    #[test]
    fn test_valid_last_read_uri() {
        // A valid last_read URI ends with last_read.
        let uri =
            "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/last_read";
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse valid last_read URI");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(parsed.resource, Resource::LastRead);
    }

    #[test]
    fn test_valid_post_uri() {
        // A valid post URI includes the posts/ segment followed by an identifier.
        let uri = post_uri_builder(
            "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".into(),
            "0032SSN7Q4EVG".into(),
        );
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse valid post URI");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(parsed.resource, Resource::Post("0032SSN7Q4EVG".to_string()));
    }

    #[test]
    fn test_valid_follow_uri() {
        // A valid follow URI.
        // TODO Is this a valid Follow URI, where the author follows themselves?
        let uri = follow_uri_builder(
            "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".into(),
            "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".into(),
        );
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse valid follow URI");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        // Assuming PubkyId::try_from("def456") returns a PubkyId that equals PubkyId::try_from("def456")
        assert_eq!(
            parsed.resource,
            Resource::Follow(PubkyId::try_from(USER_ID).unwrap())
        );
    }

    #[test]
    fn test_valid_bookmark_uri() {
        let bookmark_id = "8Z8CWH8NVYQY39ZEBFGKQWWEKG";
        let uri = bookmark_uri_builder(
            "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".into(),
            bookmark_id.into(),
        );
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse valid bookmark URI");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(parsed.resource, Resource::Bookmark(bookmark_id.to_string()));
    }

    #[test]
    fn test_valid_tag_uri() {
        let uri = tag_uri_builder(
            "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".into(),
            "8Z8CWH8NVYQY39ZEBFGKQWWEKG".into(),
        );
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse valid tag URI");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(
            parsed.resource,
            Resource::Tag("8Z8CWH8NVYQY39ZEBFGKQWWEKG".to_string())
        );
    }

    #[test]
    fn test_valid_file_uri() {
        let uri = file_uri_builder(
            "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".into(),
            "file003".into(),
        );
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse valid file URI");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(parsed.resource, Resource::File("file003".to_string()));
    }

    #[test]
    fn test_valid_blob_uri() {
        let uri = "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/blobs/8Z8CWH8NVYQY39ZEBFGKQWWEKG";
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse valid blob URI");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(
            parsed.resource,
            Resource::Blob("8Z8CWH8NVYQY39ZEBFGKQWWEKG".to_string())
        );
    }

    #[test]
    fn test_valid_feed_uri() {
        let uri = "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/feeds/8Z8CWH8NVYQY39ZEBFGKQWWEKG";
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse valid feed URI");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(
            parsed.resource,
            Resource::Feed("8Z8CWH8NVYQY39ZEBFGKQWWEKG".to_string())
        );
    }

    #[test]
    fn test_no_resource_segments() {
        // When there are no segments beyond the public and app paths,
        // the resource should be Unknown.
        let uri = "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/";
        let parsed =
            ParsedUri::try_from(uri).expect("Failed to parse URI with no resource segments");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(parsed.resource, Resource::Unknown);
    }

    #[test]
    fn test_unknown_resource() {
        // Unknown resource type yields Resource::Unknown.
        let uri = "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/unknown/xyz";
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse URI with unknown resource");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(parsed.resource, Resource::Unknown);
    }

    // Failure cases

    #[test]
    fn test_invalid_scheme() {
        let uri = "http://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/profile.json";
        let result = ParsedUri::try_from(uri);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_public_path() {
        // Change the public path so it doesn't match.
        let uri = "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/invalid/pubky.app/profile.json";
        let result = ParsedUri::try_from(uri);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_app_path() {
        // Change the app path so it doesn't match.
        let uri = "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/other.app/profile.json";
        let result = ParsedUri::try_from(uri);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_host() {
        // URL with missing host.
        let uri = "pubky:///pub/pubky.app/profile.json";
        let result = ParsedUri::try_from(uri);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_url() {
        let uri = "not a url";
        let result = ParsedUri::try_from(uri);
        assert!(result.is_err());
    }
}
