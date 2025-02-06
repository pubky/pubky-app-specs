use crate::{
    common::timestamp,
    traits::{HasPath, Validatable},
    APP_PATH, PUBLIC_PATH,
};
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use crate::traits::Json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Represents the last read timestamp for notifications.
/// URI: /pub/pubky.app/last_read
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppLastRead {
    pub timestamp: i64, // Unix epoch time in milliseconds
}

impl PubkyAppLastRead {
    /// Creates a new `PubkyAppLastRead` instance.
    pub fn new() -> Self {
        let timestamp = timestamp() / 1_000; // Convert to milliseconds
        Self { timestamp }
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppLastRead {
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
impl Json for PubkyAppLastRead {}

impl Validatable for PubkyAppLastRead {
    fn validate(&self, _id: Option<&str>) -> Result<(), String> {
        // Validate timestamp is a positive integer
        if self.timestamp <= 0 {
            return Err("Validation Error: Timestamp must be a positive integer".into());
        }
        Ok(())
    }
}

impl HasPath for PubkyAppLastRead {
    const PATH_SEGMENT: &'static str = "last_read";

    fn create_path(&self) -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT].concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Validatable;

    #[test]
    fn test_new() {
        let last_read = PubkyAppLastRead::new();
        let now = timestamp() / 1_000;
        // within 1 second
        assert!(last_read.timestamp <= now && last_read.timestamp >= now - 1_000);
    }

    #[test]
    fn test_create_path() {
        let last_read = PubkyAppLastRead::new();
        let path = last_read.create_path();
        assert_eq!(path, format!("{}{}last_read", PUBLIC_PATH, APP_PATH));
    }

    #[test]
    fn test_validate() {
        let last_read = PubkyAppLastRead::new();
        let result = last_read.validate(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_timestamp() {
        let last_read = PubkyAppLastRead { timestamp: -1 };
        let result = last_read.validate(None);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_valid() {
        let last_read_json = r#"
        {
            "timestamp": 1700000000
        }
        "#;

        let blob = last_read_json.as_bytes();
        let last_read = <PubkyAppLastRead as Validatable>::try_from(blob, "").unwrap();
        assert_eq!(last_read.timestamp, 1700000000);
    }
}
