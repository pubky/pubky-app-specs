use crate::{
    traits::{HasPath, TimestampId, Validatable},
    APP_PATH,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use url::Url;
use utoipa::ToSchema;

// Validation
const MAX_SHORT_CONTENT_LENGTH: usize = 1000;
const MAX_LONG_CONTENT_LENGTH: usize = 50000;

/// Represents the type of pubky-app posted data
/// Used primarily to best display the content in UI
#[derive(Serialize, Deserialize, ToSchema, Default, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PubkyAppPostKind {
    #[default]
    Short,
    Long,
    Image,
    Video,
    Link,
    File,
}

impl fmt::Display for PubkyAppPostKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string_repr = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        write!(f, "{}", string_repr)
    }
}

/// Represents embedded content within a post
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct PubkyAppPostEmbed {
    kind: PubkyAppPostKind, // Kind of the embedded content
    uri: String,            // URI of the embedded content
}

/// Represents raw post in homeserver with content and kind
/// URI: /pub/pubky.app/posts/:post_id
/// Where post_id is CrockfordBase32 encoding of timestamp
///
/// Example URI:
///
/// `/pub/pubky.app/posts/00321FCW75ZFY`
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct PubkyAppPost {
    content: String,
    kind: PubkyAppPostKind,
    parent: Option<String>, // If a reply, the URI of the parent post.
    embed: Option<PubkyAppPostEmbed>,
    attachments: Option<Vec<String>>,
}

impl PubkyAppPost {
    /// Creates a new `PubkyAppPost` instance and sanitizes it.
    pub fn new(
        content: String,
        kind: PubkyAppPostKind,
        parent: Option<String>,
        embed: Option<PubkyAppPostEmbed>,
        attachments: Option<Vec<String>>,
    ) -> Self {
        let post = PubkyAppPost {
            content,
            kind,
            parent,
            embed,
            attachments,
        };
        post.sanitize()
    }
}

impl TimestampId for PubkyAppPost {}

impl HasPath for PubkyAppPost {
    fn create_path(&self) -> String {
        format!("{}posts/{}", APP_PATH, self.create_id())
    }
}

impl Validatable for PubkyAppPost {
    fn sanitize(self) -> Self {
        // Sanitize content
        let mut content = self.content.trim().to_string();

        // We are using content keyword `[DELETED]` for deleted posts from a homeserver that still have relationships
        // placed by other users (replies, tags, etc). This content is exactly matched by the client to apply effects to deleted content.
        // Placing posts with content `[DELETED]` is not allowed.
        if content == *"[DELETED]" {
            content = "empty".to_string()
        }

        // Define content length limits based on PubkyAppPostKind
        let max_content_length = match self.kind {
            PubkyAppPostKind::Short => MAX_SHORT_CONTENT_LENGTH,
            PubkyAppPostKind::Long => MAX_LONG_CONTENT_LENGTH,
            _ => MAX_SHORT_CONTENT_LENGTH, // Default limit for other kinds
        };

        let content = content.chars().take(max_content_length).collect::<String>();

        // Sanitize parent URI if present
        let parent = if let Some(uri_str) = &self.parent {
            match Url::parse(uri_str) {
                Ok(url) => Some(url.to_string()), // Valid URI, use normalized version
                Err(_) => None,                   // Invalid URI, discard or handle appropriately
            }
        } else {
            None
        };

        // Sanitize embed if present
        let embed = if let Some(embed) = &self.embed {
            match Url::parse(&embed.uri) {
                Ok(url) => Some(PubkyAppPostEmbed {
                    kind: embed.kind.clone(),
                    uri: url.to_string(), // Use normalized version
                }),
                Err(_) => None, // Invalid URI, discard or handle appropriately
            }
        } else {
            None
        };

        PubkyAppPost {
            content,
            kind: self.kind,
            parent,
            embed,
            attachments: self.attachments,
        }
    }

    fn validate(&self, id: &str) -> Result<(), String> {
        self.validate_id(id)?;

        // Validate content length
        match self.kind {
            PubkyAppPostKind::Short => {
                if self.content.chars().count() > MAX_SHORT_CONTENT_LENGTH {
                    return Err(
                        "Validation Error: Post content exceeds maximum length for Short kind"
                            .into(),
                    );
                }
            }
            PubkyAppPostKind::Long => {
                if self.content.chars().count() > MAX_LONG_CONTENT_LENGTH {
                    return Err(
                        "Validation Error: Post content exceeds maximum length for Short kind"
                            .into(),
                    );
                }
            }
            _ => (),
        };

        // TODO: additional validation. Attachement URLs...?

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Validatable;
    use bytes::Bytes;

    #[test]
    fn test_create_id() {
        let post = PubkyAppPost::new(
            "Hello World!".to_string(),
            PubkyAppPostKind::Short,
            None,
            None,
            None,
        );

        let post_id = post.create_id();
        println!("Generated Post ID: {}", post_id);

        // Assert that the post ID is 13 characters long
        assert_eq!(post_id.len(), 13);
    }

    #[test]
    fn test_new() {
        let content = "This is a test post".to_string();
        let kind = PubkyAppPostKind::Short;
        let post = PubkyAppPost::new(content.clone(), kind.clone(), None, None, None);

        assert_eq!(post.content, content);
        assert_eq!(post.kind, kind);
        assert!(post.parent.is_none());
        assert!(post.embed.is_none());
        assert!(post.attachments.is_none());
    }

    #[test]
    fn test_create_path() {
        let post = PubkyAppPost::new(
            "Test post".to_string(),
            PubkyAppPostKind::Short,
            None,
            None,
            None,
        );

        let post_id = post.create_id();
        let path = post.create_path();

        // Check if the path starts with the expected prefix
        let prefix = format!("{}posts/", APP_PATH);
        assert!(path.starts_with(&prefix));

        let expected_path_len = prefix.len() + post_id.len();
        assert_eq!(path.len(), expected_path_len);
    }

    #[test]
    fn test_sanitize() {
        let content = "  This is a test post with extra whitespace   ".to_string();
        let post = PubkyAppPost::new(
            content.clone(),
            PubkyAppPostKind::Short,
            Some("invalid uri".to_string()),
            Some(PubkyAppPostEmbed {
                kind: PubkyAppPostKind::Link,
                uri: "invalid uri".to_string(),
            }),
            None,
        );

        let sanitized_post = post.sanitize();
        assert_eq!(sanitized_post.content, content.trim());
        assert!(sanitized_post.parent.is_none());
        assert!(sanitized_post.embed.is_none());
    }

    #[test]
    fn test_validate_valid() {
        let post = PubkyAppPost::new(
            "Valid content".to_string(),
            PubkyAppPostKind::Short,
            None,
            None,
            None,
        );

        let id = post.create_id();
        let result = post.validate(&id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_id() {
        let post = PubkyAppPost::new(
            "Valid content".to_string(),
            PubkyAppPostKind::Short,
            None,
            None,
            None,
        );

        let invalid_id = "INVALIDID12345";
        let result = post.validate(&invalid_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_valid() {
        let post_json = r#"
        {
            "content": "Hello World!",
            "kind": "short",
            "parent": null,
            "embed": null,
            "attachments": null
        }
        "#;

        let id = PubkyAppPost::new(
            "Hello World!".to_string(),
            PubkyAppPostKind::Short,
            None,
            None,
            None,
        )
        .create_id();

        let blob = Bytes::from(post_json);
        let post = <PubkyAppPost as Validatable>::try_from(&blob, &id).unwrap();

        assert_eq!(post.content, "Hello World!");
    }

    #[test]
    fn test_try_from_invalid_content() {
        let content = "[DELETED]".to_string();
        let post_json = format!(
            r#"{{
                "content": "{}",
                "kind": "short",
                "parent": null,
                "embed": null,
                "attachments": null
            }}"#,
            content
        );

        let id = PubkyAppPost::new(content.clone(), PubkyAppPostKind::Short, None, None, None)
            .create_id();

        let blob = Bytes::from(post_json);
        let post = <PubkyAppPost as Validatable>::try_from(&blob, &id).unwrap();

        assert_eq!(post.content, "empty"); // After sanitization
    }
}
