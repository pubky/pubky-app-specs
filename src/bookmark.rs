use crate::{
    common::timestamp,
    traits::{HasPath, HashId, Validatable},
    APP_PATH,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Represents raw homeserver bookmark with id
/// URI: /pub/pubky.app/bookmarks/:bookmark_id
///
/// Example URI:
///
/// `/pub/pubky.app/bookmarks/AF7KQ6NEV5XV1EG5DVJ2E74JJ4`
///
/// Where bookmark_id is Crockford-base32(Blake3("{uri_bookmarked}"")[:half])
#[derive(Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppBookmark {
    pub uri: String,
    pub created_at: i64,
}

impl PubkyAppBookmark {
    /// Creates a new `PubkyAppBookmark` instance.
    pub fn new(uri: String) -> Self {
        let created_at = timestamp();
        Self { uri, created_at }.sanitize()
    }
}

impl HashId for PubkyAppBookmark {
    /// Bookmark ID is created based on the hash of the URI bookmarked
    fn get_id_data(&self) -> String {
        self.uri.clone()
    }
}

impl HasPath for PubkyAppBookmark {
    fn create_path(&self) -> String {
        format!("{}bookmarks/{}", APP_PATH, self.create_id())
    }
}

impl Validatable for PubkyAppBookmark {
    fn validate(&self, id: &str) -> Result<(), String> {
        self.validate_id(id)?;
        // TODO: more bookmarks validation?
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Validatable;

    #[test]
    fn test_create_bookmark_id() {
        let bookmark = PubkyAppBookmark {
            uri: "user_id/pub/pubky.app/posts/post_id".to_string(),
            created_at: 1627849723,
        };

        let bookmark_id = bookmark.create_id();
        assert_eq!(bookmark_id, "AF7KQ6NEV5XV1EG5DVJ2E74JJ4");
    }

    #[test]
    fn test_create_path() {
        let bookmark = PubkyAppBookmark {
            uri: "pubky://user_id/pub/pubky.app/posts/post_id".to_string(),
            created_at: 1627849723,
        };
        let expected_id = bookmark.create_id();
        let expected_path = format!("{}bookmarks/{}", APP_PATH, expected_id);
        let path = bookmark.create_path();
        assert_eq!(path, expected_path);
    }

    #[test]
    fn test_validate_valid() {
        let bookmark =
            PubkyAppBookmark::new("pubky://user_id/pub/pubky.app/posts/post_id".to_string());
        let id = bookmark.create_id();
        let result = bookmark.validate(&id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_id() {
        let bookmark = PubkyAppBookmark::new("user_id/pub/pubky.app/posts/post_id".to_string());
        let invalid_id = "INVALIDID";
        let result = bookmark.validate(invalid_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_valid() {
        let bookmark_json = r#"
        {
            "uri": "user_id/pub/pubky.app/posts/post_id",
            "created_at": 1627849723
        }
        "#;

        let uri = "user_id/pub/pubky.app/posts/post_id".to_string();
        let bookmark = PubkyAppBookmark::new(uri.clone());
        let id = bookmark.create_id();

        let blob = bookmark_json.as_bytes();
        let bookmark_parsed = <PubkyAppBookmark as Validatable>::try_from(blob, &id).unwrap();

        assert_eq!(bookmark_parsed.uri, uri);
    }
}
