use crate::models::legacy_v0::{
    common::timestamp,
    traits::{HasIdPath, Validatable},
    PubkyId, APP_PATH, PUBLIC_PATH,
};
use serde::{Deserialize, Serialize};

/// Represents raw homeserver Mute object with timestamp
/// URI: /pub/pubky.app/mutes/:user_id
///
/// Example URI:
///
/// `/pub/pubky.app/mutes/pxnu33x7jtpx9ar1ytsi4yxbp6a5o36gwhffs8zoxmbuptici1jy`
///
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct PubkyAppMute {
    pub created_at: i64,
}

impl PubkyAppMute {
    /// Creates a new `PubkyAppMute` instance.
    pub fn new() -> Self {
        let created_at = timestamp();
        Self { created_at }
    }
}

impl Validatable for PubkyAppMute {
    fn validate(&self, id: Option<&str>) -> Result<(), String> {
        // Validate the muteee ID
        if let Some(id) = id {
            PubkyId::try_from(id)?;
        }
        // TODO: additional Mute validation? E.g., validate `created_at` ?
        Ok(())
    }
}

impl HasIdPath for PubkyAppMute {
    const PATH_SEGMENT: &'static str = "mutes/";

    fn create_path(pubky_id: &str) -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT, pubky_id].concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::legacy_v0::common::timestamp;
    use crate::models::legacy_v0::traits::Validatable;

    #[test]
    fn test_new() {
        let mute = PubkyAppMute::new();
        // Check that created_at is recent
        let now = timestamp();
        assert!(mute.created_at <= now && mute.created_at >= now - 1_000_000);
        // within 1 second
    }

    #[test]
    fn test_create_path_with_id() {
        let path =
            PubkyAppMute::create_path("operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo");
        assert_eq!(
            path,
            "/pub/pubky.app/mutes/operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo"
        );
    }

    #[test]
    fn test_validate() {
        let mute = PubkyAppMute::new();
        let result = mute.validate(Some("operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_id() {
        let mute = PubkyAppMute::new();
        let result = mute.validate(Some("not_a_valid_pubky_id"));
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
        let mute_parsed = <PubkyAppMute as Validatable>::try_from(
            blob,
            "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo",
        )
        .unwrap();

        assert_eq!(mute_parsed.created_at, 1627849723);
    }
}
