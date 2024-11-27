use crate::traits::{HashId, Validatable};
use serde::{Deserialize, Serialize};

/// Represents raw homeserver bookmark with id
/// URI: /pub/pubky.app/bookmarks/:bookmark_id
///
/// Example URI:
///
/// `/pub/pubky.app/bookmarks/AF7KQ6NEV5XV1EG5DVJ2E74JJ4`
///
/// Where bookmark_id is Crockford-base32(Blake3("{uri_bookmarked}"")[:half])
#[derive(Serialize, Deserialize, Default)]
pub struct PubkyAppBookmark {
    uri: String,
    created_at: i64,
}

impl HashId for PubkyAppBookmark {
    /// Bookmark ID is created based on the hash of the URI bookmarked
    fn get_id_data(&self) -> String {
        self.uri.clone()
    }
}

impl Validatable for PubkyAppBookmark {
    fn validate(&self, id: &str) -> Result<(), String> {
        self.validate_id(id)?;
        // TODO: more bookmarks validation?
        Ok(())
    }
}

#[test]
fn test_create_bookmark_id() {
    let bookmark = PubkyAppBookmark {
        uri: "user_id/pub/pubky.app/posts/post_id".to_string(),
        created_at: 1627849723,
    };

    let bookmark_id = bookmark.create_id();
    println!("Generated Bookmark ID: {}", bookmark_id);
}
