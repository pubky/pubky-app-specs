use crate::{
    common::timestamp,
    traits::{HasPath, HashId, Validatable},
    APP_PATH, PUBLIC_PATH,
};
use serde::{Deserialize, Serialize};
use url::Url;

// Validation
const MAX_TAG_LABEL_LENGTH: usize = 20;
const MIN_TAG_LABEL_LENGTH: usize = 1;

#[cfg(target_arch = "wasm32")]
use crate::traits::Json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Represents raw homeserver tag with id
/// URI: /pub/pubky.app/tags/:tag_id
///
/// Example URI:
///
/// `/pub/pubky.app/tags/FPB0AM9S93Q3M1GFY1KV09GMQM`
///
/// Where tag_id is Crockford-base32(Blake3("{uri_tagged}:{label}")[:half])
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppTag {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub uri: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub label: String,
    pub created_at: i64,
}

impl PubkyAppTag {
    pub fn new(uri: String, label: String) -> Self {
        let created_at = timestamp();
        Self {
            uri,
            label,
            created_at,
        }
        .sanitize()
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppTag {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = fromJson))]
    pub fn from_json(js_value: &JsValue) -> Result<Self, String> {
        Self::import_json(js_value)
    }

    /// Serialize to JSON for WASM.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = toJson))]
    pub fn to_json(&self) -> Result<JsValue, String> {
        self.export_json()
    }

    /// Getter for `uri`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn uri(&self) -> String {
        self.uri.clone()
    }

    /// Getter for `label`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn label(&self) -> String {
        self.label.clone()
    }
}

#[cfg(target_arch = "wasm32")]
impl Json for PubkyAppTag {}

impl HasPath for PubkyAppTag {
    const PATH_SEGMENT: &'static str = "tags/";

    fn create_path(&self) -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT, &self.create_id()].concat()
    }
}

impl HashId for PubkyAppTag {
    /// Tag ID is created based on the hash of the URI tagged and the label used
    fn get_id_data(&self) -> String {
        format!("{}:{}", self.uri, self.label)
    }
}

impl Validatable for PubkyAppTag {
    fn sanitize(self) -> Self {
        // Remove spaces from the tag and keep it as one word
        // Returns a lowercase tag
        let mut label = self
            .label
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .to_lowercase();

        // Enforce maximum label length safely
        label = label.chars().take(MAX_TAG_LABEL_LENGTH).collect::<String>();

        // Sanitize URI
        let uri = match Url::parse(&self.uri) {
            Ok(url) => {
                // If the URL is valid, reformat it to a sanitized string representation
                url.to_string()
            }
            Err(_) => {
                // If the URL is invalid, return as-is for error reporting later
                self.uri.trim().to_string()
            }
        };

        PubkyAppTag {
            uri,
            label,
            created_at: self.created_at,
        }
    }

    fn validate(&self, id: &str) -> Result<(), String> {
        // Validate the tag ID
        self.validate_id(id)?;

        // Validate label length
        match self.label.chars().count() {
            len if len > MAX_TAG_LABEL_LENGTH => {
                return Err("Validation Error: Tag label exceeds maximum length".to_string())
            }
            len if len < MIN_TAG_LABEL_LENGTH => {
                return Err("Validation Error: Tag label is shorter than minimum length".to_string())
            }
            _ => (),
        };

        // Validate URI format
        match Url::parse(&self.uri) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!(
                "Validation Error: Invalid URI format: {}",
                self.uri
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{traits::Validatable, APP_PATH};

    #[test]
    fn test_label_id() {
        // Precomputed earlier
        let tag_id = "CBYS8P6VJPHC5XXT4WDW26662W";
        // Create new tag
        let tag = PubkyAppTag {
            uri: "pubky://user_id/pub/pubky.app/posts/post_id".to_string(),
            created_at: 1627849723,
            label: "cool".to_string(),
        };

        let new_tag_id = tag.create_id();
        assert!(!tag_id.is_empty());

        // Check if the tag ID is correct
        assert_eq!(new_tag_id, tag_id);

        let wrong_tag = PubkyAppTag {
            uri: "pubky://user_id/pub/pubky.app/posts/post_id".to_string(),
            created_at: 1627849723,
            label: "co0l".to_string(),
        };

        // Assure that the new tag has wrong ID
        assert_ne!(wrong_tag.create_id(), tag_id);
    }

    #[test]
    fn test_create_id() {
        let tag = PubkyAppTag {
            uri: "https://example.com/post/1".to_string(),
            created_at: 1627849723000,
            label: "cool".to_string(),
        };

        let tag_id = tag.create_id();
        println!("Generated Tag ID: {}", tag_id);

        // Assert that the tag ID is of expected length
        // The length depends on your implementation of create_id
        assert!(!tag_id.is_empty());
    }

    #[test]
    fn test_new() {
        let uri = "https://example.com/post/1".to_string();
        let label = "interesting".to_string();
        let tag = PubkyAppTag::new(uri.clone(), label.clone());

        assert_eq!(tag.uri, uri);
        assert_eq!(tag.label, label);
        // Check that created_at is recent
        let now = timestamp();

        assert!(tag.created_at <= now && tag.created_at >= now - 1_000_000); // within 1 second
    }

    #[test]
    fn test_create_path() {
        let tag = PubkyAppTag {
            uri: "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/posts/0032FNCGXE3R0".to_string(),
            created_at: 1627849723000,
            label: "cool".to_string(),
        };

        let expected_id = tag.create_id();
        let expected_path = format!("{}{}tags/{}", PUBLIC_PATH, APP_PATH, expected_id);
        let path = tag.create_path();

        assert_eq!(path, expected_path);
    }

    #[test]
    fn test_sanitize() {
        let tag = PubkyAppTag {
            uri: "pubky://user_id/pub/pubky.app/posts/0000000000000".to_string(),
            label: "   CoOl  ".to_string(),
            created_at: 1627849723000,
        };

        let sanitized_tag = tag.sanitize();
        assert_eq!(sanitized_tag.label, "cool");
    }

    #[test]
    fn test_validate_valid() {
        let tag = PubkyAppTag {
            uri: "pubky://user_id/pub/pubky.app/posts/0000000000000".to_string(),
            label: "cool".to_string(),
            created_at: 1627849723000,
        };

        let id = tag.create_id();
        let result = tag.validate(&id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_label_length() {
        let tag = PubkyAppTag {
            uri: "pubky://user_id/pub/pubky.app/posts/0000000000000".to_string(),
            label: "a".repeat(MAX_TAG_LABEL_LENGTH + 1),
            created_at: 1627849723000,
        };

        let id = tag.create_id();
        let result = tag.validate(&id);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Validation Error: Tag label exceeds maximum length"
        );
    }

    #[test]
    fn test_validate_invalid_id() {
        let tag = PubkyAppTag {
            uri: "pubky://user_id/pub/pubky.app/posts/0000000000000".to_string(),
            label: "cool".to_string(),
            created_at: 1627849723000,
        };

        let invalid_id = "INVALIDID";
        let result = tag.validate(invalid_id);
        assert!(result.is_err());
        // You can check the specific error message if necessary
    }

    #[test]
    fn test_try_from_valid() {
        let tag_json = r#"
        {
            "uri": "pubky://user_pubky_id/pub/pubky.app/profile.json",
            "label": "Cool Tag",
            "created_at": 1627849723000
        }
        "#;

        let id = PubkyAppTag::new(
            "pubky://user_pubky_id/pub/pubky.app/profile.json".to_string(),
            "Cool Tag".to_string(),
        )
        .create_id();

        let blob = tag_json.as_bytes();
        let tag = <PubkyAppTag as Validatable>::try_from(blob, &id).unwrap();
        assert_eq!(tag.uri, "pubky://user_pubky_id/pub/pubky.app/profile.json");
        assert_eq!(tag.label, "cooltag"); // After sanitization
    }

    #[test]
    fn test_try_from_invalid_uri() {
        let tag_json = r#"
        {
            "uri": "invalid_uri",
            "label": "Cool Tag",
            "created_at": 1627849723000
        }
        "#;

        let id = "D2DV4EZDA03Q3KCRMVGMDYZ8C0";
        let blob = tag_json.as_bytes();
        let result = <PubkyAppTag as Validatable>::try_from(blob, id);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Validation Error: Invalid URI format: invalid_uri"
        );
    }

    #[test]
    fn test_incorrect_label() {
        let tag = PubkyAppTag {
            uri: "user_id/pub/pubky.app/posts/post_id".to_string(),
            created_at: 1627849723,
            label: "cool".to_string(),
        };
        let tag_id = tag.create_id();

        if let Err(e) = tag.validate(&tag_id) {
            assert_eq!(
                e.to_string(),
                format!("Validation Error: Invalid URI format: {}", tag.uri),
                "The error message is not related URI or the message description is wrong"
            )
        };

        let tag = PubkyAppTag {
            uri: "pubky://user_id/pub/pubky.app/posts/post_id".to_string(),
            created_at: 1627849723,
            label: "coolc00lcolaca0g00llooll".to_string(),
        };

        // Precomputed earlier
        let label_id = tag.create_id();

        if let Err(e) = tag.validate(&label_id) {
            assert_eq!(
                e.to_string(),
                "Validation Error: Tag label exceeds maximum length".to_string(),
                "The error message is not related tag length or the message description is wrong"
            )
        };
    }

    #[test]
    fn test_white_space_tag() {
        // All the tags has to be that label after sanitation
        let label = "cool";

        let leading_whitespace = PubkyAppTag {
            uri: "pubky://user_id/pub/pubky.app/posts/post_id".to_string(),
            created_at: 1627849723,
            label: " cool".to_string(),
        };
        let mut sanitazed_label = leading_whitespace.sanitize();
        assert_eq!(sanitazed_label.label, label);

        let trailing_whitespace = PubkyAppTag {
            uri: "pubky://user_id/pub/pubky.app/posts/post_id".to_string(),
            created_at: 1627849723,
            label: " cool".to_string(),
        };
        sanitazed_label = trailing_whitespace.sanitize();
        assert_eq!(sanitazed_label.label, label);

        let space_between = PubkyAppTag {
            uri: "pubky://user_id/pub/pubky.app/posts/post_id".to_string(),
            created_at: 1627849723,
            label: "   co ol ".to_string(),
        };
        sanitazed_label = space_between.sanitize();
        assert_eq!(sanitazed_label.label, "cool");
    }
}
