use crate::traits::Validatable;
use serde::{Deserialize, Serialize};

/// Represents raw homeserver follow object with timestamp
/// URI: /pub/pubky.app/follows/:user_id
///
/// Example URI:
///
/// `/pub/pubky.app/follows/pxnu33x7jtpx9ar1ytsi4yxbp6a5o36gwhffs8zoxmbuptici1jy``
///
#[derive(Serialize, Deserialize, Default)]
pub struct PubkyAppFollow {
    created_at: i64,
}

impl Validatable for PubkyAppFollow {
    fn validate(&self, _id: &str) -> Result<(), String> {
        // TODO: additional follow validation? E.g, validate `created_at` ?
        Ok(())
    }
}
