use crate::{
    common::timestamp,
    traits::{HasPath, TimestampId, Validatable},
    APP_PATH,
};
use serde::{Deserialize, Serialize};

/// Represents a file uploaded by the user.
/// URI: /pub/pubky.app/files/:file_id
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct PubkyAppFile {
    name: String,
    created_at: i64,
    src: String,
    content_type: String,
    size: i64,
}

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
    }
}

impl TimestampId for PubkyAppFile {}

impl HasPath for PubkyAppFile {
    fn create_path(&self) -> String {
        format!("{}files/{}", APP_PATH, self.create_id())
    }
}

impl Validatable for PubkyAppFile {
    // TODO: content_type validation.
    fn validate(&self, id: &str) -> Result<(), String> {
        self.validate_id(id)?;
        // TODO: content_type validation.
        // TODO: size and other validation.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Validatable;
    use bytes::Bytes;

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
        let prefix = format!("{}files/", APP_PATH);
        assert!(path.starts_with(&prefix));

        let expected_path_len = prefix.len() + file_id.len();
        assert_eq!(path.len(), expected_path_len);
    }

    #[test]
    fn test_validate_valid() {
        let file = PubkyAppFile::new(
            "example.png".to_string(),
            "/uploads/example.png".to_string(),
            "image/png".to_string(),
            1024,
        );
        let id = file.create_id();
        let result = file.validate(&id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_id() {
        let file = PubkyAppFile::new(
            "example.png".to_string(),
            "/uploads/example.png".to_string(),
            "image/png".to_string(),
            1024,
        );
        let invalid_id = "INVALIDID";
        let result = file.validate(&invalid_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_valid() {
        let file_json = r#"
        {
            "name": "example.png",
            "created_at": 1627849723,
            "src": "/uploads/example.png",
            "content_type": "image/png",
            "size": 1024
        }
        "#;

        let file = PubkyAppFile::new(
            "example.png".to_string(),
            "/uploads/example.png".to_string(),
            "image/png".to_string(),
            1024,
        );
        let id = file.create_id();

        let blob = Bytes::from(file_json);
        let file_parsed = <PubkyAppFile as Validatable>::try_from(&blob, &id).unwrap();

        assert_eq!(file_parsed.name, "example.png");
        assert_eq!(file_parsed.src, "/uploads/example.png");
        assert_eq!(file_parsed.content_type, "image/png");
        assert_eq!(file_parsed.size, 1024);
    }
}
