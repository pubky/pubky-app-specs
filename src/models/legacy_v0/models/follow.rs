use crate::models::legacy_v0::{
    common::timestamp,
    traits::{HasIdPath, Validatable},
    PubkyId, APP_PATH, PUBLIC_PATH,
};
use serde::{Deserialize, Serialize};

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
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct PubkyAppFollow {
    pub created_at: i64,
}

// #[cfg(target_arch = "wasm32")]
// impl Json for PubkyAppFollow {}

impl PubkyAppFollow {
    /// Creates a new `PubkyAppFollow` instance.
    pub fn new() -> Self {
        let created_at = timestamp();
        Self { created_at }
    }
}

impl Validatable for PubkyAppFollow {
    fn validate(&self, id: Option<&str>) -> Result<(), String> {
        // Validate the followee ID
        if let Some(id) = id {
            PubkyId::try_from(id)?;
        }
        // TODO: additional follow validation? E.g., validate `created_at`?
        Ok(())
    }
}

impl HasIdPath for PubkyAppFollow {
    const PATH_SEGMENT: &'static str = "follows/";

    fn create_path(pubky_id: &str) -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT, pubky_id].concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::legacy_v0::traits::Validatable;

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
        let path = PubkyAppFollow::create_path("user_id123");
        assert_eq!(path, "/pub/pubky.app/follows/user_id123");
    }

    #[test]
    fn test_validate() {
        let follow = PubkyAppFollow::new();
        let result = follow.validate(Some("operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_id() {
        let follow = PubkyAppFollow::new();
        let result = follow.validate(Some("not_a_valid_pubky_id"));
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
        let follow_parsed = <PubkyAppFollow as Validatable>::try_from(
            blob,
            "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo",
        )
        .unwrap();

        assert_eq!(follow_parsed.created_at, 1627849723);
    }
}
