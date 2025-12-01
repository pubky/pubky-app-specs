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

#[cfg(not(target_arch = "wasm32"))]
impl From<pubky::PublicKey> for PubkyId {
    fn from(pk: pubky::PublicKey) -> Self {
        Self(pk.to_z32())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<pubky::Keypair> for PubkyId {
    fn from(keypair: pubky::Keypair) -> Self {
        Self::from(keypair.public_key())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    use pubky::Keypair;

    #[test]
    fn test_try_from_valid() {
        let valid_key = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";
        let result = PubkyId::try_from(valid_key);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_ref(), valid_key);
    }

    #[test]
    fn test_try_from_invalid_length() {
        let invalid_key = "short";
        let result = PubkyId::try_from(invalid_key);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Validation Error: the string is not 52 utf chars"
        );
    }

    #[test]
    fn test_try_from_invalid_encoding() {
        // 52 characters but invalid z-base-32 (contains invalid char '0')
        let invalid_key = "0perrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rd0";
        let result = PubkyId::try_from(invalid_key);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Validation Error: invalid public key encoding"
        );
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_from_public_key() {
        // Create a keypair and extract the public key
        let keypair = Keypair::random();
        let public_key = keypair.public_key();
        let expected_z32 = public_key.to_z32();

        // Convert PublicKey to PubkyId
        let pubky_id = PubkyId::from(public_key);

        // Verify the PubkyId contains the correct z32 string
        assert_eq!(pubky_id.as_ref(), expected_z32);
        assert_eq!(pubky_id.to_string(), expected_z32);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_from_keypair() {
        // Create a keypair
        let keypair = Keypair::random();
        let expected_z32 = keypair.public_key().to_z32();

        // Convert Keypair to PubkyId
        let pubky_id = PubkyId::from(keypair);

        // Verify the PubkyId contains the correct z32 string (derived from public key)
        assert_eq!(pubky_id.as_ref(), expected_z32);
        assert_eq!(pubky_id.to_string(), expected_z32);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_from_public_key_produces_valid_pubky_id() {
        // Ensure the PubkyId created from PublicKey is valid (52 char z32)
        let keypair = Keypair::random();
        let public_key = keypair.public_key();
        let pubky_id = PubkyId::from(public_key);

        // The inner string should be 52 characters (valid z32 public key)
        assert_eq!(pubky_id.as_ref().len(), 52);

        // Should also work with try_from validation
        let result = PubkyId::try_from(pubky_id.as_ref());
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_from_keypair_produces_valid_pubky_id() {
        // Ensure the PubkyId created from Keypair is valid (52 char z32)
        let keypair = Keypair::random();
        let pubky_id = PubkyId::from(keypair);

        // The inner string should be 52 characters (valid z32 public key)
        assert_eq!(pubky_id.as_ref().len(), 52);

        // Should also work with try_from validation
        let result = PubkyId::try_from(pubky_id.as_ref());
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_from_keypair_and_public_key_produce_same_result() {
        // Create a keypair
        let keypair = Keypair::random();
        let public_key = keypair.public_key();

        // Store expected value before moving keypair
        let expected_z32 = public_key.to_z32();

        // Convert both to PubkyId
        let pubky_id_from_keypair = PubkyId::from(keypair);
        let pubky_id_from_public_key = PubkyId::from(public_key);

        // Both should produce the same PubkyId
        assert_eq!(pubky_id_from_keypair, pubky_id_from_public_key);
        assert_eq!(pubky_id_from_keypair.as_ref(), expected_z32);
    }
}
