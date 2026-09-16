use crate::constants::social_path;
use crate::traits::{Root, ValidationCtx, ValidationError};
use crate::{
    common::{check_extra_keys, timestamp, validate_safe_json_int},
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

/// Represents raw homeserver follow object with timestamp
///
/// On follow objects, the main data is encoded in the path
///
/// URI: /pub/social/v1/follows/:user_id.json
///
/// Example URI:
///
/// `/pub/social/v1/follows/pxnu33x7jtpx9ar1ytsi4yxbp6a5o36gwhffs8zoxmbuptici1jy`
///
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkySocialFollow {
    pub created_at: i64,
    /// Unknown members, preserved on rewrite; see the module contract in `models/mod.rs`.
    #[serde(flatten)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl PubkySocialFollow {
    /// Creates a new `PubkySocialFollow` instance.
    pub fn new() -> Self {
        Self {
            created_at: timestamp(),
            extra: Default::default(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkySocialFollow {
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
impl Json for PubkySocialFollow {}

impl Validatable for PubkySocialFollow {
    fn validate_fields(
        &self,
        id: Option<&str>,
        _ctx: &ValidationCtx,
    ) -> Result<(), ValidationError> {
        // Validate the followee ID
        if let Some(id) = id {
            PubkyId::try_from(id)?;
        }
        check_extra_keys(&self.extra, &["created_at"])?;
        validate_safe_json_int(self.created_at)?;
        Ok(())
    }
}

impl HasIdPath for PubkySocialFollow {
    const ROOT: Root = Root::Pub;
    const PATH_SEGMENT: &'static str = "follows/";

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
    use crate::traits::Validatable;
    use crate::traits::PUB_CTX;

    #[test]
    fn test_unknown_members_survive_and_created_at_is_safe() {
        const PK: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";
        let blob = br#"{"created_at":1727740800000000,"ext":{"list":"friends"}}"#;
        let follow = <PubkySocialFollow as Validatable>::try_from(blob, PK, &PUB_CTX).unwrap();
        assert_eq!(follow.extra["ext"]["list"], "friends");
        let back = serde_json::to_string(&follow).unwrap();
        assert!(back.contains(r#""ext":{"list":"friends"}"#), "{back}");
        let mut shadow = PubkySocialFollow::new();
        shadow.extra.insert("created_at".into(), 1.into());
        assert!(shadow
            .validate(Some(PK), &PUB_CTX)
            .unwrap_err()
            .contains("shadow"));
        let mut huge = PubkySocialFollow::new();
        huge.created_at = i64::MAX;
        assert!(huge
            .validate(Some(PK), &PUB_CTX)
            .unwrap_err()
            .contains("JSON-safe"));
    }

    #[test]
    fn test_new() {
        let follow = PubkySocialFollow::new();
        // Check that created_at is recent
        let now = timestamp();
        // within 1 second
        assert!(follow.created_at <= now && follow.created_at >= now - 1_000_000);
    }

    #[test]
    fn test_create_path_with_id() {
        let path = PubkySocialFollow::create_path("user_id123");
        assert_eq!(path, "/pub/social/v1/follows/user_id123.json");
    }

    #[test]
    fn test_validate() {
        let follow = PubkySocialFollow::new();
        let result = follow.validate(
            Some("operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo"),
            &PUB_CTX,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_id() {
        let follow = PubkySocialFollow::new();
        let result = follow.validate(Some("not_a_valid_pubky_id"), &PUB_CTX);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_valid() {
        let follow_json = r#"
        {
            "created_at": 1627849723
        }
        "#;

        let blob = follow_json.as_bytes();
        let follow_parsed = <PubkySocialFollow as Validatable>::try_from(
            blob,
            "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo",
            &PUB_CTX,
        )
        .unwrap();

        assert_eq!(follow_parsed.created_at, 1627849723);
    }
}
