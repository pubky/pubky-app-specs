use crate::{
    common::timestamp,
    traits::{HasPubkyIdPath, Validatable},
    APP_PATH,
};
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use crate::traits::ToJson;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Represents raw homeserver follow object with timestamp
///
/// On follow objects, the main data is encoded in the path
///
/// URI: /pub/pubky.app/follows/:user_id
///
/// Example URI:
///
/// `/pub/pubky.app/follows/pxnu33x7jtpx9ar1ytsi4yxbp6a5o36gwhffs8zoxmbuptici1jy`
///
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppFollow {
    pub created_at: i64,
}

// #[cfg(target_arch = "wasm32")]
// impl ToJson for PubkyAppFollow {}

impl PubkyAppFollow {
    /// Creates a new `PubkyAppFollow` instance.
    pub fn new() -> Self {
        let created_at = timestamp();
        Self { created_at }
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppFollow {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = toJson))]
    pub fn json(&self) -> Result<JsValue, JsValue> {
        self.to_json()
    }
}

#[cfg(target_arch = "wasm32")]
impl ToJson for PubkyAppFollow {}

impl Validatable for PubkyAppFollow {
    fn validate(&self, _id: &str) -> Result<(), String> {
        // TODO: additional follow validation? E.g., validate `created_at`?
        Ok(())
    }
}

impl HasPubkyIdPath for PubkyAppFollow {
    fn create_path(&self, pubky_id: &str) -> String {
        format!("{}follows/{}", APP_PATH, pubky_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Validatable;

    #[test]
    fn test_new() {
        let follow = PubkyAppFollow::new();
        // Check that created_at is recent
        let now = timestamp();
        // within 1 second
        assert!(follow.created_at <= now && follow.created_at >= now - 1_000_000);
    }

    #[test]
    fn test_create_path_with_id() {
        let mute = PubkyAppFollow::new();
        let path = mute.create_path("user_id123");
        assert_eq!(path, "/pub/pubky.app/follows/user_id123");
    }

    #[test]
    fn test_validate() {
        let follow = PubkyAppFollow::new();
        let result = follow.validate("some_user_id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_try_from_valid() {
        let follow_json = r#"
        {
            "created_at": 1627849723
        }
        "#;

        let blob = follow_json.as_bytes();
        let follow_parsed =
            <PubkyAppFollow as Validatable>::try_from(blob, "some_user_id").unwrap();

        assert_eq!(follow_parsed.created_at, 1627849723);
    }
}
