use pkarr::PublicKey;
use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Represents user data with name, bio, image, links, and status.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyId(String);

impl PubkyId {
    pub fn try_from(str: &str) -> Result<Self, String> {
        // Validate string is a valid Pkarr public key
        match PublicKey::try_from(str) {
            Ok(_) => Ok(PubkyId(str.to_string())),
            Err(e) => Err(format!("Validation Error: Not a valid pubky id: {}", e)),
        }
    }
}

impl fmt::Display for PubkyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for PubkyId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for PubkyId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
