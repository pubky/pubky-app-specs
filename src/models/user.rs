use crate::canonicalize::{checked, AllowedSchemes};
use crate::constants::social_path;
use crate::traits::{Root, ValidationCtx, ValidationError};
use crate::{
    common::{check_extra, code_point_len, frozen_trim},
    limits::VALIDATION_LIMITS,
    traits::{HasPath, Validatable},
};
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use crate::traits::Json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// URI: /pub/social/v1/profile.json
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkySocialUser {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    // Avoid wasm-pack automatically generating getter/setters for the pub fields.
    pub name: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub bio: Option<String>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub image: Option<String>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub links: Option<Vec<PubkySocialUserLink>>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub status: Option<String>,
    /// Unknown members, preserved on rewrite; see the module contract in `models/mod.rs`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Default for PubkySocialUser {
    fn default() -> Self {
        PubkySocialUser {
            name: "anonymous".to_string(),
            bio: None,
            image: None,
            links: None,
            status: None,
            extra: Default::default(),
        }
        .sanitize()
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkySocialUser {
    // Getters clone the data out because String/JsValue is not Copy.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn name(&self) -> String {
        self.name.clone()
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn bio(&self) -> Option<String> {
        self.bio.clone()
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn image(&self) -> Option<String> {
        self.image.clone()
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn links(&self) -> Option<Vec<PubkySocialUserLink>> {
        self.links.clone()
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn status(&self) -> Option<String> {
        self.status.clone()
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
impl Json for PubkySocialUser {}

/// Represents a user's single link with a title and URL.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkySocialUserLink {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub title: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub url: String,
    /// Unknown members, preserved on rewrite; see the module contract in `models/mod.rs`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkySocialUserLink {
    // Getters clone the data out because String/JsValue is not Copy.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn title(&self) -> String {
        self.title.clone()
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn url(&self) -> String {
        self.url.clone()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkySocialUser {
    /// Creates a new `PubkySocialUser` instance and sanitizes it.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(
        name: String,
        bio: Option<String>,
        image: Option<String>,
        links: Option<Vec<PubkySocialUserLink>>,
        status: Option<String>,
    ) -> Self {
        Self {
            name,
            bio,
            image,
            links,
            status,
            extra: Default::default(),
        }
        .sanitize()
    }
}

impl HasPath for PubkySocialUser {
    const ROOT: Root = Root::Pub;
    const PATH_SEGMENT: &'static str = "profile.json";

    fn create_path() -> String {
        social_path(Self::ROOT, Self::PATH_SEGMENT)
    }
}

impl Validatable for PubkySocialUser {
    fn sanitize(self) -> Self {
        let name = frozen_trim(&self.name).to_string();
        let bio = self.bio.map(|b| frozen_trim(&b).to_string());

        // The image is a reference: stored as written, so validation can hold it to its
        // canonical form instead of a rewrite hiding the difference.
        let image = self.image;

        // Sanitize status: trim whitespace only
        let status = self.status.map(|s| frozen_trim(&s).to_string());

        // Sanitize links: sanitize each link, validation handles format
        let links = self
            .links
            .map(|links_vec| links_vec.into_iter().map(|link| link.sanitize()).collect());

        PubkySocialUser {
            name,
            bio,
            image,
            links,
            status,
            extra: self.extra,
        }
    }

    fn validate_fields(
        &self,
        _id: Option<&str>,
        _ctx: &ValidationCtx,
    ) -> Result<(), ValidationError> {
        // The profile has one root, so the destination is the model's, whatever ctx a caller
        // hands in: a private image never passes the root rule here
        let ctx = &ValidationCtx { root: Self::ROOT };
        check_extra(&self.extra, &["name", "bio", "image", "links", "status"])?;

        // Validate name length
        let name_length = code_point_len(&self.name);
        if !(VALIDATION_LIMITS.user_name_min_length..=VALIDATION_LIMITS.user_name_max_length)
            .contains(&name_length)
        {
            return Err("Validation Error: Invalid name length".into());
        }

        // Validate bio length
        if let Some(bio) = &self.bio {
            if code_point_len(bio) > VALIDATION_LIMITS.user_bio_max_length {
                return Err("Validation Error: Bio exceeds maximum length".into());
            }
        }

        // The profile is public, so a private image is refused by the root rule with no owner
        if let Some(image) = &self.image {
            checked(
                "image",
                image,
                AllowedSchemes::PubkyHttpHttps,
                VALIDATION_LIMITS.image_url_max_length,
                ctx,
                None,
            )?;
        }

        // Validate links
        if let Some(links) = &self.links {
            if links.len() > VALIDATION_LIMITS.user_links_max_count {
                return Err("Validation Error: Too many links".into());
            }

            for (index, link) in links.iter().enumerate() {
                link.validate_at(Some(index), ctx)?;
            }
        }

        // Validate status length
        if let Some(status) = &self.status {
            if code_point_len(status) > VALIDATION_LIMITS.user_status_max_length {
                return Err("Validation Error: Status exceeds maximum length".into());
            }
        }

        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkySocialUserLink {
    /// Creates a new `PubkySocialUserLink` instance and sanitizes it.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(title: String, url: String) -> Self {
        Self {
            title,
            url,
            extra: Default::default(),
        }
        .sanitize()
    }
}

impl PubkySocialUserLink {
    /// The rules, with each field named as the caller sees it: `links[2].url` inside a
    /// profile, the bare field name when a link is validated on its own.
    fn validate_at(&self, index: Option<usize>, ctx: &ValidationCtx) -> Result<(), String> {
        let field = |name: &str| match index {
            Some(i) => format!("links[{i}].{name}"),
            None => name.to_string(),
        };
        check_extra(&self.extra, &["title", "url"])?;
        if frozen_trim(&self.title).is_empty() {
            return Err(format!(
                "Validation Error: {} must not be blank",
                field("title")
            ));
        }
        if code_point_len(&self.title) > VALIDATION_LIMITS.user_link_title_max_length {
            return Err(format!(
                "Validation Error: {} must be at most {} code points",
                field("title"),
                VALIDATION_LIMITS.user_link_title_max_length
            ));
        }
        checked(
            &field("url"),
            &self.url,
            AllowedSchemes::HttpHttps,
            VALIDATION_LIMITS.user_link_url_max_length,
            ctx,
            None,
        )
    }
}

impl Validatable for PubkySocialUserLink {
    fn sanitize(self) -> Self {
        PubkySocialUserLink {
            title: frozen_trim(&self.title).to_string(),
            // A reference, stored as written; see the image note above.
            url: self.url,
            extra: self.extra,
        }
    }

    fn validate_fields(
        &self,
        _id: Option<&str>,
        ctx: &ValidationCtx,
    ) -> Result<(), ValidationError> {
        self.validate_at(None, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{Validatable, PUB_CTX};

    #[test]
    fn test_new() {
        let user = PubkySocialUser::new(
            "Alice".to_string(),
            Some("Maximalist".to_string()),
            Some("https://example.com/image.png".to_string()),
            Some(vec![
                PubkySocialUserLink {
                    title: "GitHub".to_string(),
                    url: "https://github.com/alice".to_string(),
                    extra: Default::default(),
                },
                PubkySocialUserLink {
                    title: "Website".to_string(),
                    url: "https://alice.dev".to_string(),
                    extra: Default::default(),
                },
            ]),
            Some("Exploring the decentralized web.".to_string()),
        );

        assert_eq!(user.name, "Alice");
        assert_eq!(user.bio.as_deref(), Some("Maximalist"));
        assert_eq!(user.image.as_deref(), Some("https://example.com/image.png"));
        assert_eq!(
            user.status.as_deref(),
            Some("Exploring the decentralized web.")
        );
        assert!(user.links.is_some());
        assert_eq!(user.links.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_deleted_literal_is_an_ordinary_name() {
        let blob = br#"{"name":"[DELETED]"}"#;
        let user = <PubkySocialUser as Validatable>::try_from(blob, "", &PUB_CTX).unwrap();
        assert_eq!(user.name, "[DELETED]");
    }

    #[test]
    fn test_unknown_members_survive_rewrite() {
        let json = r#"{"name":"Alice","links":[{"title":"t","url":"https://a.b/","rel":"me"}],"ext":{"badge":1}}"#;
        let user: PubkySocialUser = serde_json::from_str(json).unwrap();
        let out: serde_json::Value = serde_json::to_value(&user).unwrap();
        assert_eq!(out["ext"]["badge"], 1);
        assert_eq!(out["links"][0]["rel"], "me");
        assert_eq!(out["name"], "Alice");
    }

    #[test]
    fn test_size_cap() {
        let cap = VALIDATION_LIMITS.object_max_bytes;
        let blob = |len: usize| {
            // Compact and complete, so the reserialized form has the same length
            let head =
                r#"{"name":"Alice","bio":null,"image":null,"links":null,"status":null,"pad":""#;
            let mut s = head.to_string();
            s.push_str(&"a".repeat(len - head.len() - 2));
            s.push_str(r#""}"#);
            assert_eq!(s.len(), len);
            s.into_bytes()
        };
        assert!(<PubkySocialUser as Validatable>::try_from(&blob(cap), "", &PUB_CTX).is_ok());
        let err =
            <PubkySocialUser as Validatable>::try_from(&blob(cap + 1), "", &PUB_CTX).unwrap_err();
        assert!(err.contains("exceeds"), "{err}");
    }

    #[test]
    fn test_frozen_trim_and_code_points() {
        let user = PubkySocialUser::new(
            "\u{3000}Alice\u{3000}".to_string(),
            Some("\u{1F600}".repeat(VALIDATION_LIMITS.user_bio_max_length)),
            None,
            None,
            Some("\u{200B}ok".to_string()),
        );
        assert_eq!(user.name, "Alice");
        assert_eq!(user.status.as_deref(), Some("\u{200B}ok"));
        assert!(user.validate(None, &PUB_CTX).is_ok());
    }

    #[test]
    fn test_create_path() {
        let path = PubkySocialUser::create_path();
        assert_eq!(path, "/pub/social/v1/profile.json");
    }

    #[test]
    fn test_sanitize() {
        let user = PubkySocialUser::new(
            "   Alice   ".to_string(),
            Some("  Maximalist and developer.  ".to_string()),
            Some("  https://example.com/image.png  ".to_string()),
            Some(vec![
                PubkySocialUserLink {
                    title: " GitHub ".to_string(),
                    url: " https://github.com/alice ".to_string(),
                    extra: Default::default(),
                },
                PubkySocialUserLink {
                    title: "Website".to_string(),
                    url: "  https://example.com  ".to_string(),
                    extra: Default::default(),
                },
            ]),
            Some("  Exploring the decentralized web.  ".to_string()),
        );

        assert_eq!(user.name, "Alice");
        assert_eq!(user.bio.as_deref(), Some("Maximalist and developer."));
        // The padded image survives the builder and is rejected, never repaired
        assert_eq!(
            user.image.as_deref(),
            Some("  https://example.com/image.png  ")
        );
        assert_eq!(
            user.status.as_deref(),
            Some("Exploring the decentralized web.")
        );
        let e = user.validate(None, &PUB_CTX).unwrap_err();
        assert!(e.contains("image") && e.contains("canonical"), "{e}");

        let links = user.links.unwrap();
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].title, "GitHub");
        assert_eq!(links[0].url, " https://github.com/alice ");
        assert_eq!(links[1].title, "Website");
        assert_eq!(links[1].url, "  https://example.com  ");
        let e = links[0].validate(None, &PUB_CTX).unwrap_err();
        assert!(e.contains("url") && e.contains("canonical"), "{e}");
    }

    #[test]
    fn test_validate() {
        let user = PubkySocialUser::new(
            "Alice".to_string(),
            Some("Maximalist".to_string()),
            Some("https://example.com/image.png".to_string()),
            None,
            Some("Exploring the decentralized web.".to_string()),
        );

        let result = user.validate(None, &PUB_CTX);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_name() {
        // Test name too short
        let user = PubkySocialUser::new(
            "Al".to_string(), // Too short
            None,
            None,
            None,
            None,
        );

        let result = user.validate(None, &PUB_CTX);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Validation Error: Invalid name length"
        );

        // Test name too long - sanitization should NOT truncate
        let long_name = "a".repeat(VALIDATION_LIMITS.user_name_max_length + 1);
        let user = PubkySocialUser::new(long_name.clone(), None, None, None, None);

        // Sanitization should preserve full length
        assert_eq!(user.name.len(), VALIDATION_LIMITS.user_name_max_length + 1);

        // Validation should catch the violation
        let result = user.validate(None, &PUB_CTX);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid name length"));
    }

    #[test]
    fn test_try_from_valid() {
        let user_json = r#"
        {
            "name": "Alice",
            "bio": "Maximalist",
            "image": "https://example.com/image.png",
            "links": [
                {
                    "title": "GitHub",
                    "url": "https://github.com/alice"
                },
                {
                    "title": "Website",
                    "url": "https://alice.dev"
                }
            ],
            "status": "Exploring the decentralized web."
        }
        "#;

        let blob = user_json.as_bytes();
        let user = <PubkySocialUser as Validatable>::try_from(blob, "", &PUB_CTX).unwrap();

        assert_eq!(user.name, "Alice");
        assert_eq!(user.bio.as_deref(), Some("Maximalist"));
        assert_eq!(user.image.as_deref(), Some("https://example.com/image.png"));
        assert_eq!(
            user.status.as_deref(),
            Some("Exploring the decentralized web.")
        );
        assert!(user.links.is_some());
        assert_eq!(user.links.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_try_from_invalid_link() {
        let user_json = r#"
        {
            "name": "Alice",
            "links": [
                {
                    "title": "GitHub",
                    "url": "invalid_url"
                }
            ]
        }
        "#;

        let blob = user_json.as_bytes();
        let result = <PubkySocialUser as Validatable>::try_from(blob, "", &PUB_CTX);

        // Invalid link URL should cause validation to fail
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Validation Error: links[0].url must be a canonical web URI of at most 300 code points: invalid_url"
        );
    }

    #[test]
    fn test_validate_invalid_image_url() {
        let user_json = r#"
        {
            "name": "Alice",
            "image": "invalid_image_url"
        }
        "#;

        let blob = user_json.as_bytes();
        let result = <PubkySocialUser as Validatable>::try_from(blob, "", &PUB_CTX);

        // Invalid image URL should cause validation to fail
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Validation Error: image must be a canonical pubky or web URI of at most 300 code points: invalid_image_url"
        );
    }

    #[test]
    fn test_sanitize_preserves_invalid_urls() {
        // Sanitize should preserve invalid URLs (just trim), validation rejects them
        let user = PubkySocialUser {
            name: "Alice".to_string(),
            bio: None,
            image: Some("  invalid_image_url  ".to_string()),
            links: Some(vec![PubkySocialUserLink {
                title: "Test".to_string(),
                url: "  invalid_link_url  ".to_string(),
                extra: Default::default(),
            }]),
            status: None,
            extra: Default::default(),
        };

        let sanitized = user.sanitize();

        // Image is stored exactly as written
        assert_eq!(sanitized.image.as_deref(), Some("  invalid_image_url  "));

        // So is the link url
        let links = sanitized.links.as_ref().unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "  invalid_link_url  ");

        // Validation should reject
        let result = sanitized.validate(None, &PUB_CTX);
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_preserves_length() {
        // Test that sanitization does NOT truncate, even if over limits
        let long_bio = "a".repeat(VALIDATION_LIMITS.user_bio_max_length + 10);
        let long_status = "b".repeat(VALIDATION_LIMITS.user_status_max_length + 10);
        let long_image = format!(
            "https://example.com/{}.png",
            "a".repeat(VALIDATION_LIMITS.image_url_max_length - 30)
        );

        let user = PubkySocialUser::new(
            "Alice".to_string(),
            Some(long_bio.clone()),
            Some(long_image.clone()),
            None,
            Some(long_status.clone()),
        );

        // Sanitization should preserve full length (only trim whitespace)
        assert_eq!(user.bio.as_deref(), Some(long_bio.as_str()));
        assert_eq!(user.status.as_deref(), Some(long_status.as_str()));
        assert_eq!(user.image.as_deref(), Some(long_image.as_str()));
    }

    #[test]
    fn test_validate_field_length_errors() {
        // Test multiple field length validation errors
        let test_cases = vec![
            (
                PubkySocialUser::new(
                    "Alice".to_string(),
                    Some("a".repeat(VALIDATION_LIMITS.user_bio_max_length + 1)),
                    None,
                    None,
                    None,
                ),
                "bio",
            ),
            (
                PubkySocialUser::new(
                    "Alice".to_string(),
                    None,
                    None,
                    None,
                    Some("a".repeat(VALIDATION_LIMITS.user_status_max_length + 1)),
                ),
                "status",
            ),
        ];

        for (user, field_name) in test_cases {
            let result = user.validate(None, &PUB_CTX);
            assert!(
                result.is_err(),
                "Should reject {} that exceeds maximum length",
                field_name
            );
            assert!(result.unwrap_err().contains("exceeds maximum length"));
        }
    }

    #[test]
    fn test_validate_too_many_links() {
        let mut links = Vec::new();
        for i in 0..VALIDATION_LIMITS.user_links_max_count + 1 {
            links.push(PubkySocialUserLink {
                title: format!("Link {}", i),
                url: format!("https://example.com/{}", i),
                extra: Default::default(),
            });
        }

        let user = PubkySocialUser::new("Alice".to_string(), None, None, Some(links), None);

        // Sanitization should preserve all links (not truncate)
        assert_eq!(
            user.links.as_ref().unwrap().len(),
            VALIDATION_LIMITS.user_links_max_count + 1
        );

        // Validation should catch the violation
        let result = user.validate(None, &PUB_CTX);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Too many links"));
    }

    #[test]
    fn test_validate_link_length_errors() {
        // Test link title too long
        let long_title = "a".repeat(VALIDATION_LIMITS.user_link_title_max_length + 1);
        let link = PubkySocialUserLink {
            title: long_title.clone(),
            url: "https://example.com".to_string(),
            extra: Default::default(),
        };
        let sanitized = link.sanitize();
        assert_eq!(
            sanitized.title.len(),
            VALIDATION_LIMITS.user_link_title_max_length + 1
        );
        let result = sanitized.validate(None, &PUB_CTX);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("title must be at most"));

        // The url cap is measured on the stored form, at the gate
        let max = VALIDATION_LIMITS.user_link_url_max_length;
        let url = |len: usize| format!("https://x.com/{}", "a".repeat(len - 14));
        let at_cap = PubkySocialUserLink::new("Test".to_string(), url(max));
        assert!(at_cap.validate(None, &PUB_CTX).is_ok());
        let over = PubkySocialUserLink::new("Test".to_string(), url(max + 1));
        let e = over.validate(None, &PUB_CTX).unwrap_err();
        assert!(
            e.contains("url") && e.contains(&format!("at most {max} code points")),
            "{e}"
        );
    }

    #[test]
    fn test_unicode_character_counting() {
        // Emoji name: 3 emoji characters (each is 1 char but multiple bytes)
        // This verifies .chars().count() is used instead of .len()
        let emoji_name = "Hi👋🏻Bob"; // 7 characters: H, i, 👋, 🏻, B, o, b
        let user = PubkySocialUser::new(emoji_name.to_string(), None, None, None, None);
        assert!(
            user.validate(None, &PUB_CTX).is_ok(),
            "Should accept emoji in name (counts chars, not bytes)"
        );

        // Unicode bio with various scripts
        let unicode_bio = "你好世界 🌍 مرحبا"; // Mix of Chinese, emoji, Arabic
        let user_with_bio = PubkySocialUser::new(
            "Alice".to_string(),
            Some(unicode_bio.to_string()),
            None,
            None,
            None,
        );
        assert!(
            user_with_bio.validate(None, &PUB_CTX).is_ok(),
            "Should accept multi-script Unicode in bio"
        );

        // Test that emoji-heavy string at max length passes
        // VALIDATION_LIMITS.user_name_max_length is 50, so 50 emoji should pass
        let max_emoji_name: String = "🔥".repeat(VALIDATION_LIMITS.user_name_max_length);
        assert_eq!(
            max_emoji_name.chars().count(),
            VALIDATION_LIMITS.user_name_max_length
        );
        let user_max_emoji = PubkySocialUser::new(max_emoji_name, None, None, None, None);
        assert!(
            user_max_emoji.validate(None, &PUB_CTX).is_ok(),
            "Should accept {} emoji characters as name",
            VALIDATION_LIMITS.user_name_max_length
        );
    }

    #[test]
    fn test_create_user_with_empty_image() {
        // Test what happens when image is an empty string
        let user = PubkySocialUser::new(
            "Alice".to_string(),
            None,
            Some("".to_string()), // Empty string for image
            None,
            None,
        );

        // After sanitization, image is still Some("")
        assert_eq!(user.image, Some("".to_string()));

        // Validation should fail: an empty string is not a canonical URI
        let result = user.validate(None, &PUB_CTX);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Validation Error: image must be a canonical pubky or web URI of at most 300 code points: "
        );
    }

    #[test]
    fn test_create_user_without_image() {
        // Test what happens when image is None
        let user = PubkySocialUser::new(
            "Alice".to_string(),
            None,
            None, // No image provided
            None,
            None,
        );

        // Image should be None
        assert_eq!(user.image, None);

        // Validation should pass - image is optional
        let result = user.validate(None, &PUB_CTX);
        assert!(result.is_ok());
    }

    const HOST: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";

    fn with_image(image: &str) -> PubkySocialUser {
        PubkySocialUser::new(
            "Alice".to_string(),
            None,
            Some(image.to_string()),
            None,
            None,
        )
    }

    #[test]
    fn test_image_goes_through_the_reference_gate() {
        for ok in [
            format!("pubky://{HOST}/pub/social/v1/files/0032SSN7Q4EVG"),
            "https://x.com/a.png".to_string(),
            "http://x.com/a.png".to_string(),
        ] {
            assert!(with_image(&ok).validate(None, &PUB_CTX).is_ok(), "{ok}");
        }
        for bad in [
            // outside the field's scheme set
            "ipfs://x".to_string(),
            // the SDK short form is not the stored spelling
            format!("pubky{HOST}/pub/social/v1/files/0032SSN7Q4EVG"),
            // padding is not the canonical spelling either
            " https://x.com/a.png".to_string(),
        ] {
            let e = with_image(&bad).validate(None, &PUB_CTX).unwrap_err();
            assert!(e.contains("image"), "{bad}: {e}");
        }
    }

    #[test]
    fn test_image_cap_is_300_code_points() {
        let max = VALIDATION_LIMITS.image_url_max_length;
        let url = |len: usize| format!("https://x.com/{}", "a".repeat(len - 14));
        assert_eq!(code_point_len(&url(max)), max);
        assert!(with_image(&url(max)).validate(None, &PUB_CTX).is_ok());
        let e = with_image(&url(max + 1))
            .validate(None, &PUB_CTX)
            .unwrap_err();
        assert!(
            e.contains("image") && e.contains(&format!("at most {max} code points")),
            "{e}"
        );
    }

    #[test]
    fn test_private_image_fails_the_root_rule() {
        let e = with_image(&format!(
            "pubky://{HOST}/priv/social/v1/files/0032SSN7Q4EVG"
        ))
        .validate(None, &PUB_CTX)
        .unwrap_err();
        assert!(e.contains("image") && e.contains("public object"), "{e}");
    }

    #[test]
    fn test_profile_root_is_the_models_not_the_callers() {
        // A caller may hand in any ctx; the profile lives under pub, so a private image
        // fails the root rule regardless
        let user = with_image(&format!(
            "pubky://{HOST}/priv/social/v1/files/0032SSN7Q4EVG"
        ));
        let blob = serde_json::to_vec(&user).unwrap();
        let priv_ctx = ValidationCtx {
            root: crate::traits::Root::Priv,
        };
        let e = <PubkySocialUser as Validatable>::try_from(&blob, "", &priv_ctx).unwrap_err();
        assert!(e.contains("image") && e.contains("public object"), "{e}");
    }

    #[test]
    fn test_link_errors_name_the_index() {
        let mut user = PubkySocialUser {
            name: "Alice".into(),
            links: Some(vec![
                PubkySocialUserLink::new("ok".into(), "https://x.com/a".into()),
                PubkySocialUserLink::new("bad".into(), format!("pubky://{HOST}")),
                PubkySocialUserLink::new(" ".into(), "https://x.com/b".into()),
            ]),
            ..Default::default()
        };
        let e = user.validate(None, &PUB_CTX).unwrap_err();
        assert!(
            e.starts_with("Validation Error: links[1].url must be"),
            "{e}"
        );
        user.links.as_mut().unwrap().remove(1);
        let e = user.validate(None, &PUB_CTX).unwrap_err();
        assert_eq!(e, "Validation Error: links[1].title must not be blank");
        // standalone, the bare field names
        let e = PubkySocialUserLink::new(" ".into(), "https://x.com".into())
            .validate(None, &PUB_CTX)
            .unwrap_err();
        assert_eq!(e, "Validation Error: title must not be blank");
    }

    #[test]
    fn test_link_urls_are_web_only() {
        let link = |url: &str| PubkySocialUserLink::new("site".to_string(), url.to_string());
        for ok in ["https://x.com/a", "http://x.com/a"] {
            assert!(link(ok).validate(None, &PUB_CTX).is_ok(), "{ok}");
        }
        for bad in [
            format!("pubky://{HOST}/pub/social/v1/profile.json"),
            "HTTP://x.com/a".to_string(),
            String::new(),
        ] {
            let e = link(&bad).validate(None, &PUB_CTX).unwrap_err();
            assert!(
                e.contains("url") && e.contains("canonical web URI"),
                "{bad}: {e}"
            );
        }
    }

    #[test]
    fn test_default_user_validates() {
        assert!(PubkySocialUser::default().validate(None, &PUB_CTX).is_ok());
    }
}
