use crate::{
    traits::{HasIdPath, TimestampId, Validatable},
    APP_PATH, PUBLIC_PATH,
};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use url::Url;

// Validation
const MAX_SHORT_CONTENT_LENGTH: usize = 2000;
const MAX_LONG_CONTENT_LENGTH: usize = 50000;
// Reserved keyword used by the system to mark deleted posts with relationships
const RESERVED_CONTENT_DELETED: &str = "[DELETED]";
const MAX_ATTACHMENTS: usize = 3;
const MAX_ATTACHMENT_URL_LENGTH: usize = 200;
// Allowed protocols for attachment URLs: pubky://, http://, https://
const ALLOWED_ATTACHMENT_PROTOCOLS: &[&str] = &["pubky", "http", "https"];

#[cfg(target_arch = "wasm32")]
use crate::traits::Json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Represents the type of pubky-app posted data
/// Used primarily to best display the content in UI
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
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

impl FromStr for PubkyAppPostKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "short" => Ok(PubkyAppPostKind::Short),
            "long" => Ok(PubkyAppPostKind::Long),
            "image" => Ok(PubkyAppPostKind::Image),
            "video" => Ok(PubkyAppPostKind::Video),
            "link" => Ok(PubkyAppPostKind::Link),
            "file" => Ok(PubkyAppPostKind::File),
            _ => Err(format!("Invalid content kind: {}", s)),
        }
    }
}

/// Represents embedded content within a post
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppPostEmbed {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub kind: PubkyAppPostKind, // Kind of the embedded content
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub uri: String, // URI of the embedded content
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppPostEmbed {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(uri: String, kind: PubkyAppPostKind) -> Self {
        PubkyAppPostEmbed { uri, kind }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn kind(&self) -> String {
        match self.kind {
            PubkyAppPostKind::Short => "Short".to_string(),
            PubkyAppPostKind::Long => "Long".to_string(),
            PubkyAppPostKind::Image => "Image".to_string(),
            PubkyAppPostKind::Video => "Video".to_string(),
            PubkyAppPostKind::Link => "Link".to_string(),
            PubkyAppPostKind::File => "File".to_string(),
        }
        // pub fn kind(&self) -> PubkyAppPostKind {
        //     self.kind.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn uri(&self) -> String {
        self.uri.clone()
    }
}

/// Represents raw post in homeserver with content and kind
/// URI: /pub/pubky.app/posts/:post_id
/// Where post_id is CrockfordBase32 encoding of timestamp
///
/// Example URI:
///
/// `/pub/pubky.app/posts/00321FCW75ZFY`
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppPost {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub content: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub kind: PubkyAppPostKind,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub parent: Option<String>, // If a reply, the URI of the parent post.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub embed: Option<PubkyAppPostEmbed>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub attachments: Option<Vec<String>>,
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppPost {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn content(&self) -> String {
        self.content.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn kind(&self) -> String {
        match self.kind {
            PubkyAppPostKind::Short => "Short".to_string(),
            PubkyAppPostKind::Long => "Long".to_string(),
            PubkyAppPostKind::Image => "Image".to_string(),
            PubkyAppPostKind::Video => "Video".to_string(),
            PubkyAppPostKind::Link => "Link".to_string(),
            PubkyAppPostKind::File => "File".to_string(),
        }
        // pub fn kind(&self) -> PubkyAppPostKind {
        //     self.kind.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn parent(&self) -> Option<String> {
        self.parent.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn embed(&self) -> Option<PubkyAppPostEmbed> {
        self.embed.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn attachments(&self) -> Option<Vec<String>> {
        self.attachments.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = fromJson))]
    pub fn from_json(js_value: &JsValue) -> Result<Self, String> {
        Self::import_json(js_value)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = toJson))]
    pub fn to_json(&self) -> Result<JsValue, String> {
        self.export_json()
    }
}

#[cfg(target_arch = "wasm32")]
impl Json for PubkyAppPost {}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppPost {
    /// Creates a new `PubkyAppPost` instance and sanitizes it.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
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

impl HasIdPath for PubkyAppPost {
    const PATH_SEGMENT: &'static str = "posts/";

    fn create_path(id: &str) -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT, id].concat()
    }
}

impl Validatable for PubkyAppPost {
    fn sanitize(self) -> Self {
        // Sanitize content: trim whitespace only
        let content = self.content.trim().to_string();

        // Sanitize parent URI if present
        let parent = if let Some(uri_str) = &self.parent {
            match Url::parse(uri_str.trim()) {
                Ok(url) => Some(url.to_string()), // Valid URI, use normalized version
                Err(_) => None,                   // Invalid URI, discard or handle appropriately
            }
        } else {
            None
        };

        // Sanitize embed if present
        let embed = if let Some(embed) = &self.embed {
            match Url::parse(embed.uri.trim()) {
                Ok(url) => Some(PubkyAppPostEmbed {
                    kind: embed.kind.clone(),
                    uri: url.to_string(), // Use normalized version
                }),
                Err(_) => None, // Invalid URI, discard or handle appropriately
            }
        } else {
            None
        };

        // Sanitize attachments: normalize URLs and filter out invalid ones
        let attachments = if let Some(attachments_vec) = &self.attachments {
            let sanitized: Vec<String> = attachments_vec
                .iter()
                .filter_map(|url_str| {
                    match Url::parse(url_str.trim()) {
                        Ok(parsed_url) => Some(parsed_url.to_string()), // Valid URL, normalized
                        Err(_) => None,                                 // Invalid URL, filter out
                    }
                })
                .collect();
            if sanitized.is_empty() {
                None
            } else {
                Some(sanitized)
            }
        } else {
            None
        };

        PubkyAppPost {
            content,
            kind: self.kind,
            parent,
            embed,
            attachments,
        }
    }

    fn validate(&self, id: Option<&str>) -> Result<(), String> {
        // Validate the post ID
        if let Some(id) = id {
            self.validate_id(id)?;
        }

        // Validate that post has meaningful content (at least one of: content, embed, or attachments)
        if self.content.trim().is_empty() && self.embed.is_none() && self.attachments.is_none() {
            return Err(
                "Validation Error: Post must have content, an embed, or attachments".into(),
            );
        }

        // We use content keyword `[DELETED]` for deleted posts from a homeserver that still have relationships
        // placed by other users (replies, tags, etc). This content is exactly matched by the client to apply effects to deleted content.
        // Placing posts with content `[DELETED]` is not allowed.
        if self.content == RESERVED_CONTENT_DELETED {
            return Err(
                "Validation Error: Content cannot be the reserved keyword '[DELETED]'".into(),
            );
        }

        // Validate content length based on post kind
        let (max_length, kind_name) = match self.kind {
            PubkyAppPostKind::Short => (MAX_SHORT_CONTENT_LENGTH, "Short"),
            PubkyAppPostKind::Long => (MAX_LONG_CONTENT_LENGTH, "Long"),
            PubkyAppPostKind::Image
            | PubkyAppPostKind::Video
            | PubkyAppPostKind::Link
            | PubkyAppPostKind::File => (MAX_SHORT_CONTENT_LENGTH, "Image/Video/Link/File"),
        };

        if self.content.chars().count() > max_length {
            return Err(format!(
                "Validation Error: Post content exceeds maximum length for {} kind (max: {} characters)",
                kind_name, max_length
            ));
        }

        // Validate attachments
        if let Some(attachments) = &self.attachments {
            if attachments.len() > MAX_ATTACHMENTS {
                return Err(format!(
                    "Validation Error: Too many attachments (max: {})",
                    MAX_ATTACHMENTS
                ));
            }

            for (index, url) in attachments.iter().enumerate() {
                if url.trim().is_empty() {
                    return Err(format!(
                        "Validation Error: Attachment URL at index {} cannot be empty",
                        index
                    ));
                }
                if url.chars().count() > MAX_ATTACHMENT_URL_LENGTH {
                    return Err(format!(
                        "Validation Error: Attachment URL at index {} exceeds maximum length (max: {} characters)",
                        index, MAX_ATTACHMENT_URL_LENGTH
                    ));
                }
                // Validate URL format and ensure it uses an allowed protocol
                let parsed_url = Url::parse(url).map_err(|_| {
                    format!(
                        "Validation Error: Invalid attachment URL format at index {}",
                        index
                    )
                })?;

                // Ensure the URL uses an allowed protocol
                if !ALLOWED_ATTACHMENT_PROTOCOLS.contains(&parsed_url.scheme()) {
                    let allowed_protocols = ALLOWED_ATTACHMENT_PROTOCOLS
                        .iter()
                        .map(|p| format!("{}://", p))
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(format!(
                        "Validation Error: Attachment URL at index {} must use one of the allowed protocols: {}",
                        index, allowed_protocols
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{traits::Validatable, APP_PATH, PUBLIC_PATH};

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
        let path = PubkyAppPost::create_path(&post_id);

        // Check if the path starts with the expected prefix
        let prefix = format!("{}{}posts/", PUBLIC_PATH, APP_PATH);
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
            Some(vec![
                "pubky://6mfxozzqmb36rc9rgy3rykoyfghfao74n8igt5tf1boehproahoy/pub/pubky.app/files/0034A0X7NJ52G".to_string(),
                "invalid uri".to_string(), // Should be filtered out
                "  pubky://6mfxozzqmb36rc9rgy3rykoyfghfao74n8igt5tf1boehproahoy/pub/pubky.app/files/0034A0X7Q3D80  ".to_string(), // Should be trimmed and normalized
            ]),
        );

        let sanitized_post = post.sanitize();
        assert_eq!(sanitized_post.content, content.trim());
        assert!(sanitized_post.parent.is_none());
        assert!(sanitized_post.embed.is_none());
        assert!(sanitized_post.attachments.is_some());
        let attachments = sanitized_post.attachments.unwrap();
        assert_eq!(attachments.len(), 2); // Invalid URL should be filtered out
        assert!(attachments[0].starts_with("pubky://"));
        assert!(attachments[1].starts_with("pubky://"));
        // Check that whitespace was trimmed
        assert!(!attachments[1].starts_with("  pubky://"));
        assert!(!attachments[1].ends_with("  "));
    }

    #[test]
    fn test_sanitize_trims_parent_and_embed() {
        let valid_parent_uri = "  pubky://6mfxozzqmb36rc9rgy3rykoyfghfao74n8igt5tf1boehproahoy/pub/pubky.app/posts/0034A0X7NJ52G  ".to_string();
        let valid_embed_uri = "  pubky://6mfxozzqmb36rc9rgy3rykoyfghfao74n8igt5tf1boehproahoy/pub/pubky.app/files/0034A0X7Q3D80  ".to_string();

        let post = PubkyAppPost::new(
            "Test content".to_string(),
            PubkyAppPostKind::Short,
            Some(valid_parent_uri.clone()),
            Some(PubkyAppPostEmbed {
                kind: PubkyAppPostKind::Link,
                uri: valid_embed_uri.clone(),
            }),
            None,
        );

        let sanitized_post = post.sanitize();

        // Check that parent URI was trimmed and normalized
        assert!(sanitized_post.parent.is_some());
        let parent = sanitized_post.parent.unwrap();
        assert!(!parent.starts_with("  "));
        assert!(!parent.ends_with("  "));
        assert!(parent.starts_with("pubky://"));

        // Check that embed URI was trimmed and normalized
        assert!(sanitized_post.embed.is_some());
        let embed = sanitized_post.embed.unwrap();
        assert!(!embed.uri.starts_with("  "));
        assert!(!embed.uri.ends_with("  "));
        assert!(embed.uri.starts_with("pubky://"));
    }

    #[test]
    fn test_validate() {
        let post = PubkyAppPost::new(
            "Valid content".to_string(),
            PubkyAppPostKind::Short,
            None,
            None,
            None,
        );

        let id = post.create_id();
        let result = post.validate(Some(&id));
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
        let result = post.validate(Some(invalid_id));
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

        let blob = post_json.as_bytes();
        let post = <PubkyAppPost as Validatable>::try_from(blob, &id).unwrap();

        assert_eq!(post.content, "Hello World!");
    }

    #[test]
    fn test_validate_reserved_keyword() {
        let post = PubkyAppPost::new(
            "[DELETED]".to_string(),
            PubkyAppPostKind::Short,
            None,
            None,
            None,
        );

        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("reserved keyword"));
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

        let blob = post_json.as_bytes();
        let result = <PubkyAppPost as Validatable>::try_from(blob, &id);

        // Should fail validation because [DELETED] is a reserved keyword
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("reserved keyword"));
    }

    #[test]
    fn test_validate_attachments_valid_protocols() {
        // Test allowed protocols (limited to MAX_ATTACHMENTS)
        let protocols = vec![
            "pubky://6mfxozzqmb36rc9rgy3rykoyfghfao74n8igt5tf1boehproahoy/pub/pubky.app/files/0034A0X7NJ52G".to_string(),
            "https://example.com/file.png".to_string(),
            "http://example.com/file.jpg".to_string(),
        ];
        assert!(
            protocols.len() <= MAX_ATTACHMENTS,
            "Test uses more than MAX_ATTACHMENTS"
        );

        let post = PubkyAppPost::new(
            "Valid content".to_string(),
            PubkyAppPostKind::Image,
            None,
            None,
            Some(protocols),
        );

        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_attachments_all_allowed_protocols() {
        // Test each allowed protocol individually to ensure all are accepted
        let allowed_protocols = vec![
            "pubky://6mfxozzqmb36rc9rgy3rykoyfghfao74n8igt5tf1boehproahoy/pub/pubky.app/files/0034A0X7NJ52G",
            "http://example.com/file.jpg",
            "https://example.com/file.png",
        ];

        for protocol_url in allowed_protocols {
            let post = PubkyAppPost::new(
                "Valid content".to_string(),
                PubkyAppPostKind::Image,
                None,
                None,
                Some(vec![protocol_url.to_string()]),
            );

            let id = post.create_id();
            let result = post.validate(Some(&id));
            assert!(result.is_ok(), "Should accept protocol: {}", protocol_url);
        }
    }

    #[test]
    fn test_validate_attachments_too_many() {
        let mut attachments = Vec::new();
        for i in 0..MAX_ATTACHMENTS + 1 {
            attachments.push(format!(
                "pubky://6mfxozzqmb36rc9rgy3rykoyfghfao74n8igt5tf1boehproahoy/pub/pubky.app/files/{}",
                i
            ));
        }

        let post = PubkyAppPost::new(
            "Valid content".to_string(),
            PubkyAppPostKind::Image,
            None,
            None,
            Some(attachments),
        );

        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Too many attachments"));
    }

    #[test]
    fn test_validate_attachments_invalid_protocol() {
        // Test that disallowed protocols are rejected
        let invalid_protocols = vec!["ftp://example.com/file", "file:///path/to/file"];

        for invalid_url in invalid_protocols {
            let post = PubkyAppPost {
                content: "Valid content".to_string(),
                kind: PubkyAppPostKind::Image,
                parent: None,
                embed: None,
                attachments: Some(vec![invalid_url.to_string()]),
            };

            let id = post.create_id();
            let result = post.validate(Some(&id));
            assert!(result.is_err(), "Should reject protocol: {}", invalid_url);
            assert!(result.unwrap_err().contains("protocol"));
        }
    }

    #[test]
    fn test_validate_attachments_invalid_url_format() {
        // Create post directly without sanitization to test validation logic
        let post = PubkyAppPost {
            content: "Valid content".to_string(),
            kind: PubkyAppPostKind::Image,
            parent: None,
            embed: None,
            attachments: Some(vec!["not a valid url".to_string()]),
        };

        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Invalid attachment URL format"));
    }

    #[test]
    fn test_validate_attachments_url_too_long() {
        // Create a URL that exceeds MAX_ATTACHMENT_URL_LENGTH (200)
        // Base URL structure: "pubky://<52-char-user-id>/pub/pubky.app/files/" = ~80 chars
        // So we need a file ID that makes the total exceed 200
        let long_file_id = "a".repeat(150); // This will make total > 200
        let long_url = format!(
            "pubky://6mfxozzqmb36rc9rgy3rykoyfghfao74n8igt5tf1boehproahoy/pub/pubky.app/files/{}",
            long_file_id
        );

        // Verify the URL is actually too long
        assert!(
            long_url.chars().count() > MAX_ATTACHMENT_URL_LENGTH,
            "URL length {} should exceed {}",
            long_url.chars().count(),
            MAX_ATTACHMENT_URL_LENGTH
        );

        let post = PubkyAppPost::new(
            "Valid content".to_string(),
            PubkyAppPostKind::Image,
            None,
            None,
            Some(vec![long_url]),
        );

        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds maximum length"));
    }

    #[test]
    fn test_validate_attachments_empty_url() {
        // Create post directly without sanitization to test validation logic
        let post = PubkyAppPost {
            content: "Valid content".to_string(),
            kind: PubkyAppPostKind::Image,
            parent: None,
            embed: None,
            attachments: Some(vec!["   ".to_string()]), // Whitespace only
        };

        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be empty"));
    }

    #[test]
    fn test_sanitize_attachments_filters_invalid() {
        let post = PubkyAppPost::new(
            "Valid content".to_string(),
            PubkyAppPostKind::Image,
            None,
            None,
            Some(vec![
                "pubky://6mfxozzqmb36rc9rgy3rykoyfghfao74n8igt5tf1boehproahoy/pub/pubky.app/files/0034A0X7NJ52G".to_string(),
                "https://example.com/file.jpg".to_string(), // Valid
                "invalid url".to_string(), // Should be filtered out
                "not a url".to_string(),   // Should be filtered out
            ]),
        );

        let sanitized = post.sanitize();
        assert!(sanitized.attachments.is_some());
        let attachments = sanitized.attachments.unwrap();
        assert_eq!(attachments.len(), 2); // Only valid URLs should remain
        assert!(attachments[0].starts_with("pubky://"));
        assert!(attachments[1].starts_with("https://"));
    }

    #[test]
    fn test_sanitize_attachments_all_invalid_becomes_none() {
        let post = PubkyAppPost::new(
            "Valid content".to_string(),
            PubkyAppPostKind::Image,
            None,
            None,
            Some(vec!["invalid url".to_string(), "not a url".to_string()]),
        );

        let sanitized = post.sanitize();
        assert!(sanitized.attachments.is_none()); // All invalid, should become None
    }

    #[test]
    fn test_validate_empty_post_rejected() {
        // Post with empty content, no embed, and no attachments should be rejected
        let post = PubkyAppPost::new("".to_string(), PubkyAppPostKind::Short, None, None, None);

        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("must have content, an embed, or attachments"));
    }

    #[test]
    fn test_validate_empty_content_with_embed_accepted() {
        // Post with empty content but with embed should be valid
        let post = PubkyAppPost::new(
            "".to_string(),
            PubkyAppPostKind::Short,
            None,
            Some(PubkyAppPostEmbed {
                kind: PubkyAppPostKind::Short,
                uri: "pubky://user123/pub/pubky.app/posts/0033SSE3B1FQ0".to_string(),
            }),
            None,
        );

        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(
            result.is_ok(),
            "Post with embed but no content should be valid"
        );
    }

    #[test]
    fn test_validate_empty_content_with_attachments_accepted() {
        // Post with empty content but with attachments should be valid
        let post = PubkyAppPost::new(
            "".to_string(),
            PubkyAppPostKind::Image,
            None,
            None,
            Some(vec![
                "pubky://user123/pub/pubky.app/files/0034A0X7NJ52G".to_string()
            ]),
        );

        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(
            result.is_ok(),
            "Post with attachments but no content should be valid"
        );
    }
}
