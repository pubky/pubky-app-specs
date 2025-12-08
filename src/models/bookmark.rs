use crate::{
    common::timestamp,
    traits::{HasIdPath, HashId, Validatable},
    APP_PATH, PUBLIC_PATH,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[cfg(target_arch = "wasm32")]
use crate::traits::Json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppBookmark {
    /// The URI of the resource this is a bookmark of
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
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

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppBookmark {
    /// Serialize to JSON for WASM.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = fromJson))]
    pub fn from_json(js_value: &JsValue) -> Result<Self, String> {
        Self::import_json(js_value)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = toJson))]
    pub fn to_json(&self) -> Result<JsValue, String> {
        self.export_json()
    }

    /// Getter for `uri`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn uri(&self) -> String {
        self.uri.clone()
    }
}

#[cfg(target_arch = "wasm32")]
impl Json for PubkyAppBookmark {}

impl HashId for PubkyAppBookmark {
    /// Bookmark ID is created based on the hash of the URI bookmarked.
    fn get_id_data(&self) -> String {
        self.uri.clone()
    }
}

impl HasIdPath for PubkyAppBookmark {
    const PATH_SEGMENT: &'static str = "bookmarks/";

    fn create_path(id: &str) -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT, id].concat()
    }
}

impl Validatable for PubkyAppBookmark {
    fn validate(&self, id: Option<&str>) -> Result<(), String> {
        // Validate the bookmark ID
        if let Some(id) = id {
            self.validate_id(id)?;
        }
        // Additional bookmark validation can be added here.

        // Validate URI format
        Url::parse(&self.uri)
            .map(|_| ())
            .map_err(|_| format!("Validation Error: Invalid URI format: {}", self.uri))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{post_uri_builder, traits::Validatable};

    #[test]
    fn test_create_bookmark_id() {
        let bookmark = PubkyAppBookmark {
            uri: post_uri_builder("user_id".into(), "post_id".into()),
            created_at: 1627849723,
        };

        let bookmark_id = bookmark.create_id();
        assert_eq!(bookmark_id, "2GN0JCHX9NYXPECQDS8KSMSE7M");
    }

    #[test]
    fn test_create_path() {
        let post_uri = post_uri_builder("user_id".into(), "post_id".into());
        let bookmark = PubkyAppBookmark {
            uri: post_uri,
            created_at: 1627849723,
        };
        let expected_id = bookmark.create_id();
        let expected_path = format!("{}{}bookmarks/{}", PUBLIC_PATH, APP_PATH, expected_id);
        let path = PubkyAppBookmark::create_path(&expected_id);
        assert_eq!(path, expected_path);
    }

    #[test]
    fn test_validate() {
        let post_uri = post_uri_builder("user_id".into(), "post_id".into());
        let bookmark = PubkyAppBookmark::new(post_uri);
        let id = bookmark.create_id();
        let result = bookmark.validate(Some(&id));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_id() {
        let post_uri = post_uri_builder("user_id".into(), "post_id".into());
        let bookmark = PubkyAppBookmark::new(post_uri);
        let invalid_id = "INVALIDID";
        let result = bookmark.validate(Some(invalid_id));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_uri() {
        let post_uri = "user_id/pub/pubky.app/posts/post_id".to_string();
        let bookmark = PubkyAppBookmark::new(post_uri);

        let id = bookmark.create_id();
        let res = bookmark.validate(Some(&id));
        assert!(res
            .unwrap_err()
            .starts_with("Validation Error: Invalid URI format"));
    }

    #[test]
    fn test_try_from_valid() {
        let bookmark_json = r#"
        {
            "uri": "pubky://user_id/pub/pubky.app/posts/post_id",
            "created_at": 1627849723
        }
        "#;

        let uri = post_uri_builder("user_id".into(), "post_id".into());
        let bookmark = PubkyAppBookmark::new(uri.clone());
        let id = bookmark.create_id();

        let blob = bookmark_json.as_bytes();
        let bookmark_parsed = <PubkyAppBookmark as Validatable>::try_from(blob, &id).unwrap();

        assert_eq!(bookmark_parsed.uri, uri);
    }
}
