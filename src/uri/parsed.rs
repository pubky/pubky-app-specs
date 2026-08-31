use crate::{
    traits::{HasIdPath, HasPath},
    PubkyId, PubkySocialBlob, PubkySocialBookmark, PubkySocialFeed, PubkySocialFile,
    PubkySocialFollow, PubkySocialMute, PubkySocialPost, PubkySocialTag, PubkySocialUser, APP_PATH,
    PROTOCOL,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use super::Resource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedUri {
    pub user_id: PubkyId,
    pub resource: Resource,
}

impl ParsedUri {
    /// Converts the [ParsedUri] back into its URI string representation.
    /// Returns an error if the resource is Unknown.
    pub fn try_to_uri_str(&self) -> Result<String, String> {
        let path = match &self.resource {
            Resource::User => PubkySocialUser::create_path(),
            Resource::Post(id) => PubkySocialPost::create_path(id),
            Resource::Follow(id) => PubkySocialFollow::create_path(id.as_ref()),
            Resource::Mute(id) => PubkySocialMute::create_path(id.as_ref()),
            Resource::Bookmark(id) => PubkySocialBookmark::create_path(id),
            Resource::Tag(id) => PubkySocialTag::create_path(id),
            Resource::File(id) => PubkySocialFile::create_path(id),
            Resource::Blob(id) => PubkySocialBlob::create_path(id),
            Resource::Feed(id) => PubkySocialFeed::create_path(id),
            Resource::Unknown => return Err("Cannot convert Unknown resource to URI".to_string()),
        };

        Ok([PROTOCOL, self.user_id.as_ref(), &path].concat())
    }
}

impl TryFrom<&str> for ParsedUri {
    type Error = String;
    fn try_from(uri: &str) -> Result<Self, Self::Error> {
        let path = super::path::try_parse_pubky_path(uri)?;

        if path.app != APP_PATH.trim_matches('/') {
            return Err(format!(
                "Expected app path '{}' but got '{}' in URI: {}",
                APP_PATH.trim_matches('/'),
                path.app,
                uri
            ));
        }

        let resource = resource_from_segments(&path.segments)?;

        Ok(ParsedUri {
            user_id: path.user_id,
            resource,
        })
    }
}

fn resource_from_segments(segments: &[String]) -> Result<Resource, String> {
    match segments {
        [] => Ok(Resource::Unknown),
        [segment] => Ok(match segment.as_str() {
            s if s == PubkySocialUser::PATH_SEGMENT.trim_end_matches('/') => Resource::User,
            _ => Resource::Unknown,
        }),
        [res_type, id, ..] if !id.is_empty() => {
            let resource_type = format!("{res_type}/");
            Ok(match resource_type.as_str() {
                PubkySocialPost::PATH_SEGMENT => Resource::Post(id.clone()),
                PubkySocialFollow::PATH_SEGMENT => {
                    PubkyId::try_from(id.as_str()).map(Resource::Follow)?
                }
                PubkySocialMute::PATH_SEGMENT => {
                    PubkyId::try_from(id.as_str()).map(Resource::Mute)?
                }
                PubkySocialBookmark::PATH_SEGMENT => Resource::Bookmark(id.clone()),
                PubkySocialTag::PATH_SEGMENT => Resource::Tag(id.clone()),
                PubkySocialFile::PATH_SEGMENT => Resource::File(id.clone()),
                PubkySocialBlob::PATH_SEGMENT => Resource::Blob(id.clone()),
                PubkySocialFeed::PATH_SEGMENT => Resource::Feed(id.clone()),
                _ => Resource::Unknown,
            })
        }
        _ => Ok(Resource::Unknown),
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
    use crate::{
        blob_uri_builder, bookmark_uri_builder, feed_uri_builder, file_uri_builder,
        follow_uri_builder, mute_uri_builder, post_uri_builder, tag_uri_builder, user_uri_builder,
    };

    use super::*;

    const USER_ID: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";

    #[test]
    fn test_empty_bookmark_uri() {
        let uri = bookmark_uri_builder(USER_ID.into(), "".into());
        let parsed_uri =
            ParsedUri::try_from(uri).expect("empty bookmark id should parse to Unknown");
        assert_eq!(
            parsed_uri.resource,
            Resource::Unknown,
            "The provided URI has bookmark_id"
        );
    }

    #[test]
    fn test_some_bookmark_uri() {
        let uri = bookmark_uri_builder(USER_ID.into(), "00".into());
        let parsed_uri = ParsedUri::try_from(uri).expect("bookmark id should parse");
        assert_eq!(
            parsed_uri.resource,
            Resource::Bookmark("00".to_string()),
            "The provided URI has wrong id"
        );
    }

    #[test]
    fn test_user() {
        let uri = user_uri_builder(USER_ID.into());
        let parsed_uri = ParsedUri::try_from(uri).expect("user uri should parse");
        assert_eq!(
            parsed_uri.resource,
            Resource::User,
            "The provided URI is not user resource type"
        );
    }

    // Successful cases

    #[test]
    fn test_valid_user_uri() {
        let user_id = PubkyId::try_from(USER_ID).unwrap();

        // A valid user URI ends with profile.json.
        let uri = user_uri_builder(USER_ID.into());
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse valid user URI");
        assert_eq!(parsed.user_id, user_id);
        assert_eq!(parsed.resource, Resource::User);

        // Repeat same checks for ParsedUri derived directly from PubkyId
        let parsed_uri_from_pubky_id = user_id.to_uri();
        assert_eq!(parsed_uri_from_pubky_id.user_id, user_id);
        assert_eq!(parsed_uri_from_pubky_id.resource, Resource::User);
    }

    #[test]
    fn test_valid_post_uri() {
        // A valid post URI includes the posts/ segment followed by an identifier.
        let uri = post_uri_builder(USER_ID.into(), "0032SSN7Q4EVG".into());
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse valid post URI");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(parsed.resource, Resource::Post("0032SSN7Q4EVG".to_string()));
    }

    #[test]
    fn test_valid_follow_uri() {
        // A valid follow URI.
        let uri = follow_uri_builder(USER_ID.into(), USER_ID.into());
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
        let uri = bookmark_uri_builder(USER_ID.into(), bookmark_id.into());
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse valid bookmark URI");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(parsed.resource, Resource::Bookmark(bookmark_id.to_string()));
    }

    #[test]
    fn test_valid_tag_uri() {
        let uri = tag_uri_builder(USER_ID.into(), "8Z8CWH8NVYQY39ZEBFGKQWWEKG".into());
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse valid tag URI");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(
            parsed.resource,
            Resource::Tag("8Z8CWH8NVYQY39ZEBFGKQWWEKG".to_string())
        );
    }

    #[test]
    fn test_valid_file_uri() {
        let uri = file_uri_builder(USER_ID.into(), "file003".into());
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse valid file URI");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(parsed.resource, Resource::File("file003".to_string()));
    }

    #[test]
    fn test_valid_blob_uri() {
        let uri = blob_uri_builder(USER_ID.into(), "8Z8CWH8NVYQY39ZEBFGKQWWEKG".into());
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse valid blob URI");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(
            parsed.resource,
            Resource::Blob("8Z8CWH8NVYQY39ZEBFGKQWWEKG".to_string())
        );
    }

    #[test]
    fn test_valid_feed_uri() {
        let uri = feed_uri_builder(USER_ID.into(), "8Z8CWH8NVYQY39ZEBFGKQWWEKG".into());
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
        let uri = format!("pubky://{USER_ID}/pub/pubky.app/");
        let parsed =
            ParsedUri::try_from(uri).expect("Failed to parse URI with no resource segments");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(parsed.resource, Resource::Unknown);
    }

    #[test]
    fn test_unknown_resource() {
        // Unknown resource type yields Resource::Unknown.
        let uri = format!("pubky://{USER_ID}/pub/pubky.app/unknown/xyz");
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse URI with unknown resource");
        assert_eq!(parsed.user_id, PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(parsed.resource, Resource::Unknown);
    }

    // Failure cases

    #[test]
    fn test_invalid_scheme() {
        let uri = format!("http://{USER_ID}/pub/pubky.app/profile.json");
        let result = ParsedUri::try_from(uri);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_public_path() {
        // Change the public path so it doesn't match.
        let uri = format!("pubky://{USER_ID}/invalid/pubky.app/profile.json");
        let result = ParsedUri::try_from(uri);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_app_path() {
        // Change the app path so it doesn't match.
        let uri = format!("pubky://{USER_ID}/pub/other.app/profile.json");
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

    // Reverse conversion tests: ParsedUri::try_to_uri_str should produce the same string as the builder functions

    #[test]
    fn test_user_uri_roundtrip() {
        let original_uri = user_uri_builder(USER_ID.into());
        let parsed = ParsedUri::try_from(original_uri.clone()).expect("Failed to parse user URI");
        let reconstructed_uri = parsed
            .try_to_uri_str()
            .expect("Failed to convert to URI string");
        assert_eq!(original_uri, reconstructed_uri, "User URI roundtrip failed");
    }

    #[test]
    fn test_post_uri_roundtrip() {
        let post_id = "0032SSN7Q4EVG";
        let original_uri = post_uri_builder(USER_ID.into(), post_id.into());
        let parsed = ParsedUri::try_from(original_uri.clone()).expect("Failed to parse post URI");
        let reconstructed_uri = parsed
            .try_to_uri_str()
            .expect("Failed to convert to URI string");
        assert_eq!(original_uri, reconstructed_uri, "Post URI roundtrip failed");
    }

    #[test]
    fn test_follow_uri_roundtrip() {
        let original_uri = follow_uri_builder(USER_ID.into(), USER_ID.into());
        let parsed = ParsedUri::try_from(original_uri.clone()).expect("Failed to parse follow URI");
        let reconstructed_uri = parsed
            .try_to_uri_str()
            .expect("Failed to convert to URI string");
        assert_eq!(
            original_uri, reconstructed_uri,
            "Follow URI roundtrip failed"
        );
    }

    #[test]
    fn test_mute_uri_roundtrip() {
        let original_uri = mute_uri_builder(USER_ID.into(), USER_ID.into());
        let parsed = ParsedUri::try_from(original_uri.clone()).expect("Failed to parse mute URI");
        let reconstructed_uri = parsed
            .try_to_uri_str()
            .expect("Failed to convert to URI string");
        assert_eq!(original_uri, reconstructed_uri, "Mute URI roundtrip failed");
    }

    #[test]
    fn test_bookmark_uri_roundtrip() {
        let bookmark_id = "8Z8CWH8NVYQY39ZEBFGKQWWEKG";
        let original_uri = bookmark_uri_builder(USER_ID.into(), bookmark_id.into());
        let parsed =
            ParsedUri::try_from(original_uri.clone()).expect("Failed to parse bookmark URI");
        let reconstructed_uri = parsed
            .try_to_uri_str()
            .expect("Failed to convert to URI string");
        assert_eq!(
            original_uri, reconstructed_uri,
            "Bookmark URI roundtrip failed"
        );
    }

    #[test]
    fn test_tag_uri_roundtrip() {
        let tag_id = "8Z8CWH8NVYQY39ZEBFGKQWWEKG";
        let original_uri = tag_uri_builder(USER_ID.into(), tag_id.into());
        let parsed = ParsedUri::try_from(original_uri.clone()).expect("Failed to parse tag URI");
        let reconstructed_uri = parsed
            .try_to_uri_str()
            .expect("Failed to convert to URI string");
        assert_eq!(original_uri, reconstructed_uri, "Tag URI roundtrip failed");
    }

    #[test]
    fn test_file_uri_roundtrip() {
        let file_id = "file003";
        let original_uri = file_uri_builder(USER_ID.into(), file_id.into());
        let parsed = ParsedUri::try_from(original_uri.clone()).expect("Failed to parse file URI");
        let reconstructed_uri = parsed
            .try_to_uri_str()
            .expect("Failed to convert to URI string");
        assert_eq!(original_uri, reconstructed_uri, "File URI roundtrip failed");
    }

    #[test]
    fn test_blob_uri_roundtrip() {
        let blob_id = "8Z8CWH8NVYQY39ZEBFGKQWWEKG";
        let original_uri = blob_uri_builder(USER_ID.into(), blob_id.into());
        let parsed = ParsedUri::try_from(original_uri.clone()).expect("Failed to parse blob URI");
        let reconstructed_uri = parsed
            .try_to_uri_str()
            .expect("Failed to convert to URI string");
        assert_eq!(original_uri, reconstructed_uri, "Blob URI roundtrip failed");
    }

    #[test]
    fn test_feed_uri_roundtrip() {
        let feed_id = "8Z8CWH8NVYQY39ZEBFGKQWWEKG";
        let original_uri = feed_uri_builder(USER_ID.into(), feed_id.into());
        let parsed = ParsedUri::try_from(original_uri.clone()).expect("Failed to parse feed URI");
        let reconstructed_uri = parsed
            .try_to_uri_str()
            .expect("Failed to convert to URI string");
        assert_eq!(original_uri, reconstructed_uri, "Feed URI roundtrip failed");
    }

    #[test]
    fn test_unknown_resource_to_uri_str_fails() {
        let uri = format!("pubky://{USER_ID}/pub/pubky.app/unknown/xyz");
        let parsed = ParsedUri::try_from(uri).expect("Failed to parse URI with unknown resource");
        assert_eq!(parsed.resource, Resource::Unknown);
        let result = parsed.try_to_uri_str();
        assert!(
            result.is_err(),
            "Unknown resource should fail to convert to URI string"
        );
    }
}
