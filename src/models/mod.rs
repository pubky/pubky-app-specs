use crate::{traits::Validatable, ParsedUri, Resource};

pub mod blob;
pub mod bookmark;
pub mod feed;
pub mod file;
pub mod follow;
pub mod last_read;
pub mod mute;
pub mod post;
pub mod tag;
pub mod user;

use super::{
    PubkyAppBlob, PubkyAppBookmark, PubkyAppFeed, PubkyAppFile, PubkyAppFollow, PubkyAppLastRead,
    PubkyAppMute, PubkyAppPost, PubkyAppTag, PubkyAppUser,
};

/// A unified enum wrapping all PubkyApp objects.
#[derive(Debug, Clone)]
pub enum PubkyAppObject {
    User(user::PubkyAppUser),
    Post(post::PubkyAppPost),
    Follow(follow::PubkyAppFollow),
    Mute(mute::PubkyAppMute),
    Bookmark(bookmark::PubkyAppBookmark),
    Tag(tag::PubkyAppTag),
    File(file::PubkyAppFile),
    Blob(blob::PubkyAppBlob),
    Feed(feed::PubkyAppFeed),
    LastRead(last_read::PubkyAppLastRead),
}

impl PubkyAppObject {
    /// Given a URI and a blob (raw data from the homeserver),
    /// this function returns the fully formed PubkyAppObject.
    pub fn from_uri(uri: &str, blob: &[u8]) -> Result<Self, String> {
        let parsed_uri = ParsedUri::try_from(uri)?;
        Self::from_resource(&parsed_uri.resource, blob)
    }

    /// Given a Resource and a blob (raw data from the homeserver),
    /// this function returns the fully formed PubkyAppObject.
    pub fn from_resource(resource: &Resource, blob: &[u8]) -> Result<Self, String> {
        match resource {
            Resource::User => {
                // For a user, no ID is needed (or you may use an empty string)
                let user = <PubkyAppUser as Validatable>::try_from(blob, "")?;
                Ok(PubkyAppObject::User(user))
            }
            Resource::Post(post_id) => {
                let post = <PubkyAppPost as Validatable>::try_from(blob, &post_id)?;
                Ok(PubkyAppObject::Post(post))
            }
            Resource::Follow(follow_id) => {
                // Use the follow id from the parsed URI.
                let follow = <PubkyAppFollow as Validatable>::try_from(blob, &follow_id)?;
                Ok(PubkyAppObject::Follow(follow))
            }
            Resource::Mute(muted_id) => {
                let mute = <PubkyAppMute as Validatable>::try_from(blob, &muted_id)?;
                Ok(PubkyAppObject::Mute(mute))
            }
            Resource::Bookmark(bookmark_id) => {
                let bookmark = <PubkyAppBookmark as Validatable>::try_from(blob, &bookmark_id)?;
                Ok(PubkyAppObject::Bookmark(bookmark))
            }
            Resource::Tag(tag_id) => {
                let tag = <PubkyAppTag as Validatable>::try_from(blob, &tag_id)?;
                Ok(PubkyAppObject::Tag(tag))
            }
            Resource::File(file_id) => {
                let file = <PubkyAppFile as Validatable>::try_from(blob, &file_id)?;
                Ok(PubkyAppObject::File(file))
            }
            Resource::Blob(blob_id) => {
                let blob_obj = <PubkyAppBlob as Validatable>::try_from(blob, &blob_id)?;
                Ok(PubkyAppObject::Blob(blob_obj))
            }
            Resource::Feed(feed_id) => {
                let feed = <PubkyAppFeed as Validatable>::try_from(blob, &feed_id)?;
                Ok(PubkyAppObject::Feed(feed))
            }
            Resource::LastRead => {
                let last_read = <PubkyAppLastRead as Validatable>::try_from(blob, "")?;
                Ok(PubkyAppObject::LastRead(last_read))
            }
            Resource::Unknown => Err(format!("Unrecognized resource {:?}", resource)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests assume that the respective try_from implementations for each model
    // parse the provided JSON. Adjust the JSON payloads as needed.

    #[test]
    fn test_import_user() {
        let uri = "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/profile.json";
        let user_json = r#"{
            "name": "Alice",
            "bio": "Hello, I am Alice",
            "image": "https://example.com/alice.png",
            "links": null,
            "status": "active"
        }"#;
        let result = PubkyAppObject::from_uri(uri, user_json.as_bytes());
        assert!(
            result.is_ok(),
            "Expected a successful import for user, got error: {:?}",
            result.err()
        );
        match result.unwrap() {
            PubkyAppObject::User(user) => {
                assert_eq!(user.name, "Alice", "User name mismatch");
                assert_eq!(
                    user.bio.unwrap_or_default(),
                    "Hello, I am Alice",
                    "User bio mismatch"
                );
            }
            other => panic!("Expected a User object, got {:?}", other),
        }
    }

    #[test]
    fn test_import_post() {
        let uri = "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/posts/0032SSN7Q4EVG";
        let post_json = r#"{
            "content": "Hello World!",
            "kind": "short",
            "parent": null,
            "embed": null,
            "attachments": null
        }"#;
        let result = PubkyAppObject::from_uri(uri, post_json.as_bytes());
        assert!(
            result.is_ok(),
            "Expected a successful import for post, got error: {:?}",
            result.err()
        );
        match result.unwrap() {
            PubkyAppObject::Post(post) => {
                assert_eq!(post.content, "Hello World!", "Post content mismatch");
            }
            other => panic!("Expected a Post object, got {:?}", other),
        }
    }

    #[test]
    fn test_import_follow() {
        let uri = "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/follows/pxnu33x7jtpx9ar1ytsi4yxbp6a5o36gwhffs8zoxmbuptici1jy";
        let follow_json = r#"{
            "created_at": 1627849723
        }"#;
        let result = PubkyAppObject::from_uri(uri, follow_json.as_bytes());
        assert!(
            result.is_ok(),
            "Expected a successful import for follow, got error: {:?}",
            result.err()
        );
        match result.unwrap() {
            PubkyAppObject::Follow(follow) => {
                assert_eq!(follow.created_at, 1627849723, "Follow created_at mismatch");
            }
            other => panic!("Expected a Follow object, got {:?}", other),
        }
    }

    #[test]
    fn test_import_mute() {
        let uri = "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/mutes/pxnu33x7jtpx9ar1ytsi4yxbp6a5o36gwhffs8zoxmbuptici1jy";
        let mute_json = r#"{
            "created_at": 1627849724
        }"#;
        let result = PubkyAppObject::from_uri(uri, mute_json.as_bytes());
        assert!(
            result.is_ok(),
            "Expected a successful import for mute, got error: {:?}",
            result.err()
        );
        match result.unwrap() {
            PubkyAppObject::Mute(mute) => {
                assert_eq!(mute.created_at, 1627849724, "Mute created_at mismatch");
            }
            other => panic!("Expected a Mute object, got {:?}", other),
        }
    }

    #[test]
    fn test_import_bookmark() {
        let uri = "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/bookmarks/8Z8CWH8NVYQY39ZEBFGKQWWEKG";
        let bookmark_json = r#"{
            "uri": "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/posts/0032SSN7Q4EVG",
            "created_at": 1627849725
        }"#;
        let result = PubkyAppObject::from_uri(uri, bookmark_json.as_bytes());
        assert!(
            result.is_ok(),
            "Expected a successful import for bookmark, got error: {:?}",
            result.err()
        );
        match result.unwrap() {
            PubkyAppObject::Bookmark(bookmark) => {
                assert_eq!(
                    bookmark.uri,
                    "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/posts/0032SSN7Q4EVG",
                    "Bookmark URI mismatch"
                );
            }
            other => panic!("Expected a Bookmark object, got {:?}", other),
        }
    }

    #[test]
    fn test_import_tag() {
        let uri = "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/tags/86805FC1CSFZD4W6HZ09S24QWG";
        let tag_json = r#"{
            "uri": "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/posts/0032SSN7Q4EVG",
            "label": "cool",
            "created_at": 1627849726
        }"#;
        let result = PubkyAppObject::from_uri(uri, tag_json.as_bytes());
        assert!(
            result.is_ok(),
            "Expected a successful import for tag, got error: {:?}",
            result.err()
        );
        match result.unwrap() {
            PubkyAppObject::Tag(tag) => {
                assert_eq!(tag.label, "cool", "Tag label mismatch");
            }
            other => panic!("Expected a Tag object, got {:?}", other),
        }
    }

    #[test]
    fn test_import_file() {
        let uri = "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/files/0032SSN7Q4EVG";
        let file_json = r#"{
            "name": "example.png",
            "created_at": 1627849727,
            "src": "https://example.com/example.png",
            "content_type": "image/png",
            "size": 1024
        }"#;
        let result = PubkyAppObject::from_uri(uri, file_json.as_bytes());
        assert!(
            result.is_ok(),
            "Expected a successful import for file, got error: {:?}",
            result.err()
        );
        match result.unwrap() {
            PubkyAppObject::File(file) => {
                assert_eq!(file.name, "example.png", "File name mismatch");
                assert_eq!(
                    file.src, "https://example.com/example.png",
                    "File src mismatch"
                );
            }
            other => panic!("Expected a File object, got {:?}", other),
        }
    }

    #[test]
    fn test_import_blob() {
        let uri = "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/blobs/8FACA7A6XMYH0GD9KBXMQ373PR";
        // For a blob, assume the JSON is an array of numbers representing the data.
        let blob_json = r#"[1,2,3,4]"#;
        let result = PubkyAppObject::from_uri(uri, blob_json.as_bytes());
        assert!(
            result.is_ok(),
            "Expected a successful import for blob, got error: {:?}",
            result.err()
        );
        match result.unwrap() {
            PubkyAppObject::Blob(blob_obj) => {
                let data = blob_obj.0;
                assert_eq!(data, vec![1, 2, 3, 4], "Blob data mismatch");
            }
            other => panic!("Expected a Blob object, got {:?}", other),
        }
    }

    #[test]
    fn test_import_feed() {
        let uri = "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/feeds/5F2NDB2HJGJ2HJBY6MPQ0H5R0G";
        let feed_json = r#"{
            "feed": {
                "tags": [],
                "reach": "following",
                "layout": "columns",
                "sort": "recent",
                "content": null
            },
            "name": "My Feed",
            "created_at": 1627849728
        }"#;
        let result = PubkyAppObject::from_uri(uri, feed_json.as_bytes());
        assert!(
            result.is_ok(),
            "Expected a successful import for feed, got error: {:?}",
            result.err()
        );
        match result.unwrap() {
            PubkyAppObject::Feed(feed) => {
                assert_eq!(feed.name, "My Feed", "Feed name mismatch");
            }
            other => panic!("Expected a Feed object, got {:?}", other),
        }
    }

    #[test]
    fn test_import_last_read() {
        let uri =
            "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/last_read";
        let last_read_json = r#"{
            "timestamp": 1627849729
        }"#;
        let result = PubkyAppObject::from_uri(uri, last_read_json.as_bytes());
        assert!(
            result.is_ok(),
            "Expected a successful import for last_read, got error: {:?}",
            result.err()
        );
        match result.unwrap() {
            PubkyAppObject::LastRead(last_read) => {
                assert_eq!(
                    last_read.timestamp, 1627849729,
                    "LastRead timestamp mismatch"
                );
            }
            other => panic!("Expected a LastRead object, got {:?}", other),
        }
    }

    #[test]
    fn test_import_unknown_resource() {
        let uri =
            "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/unknown/ID";
        let json = r#"{}"#;
        let result = PubkyAppObject::from_uri(uri, json.as_bytes());
        assert!(
            result.is_err(),
            "Expected an error for unknown resource, but got: {:?}",
            result.ok()
        );
        let err = result.err().unwrap();
        assert!(
            err.contains("Unrecognized resource"),
            "Error message does not contain expected text: {}",
            err
        );
    }
}
