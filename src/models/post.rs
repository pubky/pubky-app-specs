use crate::{
    common::sanitize_url,
    limits::VALIDATION_LIMITS,
    traits::{HasIdPath, TimestampId, Validatable},
    APP_PATH, PUBLIC_PATH,
};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use url::Url;

// Reserved keyword used by the system to mark deleted posts with relationships
const RESERVED_CONTENT_DELETED: &str = "[DELETED]";

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
    Collection,
    #[serde(other)]
    Unknown,
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
            "collection" => Ok(PubkyAppPostKind::Collection),
            _ => Err(format!("Invalid content kind: {}", s)),
        }
    }
}

impl PubkyAppPostKind {
    /// Returns `true` for every spec-recognized variant, `false` for `Unknown`.
    ///
    /// `Unknown` is the forwards-compat catch-all variant (via `#[serde(other)]`)
    /// that captures any post-kind string this version of the spec doesn't
    /// recognize yet. Most consumers — indexers, stream filters, search ranking —
    /// want to skip such posts, and this helper lets them write
    /// `if kind.is_known() { ... }` rather than
    /// `if !matches!(kind, PubkyAppPostKind::Unknown) { ... }`.
    pub fn is_known(&self) -> bool {
        !matches!(self, PubkyAppPostKind::Unknown)
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
            PubkyAppPostKind::Collection => "Collection".to_string(),
            PubkyAppPostKind::Unknown => "Unknown".to_string(),
        }
        // pub fn kind(&self) -> PubkyAppPostKind {
        //     self.kind.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn uri(&self) -> String {
        self.uri.clone()
    }
}

/// Typed JSON envelope stored in `PubkyAppPost::content` when `kind == Collection`.
///
/// A collection post curates an ordered list of other posts (URIs in `attachments`)
/// under a `name` and optional `description`. The envelope is parsed and validated
/// by the spec but never re-serialized as a top-level homeserver object.
///
/// Forward-compat: `#[serde(deny_unknown_fields)]` is intentionally NOT used so
/// future minor versions can add fields (e.g. `cover_image`) without breaking
/// older parsers. New fields must be additive and ignorable.
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct PubkyAppCollectionContent {
    /// Display name of the collection. Length bounded by
    /// `VALIDATION_LIMITS.collection_name_{min,max}_length` (in unicode
    /// scalars, after trim).
    pub name: String,
    /// Optional human-readable description. Length bounded by
    /// `VALIDATION_LIMITS.collection_description_max_length` (unicode scalars).
    pub description: Option<String>,
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
            PubkyAppPostKind::Collection => "Collection".to_string(),
            PubkyAppPostKind::Unknown => "Unknown".to_string(),
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
        let parent = self.parent.map(|uri_str| sanitize_url(&uri_str));

        // Sanitize embed if present
        let embed = self.embed.map(|e| PubkyAppPostEmbed {
            kind: e.kind,
            uri: sanitize_url(&e.uri),
        });

        // Sanitize attachments
        let attachments = self.attachments.map(|attachments_vec| {
            attachments_vec
                .into_iter()
                .map(|url_str| sanitize_url(&url_str))
                .collect()
        });

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

        // Reject posts whose kind couldn't be matched against any known variant.
        // `Unknown` is a serde catch-all for forwards-compat: older binaries can
        // deserialize events from newer clients without panicking, but such posts
        // must never pass spec validation. Same reasoning for `embed.kind`.
        if matches!(self.kind, PubkyAppPostKind::Unknown) {
            return Err("Validation Error: post kind is unknown".into());
        }
        if let Some(ref embed) = self.embed {
            if matches!(embed.kind, PubkyAppPostKind::Unknown) {
                return Err("Validation Error: embed kind is unknown".into());
            }
        }

        // Collection posts use a typed JSON envelope in `content` and lift the
        // attachment caps. They forbid `parent` and `embed`. Branch out before the
        // legacy kind-switch so the existing per-kind tuple stays untouched.
        if matches!(self.kind, PubkyAppPostKind::Collection) {
            if self.parent.is_some() || self.embed.is_some() {
                return Err(
                    "Validation Error: Collection posts cannot have parent or embed".into(),
                );
            }
            if self.content.chars().count() > VALIDATION_LIMITS.collection_content_max_length {
                return Err(format!(
                    "Validation Error: Collection content exceeds max length {}",
                    VALIDATION_LIMITS.collection_content_max_length
                ));
            }
            let envelope: PubkyAppCollectionContent =
                serde_json::from_str(&self.content).map_err(|e| {
                    format!(
                        "Validation Error: Collection content must be a valid JSON envelope: {}",
                        e
                    )
                })?;
            let name_chars = envelope.name.trim().chars().count();
            let name_min = VALIDATION_LIMITS.collection_name_min_length;
            let name_max = VALIDATION_LIMITS.collection_name_max_length;
            if !(name_min..=name_max).contains(&name_chars) {
                return Err(format!(
                    "Validation Error: Collection name must be {}..={} characters",
                    name_min, name_max
                ));
            }
            if let Some(desc) = &envelope.description {
                if desc.chars().count() > VALIDATION_LIMITS.collection_description_max_length {
                    return Err(format!(
                        "Validation Error: Collection description exceeds {} characters",
                        VALIDATION_LIMITS.collection_description_max_length
                    ));
                }
            }
            if let Some(attachments) = &self.attachments {
                if attachments.len() > VALIDATION_LIMITS.collection_items_max_count {
                    return Err(format!(
                        "Validation Error: Collection cannot have more than {} items",
                        VALIDATION_LIMITS.collection_items_max_count
                    ));
                }
                for uri in attachments {
                    if uri.chars().count() > VALIDATION_LIMITS.collection_item_uri_max_length {
                        return Err(format!(
                            "Validation Error: Collection item URI exceeds {} characters",
                            VALIDATION_LIMITS.collection_item_uri_max_length
                        ));
                    }
                    let parsed = Url::parse(uri).map_err(|_| {
                        format!("Validation Error: Invalid attachment URL: {}", uri)
                    })?;
                    if !VALIDATION_LIMITS
                        .post_allowed_attachment_protocols
                        .contains(&parsed.scheme())
                    {
                        return Err(format!(
                            "Validation Error: Disallowed attachment protocol: {}",
                            parsed.scheme()
                        ));
                    }
                }
            }
            return Ok(());
        }

        // Validate content length based on post kind
        let (max_length, kind_name) = match self.kind {
            PubkyAppPostKind::Short => (VALIDATION_LIMITS.post_short_content_max_length, "Short"),
            PubkyAppPostKind::Long => (VALIDATION_LIMITS.post_long_content_max_length, "Long"),
            PubkyAppPostKind::Image
            | PubkyAppPostKind::Video
            | PubkyAppPostKind::Link
            | PubkyAppPostKind::File => (
                VALIDATION_LIMITS.post_short_content_max_length,
                "Image/Video/Link/File",
            ),
            PubkyAppPostKind::Collection | PubkyAppPostKind::Unknown => {
                unreachable!("guarded by early-return above")
            }
        };

        if self.content.chars().count() > max_length {
            return Err(format!(
                "Validation Error: Post content exceeds maximum length for {} kind (max: {} characters)",
                kind_name, max_length
            ));
        }

        // Validate parent URI format if present
        if let Some(ref parent_uri) = self.parent {
            Url::parse(parent_uri).map_err(|_| {
                format!(
                    "Validation Error: Invalid parent URI format: {}",
                    parent_uri
                )
            })?;
        }

        // Validate embed URI format if present
        if let Some(ref embed) = self.embed {
            Url::parse(&embed.uri).map_err(|_| {
                format!("Validation Error: Invalid embed URI format: {}", embed.uri)
            })?;
        }

        // Validate attachments
        if let Some(attachments) = &self.attachments {
            if attachments.len() > VALIDATION_LIMITS.post_attachments_max_count {
                return Err(format!(
                    "Validation Error: Too many attachments (max: {})",
                    VALIDATION_LIMITS.post_attachments_max_count
                ));
            }

            for (index, url) in attachments.iter().enumerate() {
                if url.trim().is_empty() {
                    return Err(format!(
                        "Validation Error: Attachment URL at index {} cannot be empty",
                        index
                    ));
                }
                if url.chars().count() > VALIDATION_LIMITS.post_attachment_url_max_length {
                    return Err(format!(
                        "Validation Error: Attachment URL at index {} exceeds maximum length (max: {} characters)",
                        index, VALIDATION_LIMITS.post_attachment_url_max_length
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
                if !VALIDATION_LIMITS
                    .post_allowed_attachment_protocols
                    .contains(&parsed_url.scheme())
                {
                    let allowed_protocols = VALIDATION_LIMITS
                        .post_allowed_attachment_protocols
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
            Some("  pubky://6mfxozzqmb36rc9rgy3rykoyfghfao74n8igt5tf1boehproahoy/pub/pubky.app/posts/0034A0X7NJ52G  ".to_string()),
            Some(PubkyAppPostEmbed {
                kind: PubkyAppPostKind::Link,
                uri: "  pubky://6mfxozzqmb36rc9rgy3rykoyfghfao74n8igt5tf1boehproahoy/pub/pubky.app/files/0034A0X7Q3D80  ".to_string(),
            }),
            Some(vec![
                "pubky://6mfxozzqmb36rc9rgy3rykoyfghfao74n8igt5tf1boehproahoy/pub/pubky.app/files/0034A0X7NJ52G".to_string(),
                "  pubky://6mfxozzqmb36rc9rgy3rykoyfghfao74n8igt5tf1boehproahoy/pub/pubky.app/files/0034A0X7Q3D80  ".to_string(), // Should be trimmed
            ]),
        );

        let sanitized_post = post.sanitize();
        assert_eq!(sanitized_post.content, content.trim());

        // Parent URI should be trimmed
        assert!(sanitized_post.parent.is_some());
        let parent = sanitized_post.parent.unwrap();
        assert!(!parent.starts_with("  "));
        assert!(!parent.ends_with("  "));
        assert!(parent.starts_with("pubky://"));

        // Embed URI should be trimmed
        assert!(sanitized_post.embed.is_some());
        let embed = sanitized_post.embed.unwrap();
        assert!(!embed.uri.starts_with("  "));
        assert!(!embed.uri.ends_with("  "));
        assert!(embed.uri.starts_with("pubky://"));

        // Attachments should be trimmed
        assert!(sanitized_post.attachments.is_some());
        let attachments = sanitized_post.attachments.unwrap();
        assert_eq!(attachments.len(), 2);
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
    fn test_validate_invalid_parent_uri() {
        let post = PubkyAppPost::new(
            "Valid content".to_string(),
            PubkyAppPostKind::Short,
            Some("invalid uri".to_string()),
            None,
            None,
        );

        let id = post.create_id();
        let sanitized = post.sanitize();
        let result = sanitized.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid parent URI format"));
    }

    #[test]
    fn test_validate_invalid_embed_uri() {
        let post = PubkyAppPost::new(
            "Valid content".to_string(),
            PubkyAppPostKind::Short,
            None,
            Some(PubkyAppPostEmbed {
                kind: PubkyAppPostKind::Link,
                uri: "invalid uri".to_string(),
            }),
            None,
        );

        let id = post.create_id();
        let sanitized = post.sanitize();
        let result = sanitized.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid embed URI format"));
    }

    #[test]
    fn test_validate_invalid_attachment_uri() {
        let post = PubkyAppPost::new(
            "Valid content".to_string(),
            PubkyAppPostKind::Image,
            None,
            None,
            Some(vec![
                "pubky://6mfxozzqmb36rc9rgy3rykoyfghfao74n8igt5tf1boehproahoy/pub/pubky.app/files/0034A0X7NJ52G".to_string(),
                "invalid uri".to_string(),
            ]),
        );

        let id = post.create_id();
        let sanitized = post.sanitize();
        let result = sanitized.validate(Some(&id));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Invalid attachment URL format"));
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
        // Test allowed protocols (limited to post_attachments_max_count)
        let protocols = vec![
            "pubky://6mfxozzqmb36rc9rgy3rykoyfghfao74n8igt5tf1boehproahoy/pub/pubky.app/files/0034A0X7NJ52G".to_string(),
            "https://example.com/file.png".to_string(),
            "http://example.com/file.jpg".to_string(),
        ];
        assert!(
            protocols.len() <= VALIDATION_LIMITS.post_attachments_max_count,
            "Test uses more than post_attachments_max_count"
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
        for i in 0..VALIDATION_LIMITS.post_attachments_max_count + 1 {
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
        // Create a URL that exceeds post_attachment_url_max_length (200)
        // Base URL structure: "pubky://<52-char-user-id>/pub/pubky.app/files/" = ~80 chars
        // So we need a file ID that makes the total exceed 200
        let long_file_id = "a".repeat(150); // This will make total > 200
        let long_url = format!(
            "pubky://6mfxozzqmb36rc9rgy3rykoyfghfao74n8igt5tf1boehproahoy/pub/pubky.app/files/{}",
            long_file_id
        );

        // Verify the URL is actually too long
        assert!(
            long_url.chars().count() > VALIDATION_LIMITS.post_attachment_url_max_length,
            "URL length {} should exceed {}",
            long_url.chars().count(),
            VALIDATION_LIMITS.post_attachment_url_max_length
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
    fn test_sanitize_attachments_preserves_all() {
        // Sanitize should preserve all attachments (just trim), validation rejects invalid
        let post = PubkyAppPost::new(
            "Valid content".to_string(),
            PubkyAppPostKind::Image,
            None,
            None,
            Some(vec![
                "pubky://6mfxozzqmb36rc9rgy3rykoyfghfao74n8igt5tf1boehproahoy/pub/pubky.app/files/0034A0X7NJ52G".to_string(),
                "https://example.com/file.jpg".to_string(),
                "  invalid url  ".to_string(), // Should be trimmed but preserved
            ]),
        );

        let id = post.create_id();
        let sanitized = post.sanitize();
        assert!(sanitized.attachments.is_some());
        let attachments = sanitized.attachments.as_ref().unwrap();
        assert_eq!(attachments.len(), 3); // All URLs should be preserved
        assert!(attachments[0].starts_with("pubky://"));
        assert!(attachments[1].starts_with("https://"));
        assert_eq!(attachments[2], "invalid url"); // Trimmed but preserved

        // Validation should reject the invalid URL
        let result = sanitized.validate(Some(&id));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Invalid attachment URL format"));
    }

    #[test]
    fn test_sanitize_attachments_with_all_invalid_preserved() {
        // Sanitize should preserve all attachments, validation rejects invalid
        let post = PubkyAppPost::new(
            "Valid content".to_string(),
            PubkyAppPostKind::Image,
            None,
            None,
            Some(vec!["invalid url".to_string(), "not a url".to_string()]),
        );

        let id = post.create_id();
        let sanitized = post.sanitize();
        assert!(sanitized.attachments.is_some()); // Attachments preserved
        let attachments = sanitized.attachments.as_ref().unwrap();
        assert_eq!(attachments.len(), 2);

        // Validation should reject the invalid URLs
        let result = sanitized.validate(Some(&id));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Invalid attachment URL format"));
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

    // ----- v0.4.5 forwards-compat shim: PubkyAppPostKind::Unknown -----

    #[test]
    fn test_postkind_deserializes_unknown_kind_as_unknown() {
        // A future spec version adds a new kind that this binary doesn't know about.
        // The serde catch-all `Unknown` variant lets old binaries deserialize without panicking.
        let post_json = r#"
        {
            "content": "Hello",
            "kind": "totally-new-kind",
            "parent": null,
            "embed": null,
            "attachments": null
        }
        "#;

        let post: PubkyAppPost = serde_json::from_str(post_json).unwrap();
        assert_eq!(post.kind, PubkyAppPostKind::Unknown);
    }

    #[test]
    fn test_postkind_existing_variants_unchanged_after_unknown_added() {
        // Regression guard: adding Unknown with #[serde(other)] must not break round-tripping
        // any of the six existing lowercase string forms.
        for (s, expected) in [
            ("short", PubkyAppPostKind::Short),
            ("long", PubkyAppPostKind::Long),
            ("image", PubkyAppPostKind::Image),
            ("video", PubkyAppPostKind::Video),
            ("link", PubkyAppPostKind::Link),
            ("file", PubkyAppPostKind::File),
        ] {
            let json = format!(
                r#"{{"content":"x","kind":"{}","parent":null,"embed":null,"attachments":null}}"#,
                s
            );
            let post: PubkyAppPost = serde_json::from_str(&json).unwrap();
            assert_eq!(post.kind, expected, "kind={} did not round-trip", s);
            // re-serialize and ensure the lowercase string survives
            let re = serde_json::to_value(&post.kind).unwrap();
            assert_eq!(re.as_str(), Some(s));
        }
    }

    #[test]
    fn test_postkind_unknown_rejected_by_validator() {
        let post = PubkyAppPost {
            content: "x".to_string(),
            kind: PubkyAppPostKind::Unknown,
            parent: None,
            embed: None,
            attachments: None,
        };
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_lowercase().contains("unknown"),
            "validator should mention 'unknown' in the error"
        );
    }

    #[test]
    fn test_postkind_unknown_displays_as_lowercase() {
        assert_eq!(PubkyAppPostKind::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_postkind_fromstr_rejects_unknown_strings() {
        // FromStr stays strict: it does NOT produce Unknown for arbitrary input.
        // Unknown is exclusively a serde catch-all.
        assert!(PubkyAppPostKind::from_str("foobar").is_err());
        assert!(PubkyAppPostKind::from_str("totally-new-kind").is_err());
    }

    #[test]
    fn test_is_known_returns_true_for_all_recognized_variants() {
        use PubkyAppPostKind::*;
        for k in [Short, Long, Image, Video, Link, File, Collection] {
            assert!(k.is_known(), "{k:?} should be known");
        }
    }

    #[test]
    fn test_is_known_returns_false_for_unknown() {
        assert!(!PubkyAppPostKind::Unknown.is_known());
    }

    #[test]
    fn test_post_deserializes_embed_with_unknown_kind_as_unknown() {
        // Embed kinds get the same forwards-compat treatment as top-level kinds:
        // an unrecognized embed.kind deserializes to Unknown rather than failing.
        let post_json = r#"
        {
            "content": "x",
            "kind": "short",
            "parent": null,
            "embed": {"kind": "totally-new-embed-kind", "uri": "pubky://x/pub/pubky.app/posts/01"},
            "attachments": null
        }
        "#;
        let post: PubkyAppPost = serde_json::from_str(post_json).unwrap();
        assert_eq!(post.embed.unwrap().kind, PubkyAppPostKind::Unknown);
    }

    #[test]
    fn test_postkind_unknown_embed_kind_rejected_by_validator() {
        // Counterpart to `test_postkind_unknown_rejected_by_validator`:
        // an Unknown embed.kind also fails validation, so the spec stays as
        // strict as before for posts that reach validation.
        let post = PubkyAppPost {
            content: "x".to_string(),
            kind: PubkyAppPostKind::Short,
            parent: None,
            embed: Some(PubkyAppPostEmbed {
                kind: PubkyAppPostKind::Unknown,
                uri: "pubky://x/pub/pubky.app/posts/01".to_string(),
            }),
            attachments: None,
        };
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        let err = result.unwrap_err().to_lowercase();
        assert!(
            err.contains("embed") && err.contains("unknown"),
            "validator should mention 'embed' and 'unknown' in the error, got: {}",
            err
        );
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn test_postkind_unknown_wasm_getter() {
        let post = PubkyAppPost {
            content: "x".to_string(),
            kind: PubkyAppPostKind::Unknown,
            parent: None,
            embed: None,
            attachments: None,
        };
        assert_eq!(post.kind(), "Unknown");
    }

    // ----- v0.5.0 Collection variant + PubkyAppCollectionContent envelope -----

    fn collection_envelope_json(name: &str, description: Option<&str>) -> String {
        match description {
            Some(d) => serde_json::to_string(&PubkyAppCollectionContent {
                name: name.to_string(),
                description: Some(d.to_string()),
            })
            .unwrap(),
            None => serde_json::to_string(&PubkyAppCollectionContent {
                name: name.to_string(),
                description: None,
            })
            .unwrap(),
        }
    }

    fn make_collection_post(
        name: &str,
        description: Option<&str>,
        attachments: Option<Vec<String>>,
    ) -> PubkyAppPost {
        PubkyAppPost::new(
            collection_envelope_json(name, description),
            PubkyAppPostKind::Collection,
            None,
            None,
            attachments,
        )
    }

    #[test]
    fn test_collection_post_roundtrip_valid() {
        let post = make_collection_post(
            "AI papers",
            Some("Best stuff"),
            Some(vec![
                "pubky://userA/pub/pubky.app/posts/0034A0X7NJ52A".to_string(),
                "pubky://userB/pub/pubky.app/posts/0034A0X7NJ52B".to_string(),
                "pubky://userC/pub/pubky.app/posts/0034A0X7NJ52C".to_string(),
            ]),
        );
        let id = post.create_id();
        let blob = serde_json::to_vec(&post).unwrap();
        let parsed = <PubkyAppPost as Validatable>::try_from(&blob, &id).unwrap();
        assert_eq!(parsed.kind, PubkyAppPostKind::Collection);
        assert_eq!(parsed.attachments.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_collection_post_rejects_malformed_envelope() {
        let post = PubkyAppPost::new(
            "this is not JSON".to_string(),
            PubkyAppPostKind::Collection,
            None,
            None,
            None,
        );
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("JSON envelope"),
            "expected JSON envelope error, got: {}",
            err
        );
    }

    #[test]
    fn test_collection_post_rejects_empty_name() {
        let post = make_collection_post("", None, None);
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name"));
    }

    #[test]
    fn test_collection_post_rejects_oversized_name() {
        // 101 grapheme-ish chars; mix in emoji to confirm we count by unicode scalars, not bytes.
        let oversized = "a".repeat(99) + "🚀🚀";
        assert_eq!(oversized.chars().count(), 101);
        let post = make_collection_post(&oversized, None, None);
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name"));
    }

    #[test]
    fn test_collection_post_accepts_max_name() {
        let exactly_100 = "a".repeat(100);
        assert_eq!(exactly_100.chars().count(), 100);
        let post = make_collection_post(&exactly_100, None, None);
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_collection_post_rejects_oversized_description() {
        let too_long = "a".repeat(501);
        let post = make_collection_post("X", Some(&too_long), None);
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("description"));
    }

    #[test]
    fn test_collection_post_rejects_parent() {
        let post = PubkyAppPost::new(
            collection_envelope_json("X", None),
            PubkyAppPostKind::Collection,
            Some("pubky://userA/pub/pubky.app/posts/0034A0X7NJ52A".to_string()),
            None,
            None,
        );
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("parent or embed"),
            "expected parent-or-embed error, got: {}",
            err
        );
    }

    #[test]
    fn test_collection_post_rejects_embed() {
        let post = PubkyAppPost::new(
            collection_envelope_json("X", None),
            PubkyAppPostKind::Collection,
            None,
            Some(PubkyAppPostEmbed {
                kind: PubkyAppPostKind::Short,
                uri: "pubky://userA/pub/pubky.app/posts/0034A0X7NJ52A".to_string(),
            }),
            None,
        );
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("parent or embed"));
    }

    #[test]
    fn test_collection_post_accepts_100_attachments() {
        let attachments: Vec<String> = (0..100)
            .map(|i| format!("pubky://userA/pub/pubky.app/posts/{:013}", i))
            .collect();
        let post = make_collection_post("Big list", None, Some(attachments));
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_collection_post_rejects_101_attachments() {
        let attachments: Vec<String> = (0..101)
            .map(|i| format!("pubky://userA/pub/pubky.app/posts/{:013}", i))
            .collect();
        let post = make_collection_post("Too big", None, Some(attachments));
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("100 items"));
    }

    #[test]
    fn test_collection_post_accepts_300_char_uri() {
        // Build a pubky URI with the user host + a long path filling out to exactly 300 chars.
        let prefix = "pubky://userA/pub/pubky.app/posts/";
        let pad = "x".repeat(300 - prefix.chars().count());
        let uri = format!("{}{}", prefix, pad);
        assert_eq!(uri.chars().count(), 300);
        let post = make_collection_post("X", None, Some(vec![uri]));
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_collection_post_rejects_301_char_uri() {
        let prefix = "pubky://userA/pub/pubky.app/posts/";
        let pad = "x".repeat(301 - prefix.chars().count());
        let uri = format!("{}{}", prefix, pad);
        assert_eq!(uri.chars().count(), 301);
        let post = make_collection_post("X", None, Some(vec![uri]));
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("URI exceeds"));
    }

    #[test]
    fn test_postkind_collection_display_lowercase() {
        assert_eq!(PubkyAppPostKind::Collection.to_string(), "collection");
    }

    #[test]
    fn test_postkind_fromstr_collection() {
        assert_eq!(
            PubkyAppPostKind::from_str("collection").unwrap(),
            PubkyAppPostKind::Collection
        );
    }

    #[test]
    fn test_collection_post_accepts_zero_items() {
        // Curators may create a draft and add items later via edits.
        let post = make_collection_post("Drafts", None, None);
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_collection_envelope_tolerates_extra_fields() {
        // Forward-compat: the envelope intentionally does NOT use deny_unknown_fields,
        // so future minor versions can add fields like `cover_image` without breaking
        // older parsers. This test locks in that behavior.
        let envelope_json =
            r#"{"name":"X","description":"Y","cover_image":"https://example.com/x.png"}"#;
        let post = PubkyAppPost::new(
            envelope_json.to_string(),
            PubkyAppPostKind::Collection,
            None,
            None,
            None,
        );
        let id = post.create_id();
        assert!(
            post.validate(Some(&id)).is_ok(),
            "extra envelope fields must be tolerated"
        );
    }

    #[test]
    fn test_collection_post_rejects_disallowed_attachment_protocol() {
        let post =
            make_collection_post("X", None, Some(vec!["ftp://example.com/file".to_string()]));
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Disallowed attachment protocol"));
    }

    #[test]
    fn test_existing_post_kinds_unchanged_with_collection() {
        // Regression: each of the six legacy lowercase kinds still round-trips after
        // adding Collection. Catches accidental ordering / serde changes.
        for s in ["short", "long", "image", "video", "link", "file"] {
            let json = format!(
                r#"{{"content":"x","kind":"{}","parent":null,"embed":null,"attachments":null}}"#,
                s
            );
            let post: PubkyAppPost = serde_json::from_str(&json).unwrap();
            let re = serde_json::to_value(&post.kind).unwrap();
            assert_eq!(re.as_str(), Some(s), "kind={} did not round-trip", s);
        }
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn test_postkind_collection_wasm_getter() {
        let post = PubkyAppPost {
            content: collection_envelope_json("X", None),
            kind: PubkyAppPostKind::Collection,
            parent: None,
            embed: None,
            attachments: None,
        };
        assert_eq!(post.kind(), "Collection");
    }
}
