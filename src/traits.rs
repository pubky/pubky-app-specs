use crate::common::timestamp;
use base32::{decode, encode, Alphabet};
use blake3::Hasher;
use serde::de::DeserializeOwned;

#[cfg(target_arch = "wasm32")]
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub trait TimestampId {
    /// Creates a unique identifier based on the current timestamp.
    fn create_id(&self) -> String {
        // Get current time in microseconds since UNIX epoch
        let now = timestamp();

        // Convert to big-endian bytes
        let bytes = now.to_be_bytes();

        // Encode the bytes using Base32 with the Crockford alphabet
        encode(Alphabet::Crockford, &bytes)
    }

    /// Validates that the provided ID is a valid Crockford Base32-encoded timestamp,
    /// 13 characters long, and represents a reasonable timestamp.
    fn validate_id(&self, id: &str) -> Result<(), String> {
        // Ensure ID is 13 characters long
        if id.len() != 13 {
            return Err("Validation Error: Invalid ID length: must be 13 characters".into());
        }

        // Decode the Crockford Base32-encoded ID
        let decoded_bytes =
            decode(Alphabet::Crockford, id).ok_or("Failed to decode Crockford Base32 ID")?;

        if decoded_bytes.len() != 8 {
            return Err("Validation Error: Invalid ID length after decoding".into());
        }

        // Convert the decoded bytes to a timestamp in microseconds
        let timestamp_micros = i64::from_be_bytes(decoded_bytes.try_into().unwrap());

        // Get current time in microseconds
        let now_micros = timestamp();

        // Define October 1st, 2024, in microseconds since UNIX epoch
        let oct_first_2024_micros = 1727740800000000; // Timestamp for 2024-10-01 00:00:00 UTC

        // Allowable future duration (2 hours) in microseconds
        let max_future_micros = now_micros + 2 * 60 * 60 * 1_000_000;

        // Validate that the ID's timestamp is after October 1st, 2024
        if timestamp_micros < oct_first_2024_micros {
            return Err(
                "Validation Error: Invalid ID, timestamp must be after October 1st, 2024".into(),
            );
        }

        // Validate that the ID's timestamp is not more than 2 hours in the future
        if timestamp_micros > max_future_micros {
            return Err("Validation Error: Invalid ID, timestamp is too far in the future".into());
        }

        Ok(())
    }
}

/// Trait for generating an ID based on the struct's data.
pub trait HashId {
    fn get_id_data(&self) -> String;

    /// Creates a unique identifier for bookmarks and tag homeserver paths instance.
    ///
    /// The ID is generated by:
    /// 1. Concatenating the `uri` and `label` fields of the `PubkyAppTag` with a colon (`:`) separator.
    /// 2. Hashing the concatenated string using the `blake3` hashing algorithm.
    /// 3. Taking the first half of the bytes from the resulting `blake3` hash.
    /// 4. Encoding those bytes using the Crockford alphabet (Base32 variant).
    ///
    /// The resulting Crockford-encoded string is returned as the tag ID.
    ///
    /// # Returns
    /// - A `String` representing the Crockford-encoded tag ID derived from the `blake3` hash of the concatenated `uri` and `label`.
    fn create_id(&self) -> String {
        let data = self.get_id_data();

        // Create a Blake3 hash of the input data
        let mut hasher = Hasher::new();
        hasher.update(data.as_bytes());
        let blake3_hash = hasher.finalize();

        // Get the first half of the hash bytes
        let half_hash_length = blake3_hash.as_bytes().len() / 2;
        let half_hash = &blake3_hash.as_bytes()[..half_hash_length];

        // Encode the first half of the hash in Base32 using the Z-base32 alphabet
        encode(Alphabet::Crockford, half_hash)
    }

    /// Validates that the provided ID matches the generated ID.
    fn validate_id(&self, id: &str) -> Result<(), String> {
        let generated_id = self.create_id();
        if generated_id != id {
            return Err(format!("Invalid ID: expected {}, found {}", generated_id, id).into());
        }
        Ok(())
    }
}

pub trait Validatable: Sized + DeserializeOwned {
    fn try_from(blob: &[u8], id: &str) -> Result<Self, String> {
        let mut instance: Self = serde_json::from_slice(blob).map_err(|e| e.to_string())?;
        instance = instance.sanitize();
        instance.validate(id)?;
        Ok(instance)
    }

    fn validate(&self, id: &str) -> Result<(), String>;

    fn sanitize(self) -> Self {
        self
    }
}

pub trait HasPath {
    fn create_path(&self) -> String;
}

pub trait HasPubkyIdPath {
    fn create_path(&self, pubky_id: &str) -> String;
}

#[cfg(target_arch = "wasm32")]
pub trait JSdata: HasPath + Serialize {
    // helper that returns { id, path, json }
    fn get_data(&self) -> Result<JsValue, JsValue> {
        let path = self.create_path();

        let json_val = serde_wasm_bindgen::to_value(&self)
            .map_err(|e| JsValue::from_str(&format!("JSON Error: {}", e)))?;

        // Construct a small JS object
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("id"), &JsValue::null())?;
        js_sys::Reflect::set(&obj, &JsValue::from_str("path"), &JsValue::from_str(&path))?;
        js_sys::Reflect::set(&obj, &JsValue::from_str("json"), &json_val)?;

        Ok(obj.into())
    }
}
