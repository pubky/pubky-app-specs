use crate::traits::Validatable;
use serde::{Deserialize, Serialize};

/// Represents raw homeserver Mute object with timestamp
/// URI: /pub/pubky.app/mutes/:user_id
///
/// Example URI:
///
/// `/pub/pubky.app/mutes/pxnu33x7jtpx9ar1ytsi4yxbp6a5o36gwhffs8zoxmbuptici1jy``
///
#[derive(Serialize, Deserialize, Default)]
pub struct PubkyAppMute {
    created_at: i64,
}

impl Validatable for PubkyAppMute {
    fn validate(&self, _id: &str) -> Result<(), String> {
        // TODO: additional Mute validation? E.g, validate `created_at` ?
        Ok(())
    }
}
