use crate::{
    constants::MAX_SIZE,
    traits::{HasIdPath, HashId, Validatable},
    APP_PATH, PUBLIC_PATH,
};
use base32::{encode, Alphabet};
use blake3::Hasher;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use crate::traits::Json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Represents a blob, which backs a file uploaded by the user.
/// URI: /pub/pubky.app/blobs/:blob_id
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppBlob(#[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))] pub Vec<u8>);

impl PubkyAppBlob {
    /// Creates a new `PubkyAppBlob` instance.
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppBlob {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = fromJson))]
    pub fn from_json(js_value: &JsValue) -> Result<Self, String> {
        Self::import_json(js_value)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = toJson))]
    pub fn to_json(&self) -> Result<JsValue, String> {
        self.export_json()
    }

    /// Getter for the blob data as a `Uint8Array`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn data(&self) -> js_sys::Uint8Array {
        js_sys::Uint8Array::from(&self.0[..])
    }
}

#[cfg(target_arch = "wasm32")]
impl Json for PubkyAppBlob {}

impl HashId for PubkyAppBlob {
    fn get_id_data(&self) -> String {
        // data string id hashing is not needed for PubkyAppBlob as we hash the entire blob
        "".to_string()
    }

    fn create_id(&self) -> String {
        // Create a Blake3 hash of the blob data
        let mut hasher = Hasher::new();
        hasher.update(&self.0);
        let blake3_hash = hasher.finalize();

        // Get the first half of the hash bytes
        let half_hash_length = blake3_hash.as_bytes().len() / 2;
        let half_hash = &blake3_hash.as_bytes()[..half_hash_length];

        // Encode the first half of the hash in Base32 using the Z-base32 alphabet
        encode(Alphabet::Crockford, half_hash)
    }
}

impl HasIdPath for PubkyAppBlob {
    const PATH_SEGMENT: &'static str = "blobs/";

    fn create_path(id: &str) -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT, id].concat()
    }
}

impl Validatable for PubkyAppBlob {
    fn try_from(blob: &[u8], id: &str) -> Result<Self, String> {
        let instance = Self(blob.to_vec());
        instance.validate(Some(id))?;
        Ok(instance)
    }

    fn validate(&self, id: Option<&str>) -> Result<(), String> {
        // Check if the blob data is empty or exceeds maximum size
        if self.0.is_empty() {
            return Err("Validation Error: Blob size cannot be zero".to_string());
        }
        if self.0.len() > MAX_SIZE {
            return Err("Validation Error: Blob size exceeds maximum limit of 100MB".to_string());
        }

        // Validate the blob ID
        if let Some(id) = id {
            self.validate_id(id)?;
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{HashId, Validatable};

    #[test]
    fn test_create_id() {
        let blob = PubkyAppBlob(vec![1, 2]);
        let id = blob.create_id();
        assert_eq!(id, "PZBQ010FF079VVZPQG1RNFN6DR");

        // Test that same data produces same ID
        let blob2 = PubkyAppBlob(vec![1, 2]);
        assert_eq!(blob2.create_id(), id);

        // Test that different data produces different ID
        let blob3 = PubkyAppBlob(vec![1, 2, 3]);
        assert_ne!(blob3.create_id(), id);
    }

    #[test]
    fn test_validate() {
        let blob = PubkyAppBlob(vec![1, 2, 3]);
        let id = blob.create_id();
        let result = blob.validate(Some(&id));
        assert!(result.is_ok());

        // Test without ID
        let result = blob.validate(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_size_errors() {
        // Test blob at max size (should pass)
        let max_size_blob = PubkyAppBlob(vec![0; MAX_SIZE]);
        let id = max_size_blob.create_id();
        let result = max_size_blob.validate(Some(&id));
        assert!(result.is_ok(), "Blob at max size should be valid");

        // Test zero-size blob (should fail)
        let zero_size_blob = PubkyAppBlob(vec![]);
        let id = zero_size_blob.create_id();
        let result = zero_size_blob.validate(Some(&id));
        assert!(result.is_err(), "Zero-size blob should be invalid");
        assert!(result.unwrap_err().contains("cannot be zero"));

        // Test blob exceeding max size (should fail)
        let oversized_blob = PubkyAppBlob(vec![0; MAX_SIZE + 1]);
        let id = oversized_blob.create_id();
        let result = oversized_blob.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds maximum limit"));
    }

    #[test]
    fn test_validate_invalid_id() {
        let blob = PubkyAppBlob(vec![1, 2, 3]);
        let invalid_id = "INVALIDID";
        let result = blob.validate(Some(invalid_id));
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_valid() {
        let blob_data = vec![1, 2, 3, 4, 5];
        let blob = PubkyAppBlob(blob_data.clone());
        let id = blob.create_id();

        let result = <PubkyAppBlob as Validatable>::try_from(&blob_data, &id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, blob_data);
    }

    #[test]
    fn test_try_from_invalid_id() {
        let blob_data = vec![1, 2, 3];
        let invalid_id = "INVALIDID";

        let result = <PubkyAppBlob as Validatable>::try_from(&blob_data, invalid_id);
        assert!(result.is_err());
    }
}
