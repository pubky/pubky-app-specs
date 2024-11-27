use crate::traits::Validatable;
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

// Validation
const MIN_USERNAME_LENGTH: usize = 3;
const MAX_USERNAME_LENGTH: usize = 50;
const MAX_BIO_LENGTH: usize = 160;
const MAX_IMAGE_LENGTH: usize = 300;
const MAX_LINKS: usize = 5;
const MAX_LINK_TITLE_LENGTH: usize = 100;
const MAX_LINK_URL_LENGTH: usize = 300;
const MAX_STATUS_LENGTH: usize = 50;

/// Profile schema
/// URI: /pub/pubky.app/profile.json
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct PubkyAppUser {
    name: String,
    bio: Option<String>,
    image: Option<String>,
    links: Option<Vec<PubkyAppUserLink>>,
    status: Option<String>,
}

/// Represents a user's single link with a title and URL.
#[derive(Serialize, Deserialize, ToSchema, Default, Clone, Debug)]
pub struct PubkyAppUserLink {
    title: String,
    url: String,
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

    fn validate(&self, _id: &str) -> Result<(), String> {
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
                link.validate(_id)?;
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

    fn validate(&self, _id: &str) -> Result<(), String> {
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
