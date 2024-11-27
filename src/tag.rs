use crate::traits::{HasPath, HashId, Validatable};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use url::Url;

// Validation
const MAX_TAG_LABEL_LENGTH: usize = 20;

/// Represents raw homeserver tag with id
/// URI: /pub/pubky.app/tags/:tag_id
///
/// Example URI:
///
/// `/pub/pubky.app/tags/FPB0AM9S93Q3M1GFY1KV09GMQM`
///
/// Where tag_id is Crockford-base32(Blake3("{uri_tagged}:{label}")[:half])
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PubkyAppTag {
    pub uri: String,
    pub label: String,
    pub created_at: i64,
}

impl PubkyAppTag {
    pub fn new(uri: String, label: String) -> Self {
        let created_at = Utc::now().timestamp_millis();
        Self {
            uri,
            label,
            created_at,
        }
        .sanitize()
    }
}

impl HasPath for PubkyAppTag {
    fn get_path(&self) -> String {
        format!("pubky:///pub/pubky.app/tags/{}", self.create_id())
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
        // Convert label to lowercase and trim
        let label = self.label.trim().to_lowercase();

        // Enforce maximum label length safely
        let label = label.chars().take(MAX_TAG_LABEL_LENGTH).collect::<String>();

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
        if self.label.chars().count() > MAX_TAG_LABEL_LENGTH {
            return Err("Validation Error: Tag label exceeds maximum length".to_string());
        }

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
    use crate::traits::Validatable;
    use bytes::Bytes;

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
        let now = Utc::now().timestamp_millis();
        assert!(tag.created_at <= now && tag.created_at >= now - 1000); // within 1 second
    }

    #[test]
    fn test_get_path() {
        let tag = PubkyAppTag {
            uri: "https://example.com/post/1".to_string(),
            created_at: 1627849723000,
            label: "cool".to_string(),
        };

        let expected_id = tag.create_id();
        let expected_path = format!("pubky:///pub/pubky.app/tags/{}", expected_id);
        let path = tag.get_path();

        assert_eq!(path, expected_path);
    }

    #[test]
    fn test_sanitize() {
        let tag = PubkyAppTag {
            uri: "https://example.com/post/1".to_string(),
            label: "   CoOl  ".to_string(),
            created_at: 1627849723000,
        };

        let sanitized_tag = tag.sanitize();
        assert_eq!(sanitized_tag.label, "cool");
    }

    #[test]
    fn test_validate_valid() {
        let tag = PubkyAppTag {
            uri: "https://example.com/post/1".to_string(),
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
            uri: "https://example.com/post/1".to_string(),
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
            uri: "https://example.com/post/1".to_string(),
            label: "cool".to_string(),
            created_at: 1627849723000,
        };

        let invalid_id = "INVALIDID";
        let result = tag.validate(&invalid_id);
        assert!(result.is_err());
        // You can check the specific error message if necessary
    }

    #[test]
    fn test_try_from_valid() {
        let tag_json = r#"
        {
            "uri": "pubky://user_pubky_id/pub/pubky.app/v1/profile.json",
            "label": "Cool Tag",
            "created_at": 1627849723000
        }
        "#;

        let id = PubkyAppTag::new(
            "pubky://user_pubky_id/pub/pubky.app/v1/profile.json".to_string(),
            "Cool Tag".to_string(),
        )
        .create_id();

        let blob = Bytes::from(tag_json);
        let tag = <PubkyAppTag as Validatable>::try_from(&blob, &id).unwrap();
        assert_eq!(
            tag.uri,
            "pubky://user_pubky_id/pub/pubky.app/v1/profile.json"
        );
        assert_eq!(tag.label, "cool tag"); // After sanitization
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

        let id = "B55PGPFV1E5E0HQ2PB76EQGXPR";
        let blob = Bytes::from(tag_json);
        let result = <PubkyAppTag as Validatable>::try_from(&blob, &id);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Validation Error: Invalid URI format: invalid_uri"
        );
    }
}
