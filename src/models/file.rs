use crate::{
    common::timestamp,
    traits::{HasPath, TimestampId, Validatable},
    APP_PATH, PUBLIC_PATH,
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

const MIN_NAME_LENGTH: usize = 1;
const MAX_NAME_LENGTH: usize = 255;
const MAX_SRC_LENGTH: usize = 1024;
const MAX_SIZE: i64 = 10 * (1 << 20); // 10 MB

const VALID_MIME_TYPES: &[&str] = &[
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
/// URI: /pub/pubky.app/files/:file_id
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppFile {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub name: String,
    pub created_at: i64,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub src: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub content_type: String,
    pub size: i64,
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppFile {
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
impl Json for PubkyAppFile {}

impl PubkyAppFile {
    /// Creates a new `PubkyAppFile` instance.
    pub fn new(name: String, src: String, content_type: String, size: i64) -> Self {
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

impl TimestampId for PubkyAppFile {}

impl HasPath for PubkyAppFile {
    const PATH_SEGMENT: &'static str = "files/";

    fn create_path(&self) -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT, &self.create_id()].concat()
    }
}

impl Validatable for PubkyAppFile {
    fn sanitize(self) -> Self {
        let name = self.name.trim().chars().take(MAX_NAME_LENGTH).collect();

        let sanitized_src = self
            .src
            .trim()
            .chars()
            .take(MAX_SRC_LENGTH)
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

    fn validate(&self, id: Option<&str>) -> Result<(), String> {
        // Validate the file ID
        if let Some(id) = id {
            self.validate_id(id)?;
        }

        // Validate name
        let name_length = self.name.chars().count();

        if !(MIN_NAME_LENGTH..=MAX_NAME_LENGTH).contains(&name_length) {
            return Err("Validation Error: Invalid name length".into());
        }

        // Validate src
        if self.src.chars().count() == 0 {
            return Err("Validation Error: Invalid src".into());
        }
        if self.src.chars().count() > MAX_SRC_LENGTH {
            return Err("Validation Error: src exceeds maximum length".into());
        }

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

        // Validate size
        if self.size <= 0 || self.size > MAX_SIZE {
            return Err("Validation Error: Invalid size".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Validatable;

    #[test]
    fn test_new() {
        let file = PubkyAppFile::new(
            "example.png".to_string(),
            "pubky://user_id/pub/pubky.app/blobs/id".to_string(),
            "image/png".to_string(),
            1024,
        );
        assert_eq!(file.name, "example.png");
        assert_eq!(file.src, "pubky://user_id/pub/pubky.app/blobs/id");
        assert_eq!(file.content_type, "image/png");
        assert_eq!(file.size, 1024);
        // Check that created_at is recent
        let now = timestamp();
        assert!(file.created_at <= now && file.created_at >= now - 1_000_000); // within 1 second
    }

    #[test]
    fn test_create_path() {
        let file = PubkyAppFile::new(
            "example.png".to_string(),
            "pubky://user_id/pub/pubky.app/blobs/id".to_string(),
            "image/png".to_string(),
            1024,
        );
        let file_id = file.create_id();
        let path = file.create_path();

        // Check if the path starts with the expected prefix
        let prefix = format!("{}{}files/", PUBLIC_PATH, APP_PATH);
        assert!(path.starts_with(&prefix));

        let expected_path_len = prefix.len() + file_id.len();
        assert_eq!(path.len(), expected_path_len);
    }

    #[test]
    fn test_validate_valid() {
        let file = PubkyAppFile::new(
            "example.png".to_string(),
            "pubky://user_id/pub/pubky.app/blobs/id".to_string(),
            "image/png".to_string(),
            1024,
        );
        let id = file.create_id();
        let result = file.validate(Some(&id));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_id() {
        let file = PubkyAppFile::new(
            "example.png".to_string(),
            "pubky://user_id/pub/pubky.app/blobs/id".to_string(),
            "image/png".to_string(),
            1024,
        );
        let invalid_id = "INVALIDID";
        let result = file.validate(Some(invalid_id));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_content_type() {
        let file = PubkyAppFile::new(
            "example.png".to_string(),
            "pubky://user_id/pub/pubky.app/blobs/id".to_string(),
            "notavalid/content_type".to_string(),
            1024,
        );
        let id = file.create_id();
        let result = file.validate(Some(&id));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_size() {
        let file = PubkyAppFile::new(
            "example.png".to_string(),
            "pubky://user_id/pub/pubky.app/blobs/id".to_string(),
            "notavalid/content_type".to_string(),
            MAX_SIZE + 1,
        );
        let id = file.create_id();
        let result = file.validate(Some(&id));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_src() {
        let file = PubkyAppFile::new(
            "example.png".to_string(),
            "not_a_url".to_string(),
            "notavalid/content_type".to_string(),
            MAX_SIZE + 1,
        );
        let id = file.create_id();
        let result = file.validate(Some(&id));
        assert!(result.is_err());
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

        let file = PubkyAppFile::new(
            "example.png".to_string(),
            "pubky://user_id/pub/pubky.app/blobs/id".to_string(),
            "image/png".to_string(),
            1024,
        );
        let id = file.create_id();

        let blob = file_json.as_bytes();
        let file_parsed = <PubkyAppFile as Validatable>::try_from(blob, &id).unwrap();

        assert_eq!(file_parsed.name, "example.png");
        assert_eq!(file_parsed.src, "pubky://user_id/pub/pubky.app/blobs/id");
        assert_eq!(file_parsed.content_type, "image/png");
        assert_eq!(file_parsed.size, 1024);
    }
}
