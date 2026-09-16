use crate::constants::social_path;
use crate::traits::{Root, ValidationCtx, ValidationError};
use crate::{
    common::{check_extra, timestamp, validate_safe_json_int},
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
/// URI: /priv/social/v1/mutes/:user_id.json
///
/// Example URI:
///
/// `/priv/social/v1/mutes/pxnu33x7jtpx9ar1ytsi4yxbp6a5o36gwhffs8zoxmbuptici1jy`
///
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkySocialMute {
    pub created_at: i64,
    /// Unknown members, preserved on rewrite; see the module contract in `models/mod.rs`.
    #[serde(flatten)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl PubkySocialMute {
    /// Creates a new `PubkySocialMute` instance.
    pub fn new() -> Self {
        Self {
            created_at: timestamp(),
            extra: Default::default(),
        }
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
    fn validate_fields(
        &self,
        id: Option<&str>,
        _ctx: &ValidationCtx,
    ) -> Result<(), ValidationError> {
        // Validate the muteee ID
        if let Some(id) = id {
            PubkyId::try_from(id)?;
        }
        check_extra(&self.extra, &["created_at"])?;
        validate_safe_json_int(self.created_at)?;
        Ok(())
    }
}

impl HasIdPath for PubkySocialMute {
    const ROOT: Root = Root::Priv;
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
    const PK: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";
    const PRIV_CTX: ValidationCtx = ValidationCtx { root: Root::Priv };
    use crate::common::timestamp;
    use crate::traits::Validatable;

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
            "/priv/social/v1/mutes/operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo.json"
        );
        // the builder emits the private root and the parser reads it back
        let uri = crate::mute_uri_builder(PK.into(), PK.into());
        let parsed = crate::ParsedUri::try_from(uri.as_str()).unwrap();
        assert_eq!(parsed.visibility, crate::Visibility::Private);
        assert!(matches!(parsed.resource, crate::Resource::Mute(_)));
        assert_eq!(parsed.try_to_uri_str().unwrap(), uri);
        // a public mute path is not a mute
        let public = uri.replace("/priv/", "/pub/");
        let parsed = crate::ParsedUri::try_from(public.as_str()).unwrap();
        assert!(matches!(parsed.resource, crate::Resource::Unknown));
    }

    #[test]
    fn test_unknown_members_survive_and_created_at_is_safe() {
        let blob = br#"{"created_at":1727740800000000,"ext":{"reason":"spam"}}"#;
        let mute = <PubkySocialMute as Validatable>::try_from(blob, PK, &PRIV_CTX).unwrap();
        assert_eq!(mute.extra["ext"]["reason"], "spam");
        let back = serde_json::to_string(&mute).unwrap();
        assert!(back.contains(r#""ext":{"reason":"spam"}"#), "{back}");
        let mut shadow = PubkySocialMute::new();
        shadow.extra.insert("created_at".into(), 1.into());
        assert!(shadow
            .validate(Some(PK), &PRIV_CTX)
            .unwrap_err()
            .contains("shadow"));
        let mut huge = PubkySocialMute::new();
        huge.created_at = i64::MAX;
        assert!(huge
            .validate(Some(PK), &PRIV_CTX)
            .unwrap_err()
            .contains("JSON-safe"));
        // ingest by URI under the private root; the public spelling is not a mute
        let uri = crate::mute_uri_builder(PK.into(), PK.into());
        assert!(crate::PubkySocialObject::from_uri(&uri, blob).is_ok());
        assert!(crate::PubkySocialObject::from_uri(uri.replace("/priv/", "/pub/"), blob).is_err());
    }

    #[test]
    fn test_validate() {
        let mute = PubkySocialMute::new();
        let result = mute.validate(
            Some("operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo"),
            &PRIV_CTX,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_id() {
        let mute = PubkySocialMute::new();
        let result = mute.validate(Some("not_a_valid_pubky_id"), &PRIV_CTX);
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
            &PRIV_CTX,
        )
        .unwrap();

        assert_eq!(mute_parsed.created_at, 1627849723);
    }
}
