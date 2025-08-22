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
        // Sanitize name
        let sanitized_name = self.name.trim();
        // Crop name to a maximum length of MAX_USERNAME_LENGTH characters
        let mut name = sanitized_name
            .chars()
            .take(MAX_USERNAME_LENGTH)
            .collect::<String>();

        // We use username keyword `[DELETED]` for a user whose `profile.json` has been deleted
        // Therefore this is not a valid username.
        if name == *"[DELETED]" {
            name = "anonymous".to_string(); // default username
        }

        // Sanitize bio
        let bio = self
            .bio
            .map(|b| b.trim().chars().take(MAX_BIO_LENGTH).collect::<String>());

        // Sanitize image URL with URL parsing
        let image = match &self.image {
            Some(image_url) => {
                let sanitized_image_url = image_url.trim();

                match Url::parse(sanitized_image_url) {
                    Ok(_) => {
                        // Ensure the URL is within the allowed limit
                        let url = sanitized_image_url
                            .chars()
                            .take(MAX_IMAGE_LENGTH)
                            .collect::<String>();
                        Some(url) // Valid image URL
                    }
                    Err(_) => None, // Invalid image URL, set to None
                }
            }
            None => None,
        };

        // Sanitize status
        let status = self
            .status
            .map(|s| s.trim().chars().take(MAX_STATUS_LENGTH).collect::<String>());

        // Sanitize links
        let links = self.links.map(|links_vec| {
            links_vec
                .into_iter()
                .take(MAX_LINKS)
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

        // Validate image length
        if let Some(image) = &self.image {
            if image.chars().count() > MAX_IMAGE_LENGTH {
                return Err("Validation Error: Image URI exceeds maximum length".into());
            }
        }

        // Validate links
        if let Some(links) = &self.links {
            if links.len() > MAX_LINKS {
                return Err("Too many links".to_string());
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
        let title = self
            .title
            .trim()
            .chars()
            .take(MAX_LINK_TITLE_LENGTH)
            .collect::<String>();

        let url = match Url::parse(self.url.trim()) {
            Ok(parsed_url) => {
                let sanitized_url = parsed_url.to_string();
                sanitized_url
                    .chars()
                    .take(MAX_LINK_URL_LENGTH)
                    .collect::<String>()
            }
            Err(_) => "".to_string(), // Default to empty string for invalid URLs
        };

        PubkyAppUserLink { title, url }
    }

    fn validate(&self, _id: Option<&str>) -> Result<(), String> {
        if self.title.chars().count() > MAX_LINK_TITLE_LENGTH {
            return Err("Validation Error: Link title exceeds maximum length".to_string());
        }

        if self.url.chars().count() > MAX_LINK_URL_LENGTH {
            return Err("Validation Error: Link URL exceeds maximum length".to_string());
        }

        match Url::parse(&self.url) {
            Ok(_) => Ok(()),
            Err(_) => Err("Validation Error: Invalid URL format".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Validatable;
    use crate::{user_path_builder, APP_PATH, PUBLIC_PATH};

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
        let path = user_path_builder();
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
    fn test_validate_valid() {
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
}
