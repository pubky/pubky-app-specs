use crate::constants::social_path;
use crate::traits::{Root, ValidationCtx, ValidationError};
use crate::{
    common::timestamp,
    limits::VALIDATION_LIMITS,
    traits::{HasIdPath, TimestampId, Validatable},
};
use mime::Mime;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use url::Url;

#[cfg(target_arch = "wasm32")]
use crate::traits::Json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

// Local to this model so the shared limits table carries only 1.0 rows.
const FILE_NAME_MIN_LENGTH: usize = 1;
const FILE_NAME_MAX_LENGTH: usize = 255;
const FILE_SRC_MAX_LENGTH: usize = 1024;

/// Valid MIME types for file attachments.
pub const VALID_MIME_TYPES: &[&str] = &[
    "application/javascript",
    "application/json",
    "application/octet-stream",
    "application/pdf",
    "application/x-www-form-urlencoded",
    "application/xml",
    "application/zip",
    "audio/mpeg",
    "audio/wav",
    "image/gif",
    "image/jpeg",
    "image/png",
    "image/svg+xml",
    "image/webp",
    "multipart/form-data",
    "text/css",
    "text/html",
    "text/plain",
    "text/xml",
    "video/mp4",
    "video/mpeg",
];

/// Represents a file uploaded by the user.
/// URI: /pub/social/v1/files/:file_id
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkySocialFile {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub name: String,
    pub created_at: i64,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub src: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub content_type: String,
    pub size: usize,
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkySocialFile {
    // Getters clone the data out because String/JsValue is not Copy.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn name(&self) -> String {
        self.name.clone()
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn src(&self) -> String {
        self.src.clone()
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn content_type(&self) -> String {
        self.content_type.clone()
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
impl Json for PubkySocialFile {}

impl PubkySocialFile {
    /// Creates a new `PubkySocialFile` instance.
    pub fn new(name: String, src: String, content_type: String, size: usize) -> Self {
        let created_at = timestamp();
        Self {
            name,
            created_at,
            src,
            content_type,
            size,
        }
        .sanitize()
    }
}

impl TimestampId for PubkySocialFile {}

impl HasIdPath for PubkySocialFile {
    const ROOT: Root = Root::Pub;
    const PATH_SEGMENT: &'static str = "files/";

    fn create_path(id: &str) -> String {
        social_path(Self::ROOT, &format!("{}{id}.json", Self::PATH_SEGMENT))
    }
}

impl Validatable for PubkySocialFile {
    fn sanitize(self) -> Self {
        let name = self.name.trim().to_string();

        let sanitized_src = self
            .src
            .trim()
            .chars()
            .take(FILE_SRC_MAX_LENGTH)
            .collect::<String>();

        let src = match Url::parse(&sanitized_src) {
            Ok(_) => Some(sanitized_src),
            Err(_) => None, // Invalid src URL, set to None
        };

        let content_type = self.content_type.trim().to_string();

        Self {
            name,
            created_at: self.created_at,
            src: src.unwrap_or("".to_string()),
            content_type,
            size: self.size,
        }
    }

    fn validate(&self, id: Option<&str>, _ctx: &ValidationCtx) -> Result<(), ValidationError> {
        // Validate the file ID
        if let Some(id) = id {
            self.validate_id(id)?;
        }

        // Validate size
        if self.size == 0 {
            return Err("Validation Error: File size cannot be zero".to_string());
        }
        if self.size > VALIDATION_LIMITS.max_file_size_bytes {
            return Err("Validation Error: File size exceeds maximum limit of 100MB".to_string());
        }

        // Validate name
        let name_length = self.name.chars().count();

        if !(FILE_NAME_MIN_LENGTH..=FILE_NAME_MAX_LENGTH).contains(&name_length) {
            return Err("Validation Error: Invalid name length".into());
        }

        // Validate src
        if self.src.chars().count() == 0 {
            return Err("Validation Error: Invalid src".into());
        }
        if self.src.chars().count() > FILE_SRC_MAX_LENGTH {
            return Err("Validation Error: src exceeds maximum length".into());
        }
        // Validate URL format
        Url::parse(&self.src)
            .map_err(|_| "Validation Error: Invalid src URI format".to_string())?;

        // validate content type
        match Mime::from_str(&self.content_type) {
            Ok(mime) => {
                if !VALID_MIME_TYPES.contains(&mime.essence_str()) {
                    return Err("Validation Error: Invalid content type".into());
                }
            }
            Err(_) => {
                return Err("Validation Error: Invalid content type".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::PUB_CTX;
    use crate::{blob_uri_builder, traits::Validatable};

    #[test]
    fn test_new() {
        let file = PubkySocialFile::new(
            "example.png".to_string(),
            blob_uri_builder("user_id".into(), "id".into()),
            "image/png".to_string(),
            1024,
        );
        assert_eq!(file.name, "example.png");
        assert_eq!(file.src, "pubky://user_id/pub/social/v1/blobs/id");
        assert_eq!(file.content_type, "image/png");
        assert_eq!(file.size, 1024);
        // Check that created_at is recent
        let now = timestamp();
        assert!(file.created_at <= now && file.created_at >= now - 1_000_000); // within 1 second
    }

    #[test]
    fn test_create_path() {
        let file = PubkySocialFile::new(
            "example.png".to_string(),
            blob_uri_builder("user_id".into(), "id".into()),
            "image/png".to_string(),
            1024,
        );
        let file_id = file.create_id();
        let path = PubkySocialFile::create_path(&file_id);

        // Check if the path starts with the expected prefix
        let prefix = "/pub/social/v1/files/".to_string();
        assert!(path.starts_with(&prefix));

        let expected_path_len = prefix.len() + file_id.len() + ".json".len();
        assert_eq!(path.len(), expected_path_len);
    }

    #[test]
    fn test_validate() {
        let file = PubkySocialFile::new(
            "example.png".to_string(),
            blob_uri_builder("user_id".into(), "id".into()),
            "image/png".to_string(),
            1024,
        );
        let id = file.create_id();
        let result = file.validate(Some(&id), &PUB_CTX);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_id() {
        let file = PubkySocialFile::new(
            "example.png".to_string(),
            blob_uri_builder("user_id".into(), "id".into()),
            "image/png".to_string(),
            1024,
        );
        let invalid_id = "INVALIDID";
        let result = file.validate(Some(invalid_id), &PUB_CTX);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_field_errors() {
        // Test multiple validation errors
        let test_cases = vec![
            // Invalid content type
            (
                PubkySocialFile::new(
                    "example.png".to_string(),
                    blob_uri_builder("user_id".into(), "id".into()),
                    "notavalid/content_type".to_string(),
                    1024,
                ),
                "Invalid content type",
            ),
            // Invalid size (too large)
            (
                PubkySocialFile::new(
                    "example.png".to_string(),
                    blob_uri_builder("user_id".into(), "id".into()),
                    "image/png".to_string(),
                    VALIDATION_LIMITS.max_file_size_bytes + 1,
                ),
                "exceeds maximum limit",
            ),
            // Invalid size (zero)
            (
                PubkySocialFile::new(
                    "example.png".to_string(),
                    blob_uri_builder("user_id".into(), "id".into()),
                    "image/png".to_string(),
                    0,
                ),
                "cannot be zero",
            ),
        ];

        for (file, expected_error) in test_cases {
            let id = file.create_id();
            let result = file.validate(Some(&id), &PUB_CTX);
            assert!(
                result.is_err(),
                "Should reject file with {}",
                expected_error
            );
            assert!(result.unwrap_err().contains(expected_error));
        }
    }

    #[test]
    fn test_validate_invalid_src() {
        // Create file directly without sanitization to test validation logic
        let file = PubkySocialFile {
            name: "example.png".to_string(),
            created_at: timestamp(),
            src: "not_a_url".to_string(), // Invalid URL - sanitization would filter this
            content_type: "image/png".to_string(),
            size: 1024,
        };
        let id = file.create_id();
        let result = file.validate(Some(&id), &PUB_CTX);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid src"));
    }

    #[test]
    fn test_try_from_valid() {
        let file_json = r#"
        {
            "name": "example.png",
            "created_at": 1627849723,
            "src": "pubky://user_id/pub/pubky.app/blobs/id",
            "content_type": "image/png",
            "size": 1024
        }
        "#;

        let file = PubkySocialFile::new(
            "example.png".to_string(),
            blob_uri_builder("user_id".into(), "id".into()),
            "image/png".to_string(),
            1024,
        );
        let id = file.create_id();

        let blob = file_json.as_bytes();
        let file_parsed = <PubkySocialFile as Validatable>::try_from(blob, &id, &PUB_CTX).unwrap();

        assert_eq!(file_parsed.name, "example.png");
        assert_eq!(file_parsed.src, "pubky://user_id/pub/pubky.app/blobs/id");
        assert_eq!(file_parsed.content_type, "image/png");
        assert_eq!(file_parsed.size, 1024);
    }
}
