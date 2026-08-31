use crate::constants::social_path;
use crate::traits::{Root, ValidationCtx, ValidationError};
use crate::{
    common::timestamp,
    traits::{HasIdPath, Validatable},
    PubkyId,
};
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use crate::traits::Json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Represents raw homeserver Mute object with timestamp
/// URI: /pub/pubky.app/mutes/:user_id
///
/// Example URI:
///
/// `/pub/pubky.app/mutes/pxnu33x7jtpx9ar1ytsi4yxbp6a5o36gwhffs8zoxmbuptici1jy`
///
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkySocialMute {
    pub created_at: i64,
}

impl PubkySocialMute {
    /// Creates a new `PubkySocialMute` instance.
    pub fn new() -> Self {
        let created_at = timestamp();
        Self { created_at }
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkySocialMute {
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
impl Json for PubkySocialMute {}

impl Validatable for PubkySocialMute {
    fn validate(&self, id: Option<&str>, _ctx: &ValidationCtx) -> Result<(), ValidationError> {
        // Validate the muteee ID
        if let Some(id) = id {
            PubkyId::try_from(id)?;
        }
        // TODO: additional Mute validation? E.g., validate `created_at` ?
        Ok(())
    }
}

impl HasIdPath for PubkySocialMute {
    const ROOT: Root = Root::Pub;
    const PATH_SEGMENT: &'static str = "mutes/";

    fn create_path(pubky_id: &str) -> String {
        social_path(
            Self::ROOT,
            &format!("{}{pubky_id}.json", Self::PATH_SEGMENT),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::timestamp;
    use crate::traits::Validatable;
    use crate::traits::PUB_CTX;

    #[test]
    fn test_new() {
        let mute = PubkySocialMute::new();
        // Check that created_at is recent
        let now = timestamp();
        assert!(mute.created_at <= now && mute.created_at >= now - 1_000_000);
        // within 1 second
    }

    #[test]
    fn test_create_path_with_id() {
        let path =
            PubkySocialMute::create_path("operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo");
        assert_eq!(
            path,
            "/pub/social/v1/mutes/operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo.json"
        );
    }

    #[test]
    fn test_validate() {
        let mute = PubkySocialMute::new();
        let result = mute.validate(
            Some("operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo"),
            &PUB_CTX,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_id() {
        let mute = PubkySocialMute::new();
        let result = mute.validate(Some("not_a_valid_pubky_id"), &PUB_CTX);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_valid() {
        let mute_json = r#"
        {
            "created_at": 1627849723
        }
        "#;

        let blob = mute_json.as_bytes();
        let mute_parsed = <PubkySocialMute as Validatable>::try_from(
            blob,
            "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo",
            &PUB_CTX,
        )
        .unwrap();

        assert_eq!(mute_parsed.created_at, 1627849723);
    }
}
