use crate::{
    common::timestamp,
    traits::{HasIdPath, HashId, Validatable},
    APP_PATH, PUBLIC_PATH,
};
use serde::{Deserialize, Serialize};
use url::Url;

// Validation
const MAX_TAG_LABEL_LENGTH: usize = 20;
const MIN_TAG_LABEL_LENGTH: usize = 1;
/// Disallowed characters, in addition to whitespace chars
const INVALID_CHARS: &[char] = &[',', ':'];

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
    /// The URI of the resource this is a tag on
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

impl HasIdPath for PubkyAppTag {
    const PATH_SEGMENT: &'static str = "tags/";

    fn create_path(id: &str) -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT, id].concat()
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
        // Sanitize label
        let label = self.label.to_lowercase();

        // Sanitize URI
        let uri = match Url::parse(&self.uri) {
            // If the URL is valid, reformat it to a sanitized string representation
            Ok(url) => url.to_string(),
            // If the URL is invalid, return as-is for error reporting later
            Err(_) => self.uri.trim().to_string(),
        };

        PubkyAppTag {
            uri,
            label,
            created_at: self.created_at,
        }
    }

    fn validate(&self, id: Option<&str>) -> Result<(), String> {
        // Validate the tag ID
        if let Some(id) = id {
            self.validate_id(id)?;
        }

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

        // Validate label chars: dissallow whitespace (space char, tab, newline, etc)
        if self.label.chars().any(|c| c.is_whitespace()) {
            return Err("Validation Error: Tag label has whitespace char".to_string());
        }

        // Validate label chars: dissallow INVALID_CHARS
        if let Some(c) = self.label.chars().find(|c| INVALID_CHARS.contains(c)) {
            return Err(format!("Validation Error: Tag label has invalid char: {c}"));
        }

        // Validate URI format
        Url::parse(&self.uri)
            .map(|_| ())
            .map_err(|_| format!("Validation Error: Invalid URI format: {}", self.uri))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{post_uri_builder, traits::Validatable, user_uri_builder, APP_PATH};

    #[test]
    fn test_label_id() {
        let post_uri = post_uri_builder("user_id".into(), "post_id".into());
        // Precomputed earlier
        let tag_id = "CBYS8P6VJPHC5XXT4WDW26662W";
        // Create new tag
        let tag = PubkyAppTag {
            uri: post_uri.clone(),
            created_at: 1627849723,
            label: "cool".to_string(),
        };

        let new_tag_id = tag.create_id();
        assert!(!tag_id.is_empty());

        // Check if the tag ID is correct
        assert_eq!(new_tag_id, tag_id);

        let wrong_tag = PubkyAppTag {
            uri: post_uri,
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
        let post_uri = post_uri_builder(
            "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".into(),
            "0032FNCGXE3R0".into(),
        );
        let tag = PubkyAppTag {
            uri: post_uri,
            created_at: 1627849723000,
            label: "cool".to_string(),
        };

        let expected_id = tag.create_id();
        let expected_path = format!("{}{}tags/{}", PUBLIC_PATH, APP_PATH, expected_id);
        let path = PubkyAppTag::create_path(&expected_id);

        assert_eq!(path, expected_path);
    }

    #[test]
    fn test_sanitize() {
        let post_uri = post_uri_builder("user_id".into(), "0000000000000".into());
        let tag = PubkyAppTag {
            uri: post_uri,
            label: "CoOl".to_string(),
            created_at: 1627849723000,
        };

        let sanitized_tag = tag.sanitize();
        assert_eq!(sanitized_tag.label, "cool");
    }

    #[test]
    fn test_validate_valid() {
        let post_uri = post_uri_builder("user_id".into(), "0000000000000".into());
        let tag = PubkyAppTag {
            uri: post_uri,
            label: "cool".to_string(),
            created_at: 1627849723000,
        };

        let id = tag.create_id();
        let result = tag.validate(Some(&id));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_label_length() {
        let post_uri = post_uri_builder("user_id".into(), "0000000000000".into());
        let tag = PubkyAppTag {
            uri: post_uri,
            label: "a".repeat(MAX_TAG_LABEL_LENGTH + 1),
            created_at: 1627849723000,
        };

        let id = tag.create_id();
        let result = tag.validate(Some(&id));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Validation Error: Tag label exceeds maximum length"
        );
    }

    #[test]
    fn test_validate_invalid_id() {
        let post_uri = post_uri_builder("user_id".into(), "0000000000000".into());
        let tag = PubkyAppTag {
            uri: post_uri,
            label: "cool".to_string(),
            created_at: 1627849723000,
        };

        let invalid_id = "INVALIDID";
        let result = tag.validate(Some(invalid_id));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_char() {
        let post_uri = post_uri_builder("user_id".into(), "0000000000000".into());
        let tag = PubkyAppTag {
            uri: post_uri,
            label: format!("invalidchar{}", INVALID_CHARS[0]),
            created_at: 1627849723000,
        };

        let id = tag.create_id();
        let result = tag.validate(Some(&id));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_uri() {
        let tag = PubkyAppTag {
            uri: "user_id/pub/pubky.app/posts/post_id".into(),
            label: "cool".to_string(),
            created_at: 1627849723000,
        };

        let id = tag.create_id();
        let result = tag.validate(Some(&id));
        assert!(result
            .unwrap_err()
            .starts_with("Validation Error: Invalid URI format"));
    }

    #[test]
    fn test_try_from_valid() {
        let user_uri = user_uri_builder("user_pubky_id".into());
        let tag_label = "CoolTag".to_string();
        let tag_json = format!(
            r#"
            {{
                "uri": "{user_uri}",
                "label": "{tag_label}",
                "created_at": 1627849723000
            }}
        "#
        );

        let id = PubkyAppTag::new(user_uri.clone(), tag_label.clone()).create_id();

        let blob = tag_json.as_bytes();
        let sanitized_validated_tag = <PubkyAppTag as Validatable>::try_from(blob, &id).unwrap();
        assert_eq!(sanitized_validated_tag.uri, user_uri);
        assert_eq!(sanitized_validated_tag.label, "cooltag");
    }

    #[test]
    fn test_try_from_invalid_uri() {
        let tag_json = r#"
        {
            "uri": "invalid_uri",
            "label": "CoolTag",
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
            uri: post_uri_builder("user_id".into(), "post_id".into()),
            created_at: 1627849723,
            label: "cool".to_string(),
        };
        let tag_id = tag.create_id();

        if let Err(e) = tag.validate(Some(&tag_id)) {
            assert_eq!(
                e.to_string(),
                format!("Validation Error: Invalid URI format: {}", tag.uri),
                "The error message is not related URI or the message description is wrong"
            )
        };

        let tag = PubkyAppTag {
            uri: post_uri_builder("user_id".into(), "post_id".into()),
            created_at: 1627849723,
            label: "coolc00lcolaca0g00llooll".to_string(),
        };

        // Precomputed earlier
        let label_id = tag.create_id();

        if let Err(e) = tag.validate(Some(&label_id)) {
            assert_eq!(
                e.to_string(),
                "Validation Error: Tag label exceeds maximum length".to_string(),
                "The error message is not related tag length or the message description is wrong"
            )
        };
    }

    #[test]
    fn test_white_space_tag() {
        let leading_whitespace = PubkyAppTag {
            uri: post_uri_builder("user_id".into(), "post_id".into()),
            created_at: 1627849723,
            label: " cool".to_string(),
        };
        let leading_whitespace_validate_res = leading_whitespace.validate(None);
        assert!(leading_whitespace_validate_res.is_err());

        let trailing_whitespace = PubkyAppTag {
            uri: post_uri_builder("user_id".into(), "post_id".into()),
            created_at: 1627849723,
            label: "cool ".to_string(),
        };
        let trailing_whitespace_validate_res = trailing_whitespace.validate(None);
        assert!(trailing_whitespace_validate_res.is_err());

        let trailing_newline = PubkyAppTag {
            uri: post_uri_builder("user_id".into(), "post_id".into()),
            created_at: 1627849723,
            label: "cool\n".to_string(),
        };
        let trailing_newline_validate_res = trailing_newline.validate(None);
        assert!(trailing_newline_validate_res.is_err());

        let space_between = PubkyAppTag {
            uri: post_uri_builder("user_id".into(), "post_id".into()),
            created_at: 1627849723,
            label: "   co ol ".to_string(),
        };
        let space_between_validate_res = space_between.validate(None);
        assert!(space_between_validate_res.is_err());
    }
}
