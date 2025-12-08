use crate::{
    traits::{HasPath, Validatable},
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

// Validation constants
const MIN_USERNAME_LENGTH: usize = 3;
const MAX_USERNAME_LENGTH: usize = 50;
const MAX_BIO_LENGTH: usize = 160;
const MAX_IMAGE_LENGTH: usize = 300;
const MAX_LINKS: usize = 5;
const MAX_LINK_TITLE_LENGTH: usize = 100;
const MAX_LINK_URL_LENGTH: usize = 300;
const MAX_STATUS_LENGTH: usize = 50;

/// URI: /pub/pubky.app/profile.json
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppUser {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    // Avoid wasm-pack automatically generating getter/setters for the pub fields.
    pub name: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub bio: Option<String>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub image: Option<String>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub links: Option<Vec<PubkyAppUserLink>>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub status: Option<String>,
}

impl Default for PubkyAppUser {
    fn default() -> Self {
        PubkyAppUser {
            name: "anonymous".to_string(),
            bio: None,
            image: None,
            links: None,
            status: None,
        }
        .sanitize()
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppUser {
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
    pub fn links(&self) -> Option<Vec<PubkyAppUserLink>> {
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
impl Json for PubkyAppUser {}

/// Represents a user's single link with a title and URL.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppUserLink {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub title: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub url: String,
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppUserLink {
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
impl PubkyAppUser {
    /// Creates a new `PubkyAppUser` instance and sanitizes it.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(
        name: String,
        bio: Option<String>,
        image: Option<String>,
        links: Option<Vec<PubkyAppUserLink>>,
        status: Option<String>,
    ) -> Self {
        Self {
            name,
            bio,
            image,
            links,
            status,
        }
        .sanitize()
    }
}

impl HasPath for PubkyAppUser {
    const PATH_SEGMENT: &'static str = "profile.json";

    fn create_path() -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT].concat()
    }
}

impl Validatable for PubkyAppUser {
    fn sanitize(self) -> Self {
        // Sanitize name: trim whitespace only
        let mut name = self.name.trim().to_string();

        // We use username keyword `[DELETED]` for a user whose `profile.json` has been deleted
        // Therefore this is not a valid username.
        if name == *"[DELETED]" {
            name = "anonymous".to_string(); // default username
        }

        // Sanitize bio: trim whitespace only
        let bio = self.bio.map(|b| b.trim().to_string());

        // Sanitize image URL with URL parsing
        let image = match &self.image {
            Some(image_url) => {
                let sanitized_image_url = image_url.trim();

                match Url::parse(sanitized_image_url) {
                    Ok(_) => Some(sanitized_image_url.to_string()), // Valid image URL, normalized
                    Err(_) => None, // Invalid image URL, set to None
                }
            }
            None => None,
        };

        // Sanitize status: trim whitespace only
        let status = self.status.map(|s| s.trim().to_string());

        // Sanitize links: sanitize each link and filter out empty URLs
        let links = self.links.map(|links_vec| {
            links_vec
                .into_iter()
                .map(|link| link.sanitize())
                .filter(|link| !link.url.is_empty())
                .collect()
        });

        PubkyAppUser {
            name,
            bio,
            image,
            links,
            status,
        }
    }

    fn validate(&self, _id: Option<&str>) -> Result<(), String> {
        // Validate name length
        let name_length = self.name.chars().count();
        if !(MIN_USERNAME_LENGTH..=MAX_USERNAME_LENGTH).contains(&name_length) {
            return Err("Validation Error: Invalid name length".into());
        }

        // Validate bio length
        if let Some(bio) = &self.bio {
            if bio.chars().count() > MAX_BIO_LENGTH {
                return Err("Validation Error: Bio exceeds maximum length".into());
            }
        }

        // Validate image URL format and length
        if let Some(image) = &self.image {
            if image.is_empty() {
                return Err("Validation Error: Image URI cannot be empty".into());
            }
            if image.chars().count() > MAX_IMAGE_LENGTH {
                return Err("Validation Error: Image URI exceeds maximum length".into());
            }
            // Validate URL format
            Url::parse(image)
                .map_err(|_| "Validation Error: Invalid image URI format".to_string())?;
        }

        // Validate links
        if let Some(links) = &self.links {
            if links.len() > MAX_LINKS {
                return Err("Validation Error: Too many links".into());
            }

            for link in links {
                link.validate(None)?;
            }
        }

        // Validate status length
        if let Some(status) = &self.status {
            if status.chars().count() > MAX_STATUS_LENGTH {
                return Err("Validation Error: Status exceeds maximum length".into());
            }
        }

        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppUserLink {
    /// Creates a new `PubkyAppUserLink` instance and sanitizes it.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(title: String, url: String) -> Self {
        Self { title, url }.sanitize()
    }
}

impl Validatable for PubkyAppUserLink {
    fn sanitize(self) -> Self {
        // Sanitize title: trim whitespace only
        let title = self.title.trim().to_string();

        // Sanitize URL: trim and normalize URL format
        let url = match Url::parse(self.url.trim()) {
            Ok(parsed_url) => parsed_url.to_string(), // Valid URL, normalized
            Err(_) => "".to_string(),                 // Default to empty string for invalid URLs
        };

        PubkyAppUserLink { title, url }
    }

    fn validate(&self, _id: Option<&str>) -> Result<(), String> {
        // Validate title
        if self.title.trim().is_empty() {
            return Err("Validation Error: Link title cannot be empty".into());
        }
        if self.title.chars().count() > MAX_LINK_TITLE_LENGTH {
            return Err("Validation Error: Link title exceeds maximum length".into());
        }

        // Validate URL
        if self.url.trim().is_empty() {
            return Err("Validation Error: Link URL cannot be empty".into());
        }
        if self.url.chars().count() > MAX_LINK_URL_LENGTH {
            return Err("Validation Error: Link URL exceeds maximum length".into());
        }

        // Validate URL format
        Url::parse(&self.url).map_err(|_| "Validation Error: Invalid URL format".to_string())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Validatable;
    use crate::{APP_PATH, PUBLIC_PATH};

    #[test]
    fn test_new() {
        let user = PubkyAppUser::new(
            "Alice".to_string(),
            Some("Maximalist".to_string()),
            Some("https://example.com/image.png".to_string()),
            Some(vec![
                PubkyAppUserLink {
                    title: "GitHub".to_string(),
                    url: "https://github.com/alice".to_string(),
                },
                PubkyAppUserLink {
                    title: "Website".to_string(),
                    url: "https://alice.dev".to_string(),
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
    fn test_create_path() {
        let path = PubkyAppUser::create_path();
        assert_eq!(path, format!("{}{}profile.json", PUBLIC_PATH, APP_PATH));
    }

    #[test]
    fn test_sanitize() {
        let user = PubkyAppUser::new(
            "   Alice   ".to_string(),
            Some("  Maximalist and developer.  ".to_string()),
            Some("https://example.com/image.png".to_string()),
            Some(vec![
                PubkyAppUserLink {
                    title: " GitHub ".to_string(),
                    url: " https://github.com/alice ".to_string(),
                },
                PubkyAppUserLink {
                    title: "Website".to_string(),
                    url: "invalid_url".to_string(), // Invalid URL
                },
            ]),
            Some("  Exploring the decentralized web.  ".to_string()),
        );

        assert_eq!(user.name, "Alice");
        assert_eq!(user.bio.as_deref(), Some("Maximalist and developer."));
        assert_eq!(user.image.as_deref(), Some("https://example.com/image.png"));
        assert_eq!(
            user.status.as_deref(),
            Some("Exploring the decentralized web.")
        );
        assert!(user.links.is_some());
        let links = user.links.unwrap();
        assert_eq!(links.len(), 1); // Invalid URL link should be filtered out
        assert_eq!(links[0].title, "GitHub");
        assert_eq!(links[0].url, "https://github.com/alice");
    }

    #[test]
    fn test_validate() {
        let user = PubkyAppUser::new(
            "Alice".to_string(),
            Some("Maximalist".to_string()),
            Some("https://example.com/image.png".to_string()),
            None,
            Some("Exploring the decentralized web.".to_string()),
        );

        let result = user.validate(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_name() {
        // Test name too short
        let user = PubkyAppUser::new(
            "Al".to_string(), // Too short
            None,
            None,
            None,
            None,
        );

        let result = user.validate(None);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Validation Error: Invalid name length"
        );

        // Test name too long - sanitization should NOT truncate
        let long_name = "a".repeat(MAX_USERNAME_LENGTH + 1);
        let user = PubkyAppUser::new(long_name.clone(), None, None, None, None);

        // Sanitization should preserve full length
        assert_eq!(user.name.len(), MAX_USERNAME_LENGTH + 1);

        // Validation should catch the violation
        let result = user.validate(None);
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
        let user = <PubkyAppUser as Validatable>::try_from(blob, "").unwrap();

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
        let user = <PubkyAppUser as Validatable>::try_from(blob, "").unwrap();

        // Since the link URL is invalid, it should be filtered out
        assert!(user.links.is_none() || user.links.as_ref().unwrap().is_empty());
    }

    #[test]
    fn test_sanitize_preserves_length() {
        // Test that sanitization does NOT truncate, even if over limits
        let long_bio = "a".repeat(MAX_BIO_LENGTH + 10);
        let long_status = "b".repeat(MAX_STATUS_LENGTH + 10);
        let long_image = format!(
            "https://example.com/{}.png",
            "a".repeat(MAX_IMAGE_LENGTH - 30)
        );

        let user = PubkyAppUser::new(
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
                PubkyAppUser::new(
                    "Alice".to_string(),
                    Some("a".repeat(MAX_BIO_LENGTH + 1)),
                    None,
                    None,
                    None,
                ),
                "bio",
            ),
            (
                PubkyAppUser::new(
                    "Alice".to_string(),
                    None,
                    None,
                    None,
                    Some("a".repeat(MAX_STATUS_LENGTH + 1)),
                ),
                "status",
            ),
            (
                PubkyAppUser::new(
                    "Alice".to_string(),
                    None,
                    Some(format!(
                        "https://example.com/{}.png",
                        "a".repeat(MAX_IMAGE_LENGTH - 20)
                    )),
                    None,
                    None,
                ),
                "image",
            ),
        ];

        for (user, field_name) in test_cases {
            let result = user.validate(None);
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
        for i in 0..MAX_LINKS + 1 {
            links.push(PubkyAppUserLink {
                title: format!("Link {}", i),
                url: format!("https://example.com/{}", i),
            });
        }

        let user = PubkyAppUser::new("Alice".to_string(), None, None, Some(links), None);

        // Sanitization should preserve all links (not truncate)
        assert_eq!(user.links.as_ref().unwrap().len(), MAX_LINKS + 1);

        // Validation should catch the violation
        let result = user.validate(None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Too many links"));
    }

    #[test]
    fn test_validate_link_length_errors() {
        // Test link title too long
        let long_title = "a".repeat(MAX_LINK_TITLE_LENGTH + 1);
        let link = PubkyAppUserLink {
            title: long_title.clone(),
            url: "https://example.com".to_string(),
        };
        let sanitized = link.sanitize();
        assert_eq!(sanitized.title.len(), MAX_LINK_TITLE_LENGTH + 1);
        let result = sanitized.validate(None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds maximum length"));

        // Test link URL too long - create URL that exceeds limit after normalization
        let very_long_path = "a".repeat(MAX_LINK_URL_LENGTH);
        let very_long_url = format!("https://example.com/{}", very_long_path);
        let link2 = PubkyAppUserLink {
            title: "Test".to_string(),
            url: very_long_url,
        };
        let sanitized2 = link2.sanitize();

        // Verify URL exceeds limit (accounting for potential normalization)
        if sanitized2.url.chars().count() > MAX_LINK_URL_LENGTH {
            let result = sanitized2.validate(None);
            assert!(
                result.is_err(),
                "Expected validation error for URL length {}, max is {}",
                sanitized2.url.chars().count(),
                MAX_LINK_URL_LENGTH
            );
            assert!(result.unwrap_err().contains("exceeds maximum length"));
        } else {
            // If normalization shortened it, create an even longer one
            let extremely_long_path = "a".repeat(MAX_LINK_URL_LENGTH + 50);
            let extremely_long_url = format!("https://example.com/{}", extremely_long_path);
            let link3 = PubkyAppUserLink {
                title: "Test".to_string(),
                url: extremely_long_url,
            };
            let sanitized3 = link3.sanitize();
            let result = sanitized3.validate(None);
            assert!(
                result.is_err(),
                "Expected validation error for URL length {}, max is {}",
                sanitized3.url.chars().count(),
                MAX_LINK_URL_LENGTH
            );
            assert!(result.unwrap_err().contains("exceeds maximum length"));
        }
    }

    #[test]
    fn test_unicode_character_counting() {
        // Emoji name: 3 emoji characters (each is 1 char but multiple bytes)
        // This verifies .chars().count() is used instead of .len()
        let emoji_name = "Hi👋🏻Bob"; // 7 characters: H, i, 👋, 🏻, B, o, b
        let user = PubkyAppUser::new(emoji_name.to_string(), None, None, None, None);
        assert!(
            user.validate(None).is_ok(),
            "Should accept emoji in name (counts chars, not bytes)"
        );

        // Unicode bio with various scripts
        let unicode_bio = "你好世界 🌍 مرحبا"; // Mix of Chinese, emoji, Arabic
        let user_with_bio = PubkyAppUser::new(
            "Alice".to_string(),
            Some(unicode_bio.to_string()),
            None,
            None,
            None,
        );
        assert!(
            user_with_bio.validate(None).is_ok(),
            "Should accept multi-script Unicode in bio"
        );

        // Test that emoji-heavy string at max length passes
        // MAX_USERNAME_LENGTH is 50, so 50 emoji should pass
        let max_emoji_name: String = "🔥".repeat(MAX_USERNAME_LENGTH);
        assert_eq!(max_emoji_name.chars().count(), MAX_USERNAME_LENGTH);
        let user_max_emoji = PubkyAppUser::new(max_emoji_name, None, None, None, None);
        assert!(
            user_max_emoji.validate(None).is_ok(),
            "Should accept {} emoji characters as name",
            MAX_USERNAME_LENGTH
        );
    }
}
