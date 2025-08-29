use base32::{decode, Alphabet};
use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use crate::{ParsedUri, Resource};

/// Represents user data with name, bio, image, links, and status.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyId(String);

impl PubkyId {
    pub fn try_from(s: &str) -> Result<Self, String> {
        // Validate string is a valid Pkarr public key
        // Should closely resemble the behavior of pkarr::PublicKey::try_from(&str) for the case of 52 chars
        // https://github.com/pubky/pkarr/blob/72fe80c271c1c1d2293e6a6800f227c570e8d4f5/pkarr/src/keys.rs#L142-L214
        // We avoid pkarr as a dependency by doing writing our own validation instead.
        if s.len() != 52 {
            return Err("Validation Error: the string is not 52 utf chars".to_string());
        }

        match decode(Alphabet::Z, s) {
            Some(_) => (),
            None => return Err("Validation Error: invalid public key encoding".to_string()),
        };

        Ok(PubkyId(s.to_string()))
    }

    pub fn to_uri(&self) -> ParsedUri {
        ParsedUri {
            user_id: self.clone(),
            resource: Resource::User,
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
