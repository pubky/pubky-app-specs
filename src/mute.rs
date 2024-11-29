use crate::{
    common::timestamp,
    traits::{HasPubkyIdPath, Validatable},
    APP_PATH,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Represents raw homeserver Mute object with timestamp
/// URI: /pub/pubky.app/mutes/:user_id
///
/// Example URI:
///
/// `/pub/pubky.app/mutes/pxnu33x7jtpx9ar1ytsi4yxbp6a5o36gwhffs8zoxmbuptici1jy`
///
#[derive(Serialize, Deserialize, Default, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
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
    fn validate(&self, _id: &str) -> Result<(), String> {
        // TODO: additional Mute validation? E.g., validate `created_at` ?
        Ok(())
    }
}

impl HasPubkyIdPath for PubkyAppMute {
    fn create_path(&self, pubky_id: &str) -> String {
        format!("{}mutes/{}", APP_PATH, pubky_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::timestamp;
    use crate::traits::Validatable;

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
        let mute = PubkyAppMute::new();
        let path = mute.create_path("user_id123");
        assert_eq!(path, "/pub/pubky.app/mutes/user_id123");
    }

    #[test]
    fn test_validate() {
        let mute = PubkyAppMute::new();
        let result = mute.validate("some_user_id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_try_from_valid() {
        let mute_json = r#"
        {
            "created_at": 1627849723
        }
        "#;

        let blob = mute_json.as_bytes();
        let mute_parsed = <PubkyAppMute as Validatable>::try_from(&blob, "some_user_id").unwrap();

        assert_eq!(mute_parsed.created_at, 1627849723);
    }
}
