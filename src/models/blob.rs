use crate::{
    traits::{HasPath, HashId, Validatable},
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

/// Represents a file uploaded by the user.
/// URI: /pub/pubky.app/files/:file_id
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

impl HasPath for PubkyAppBlob {
    const PATH_SEGMENT: &'static str = "blobs/";

    fn create_path(&self) -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT, &self.create_id()].concat()
    }
}

impl Validatable for PubkyAppBlob {
    fn try_from(blob: &[u8], id: &str) -> Result<Self, String> {
        let instance = Self(blob.to_vec());
        instance.validate(Some(id))?;
        Ok(instance)
    }

    fn validate(&self, id: Option<&str>) -> Result<(), String> {
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
    use crate::traits::HashId;

    #[test]
    fn test_create_id_from_small_blob() {
        let blob = PubkyAppBlob(vec![1, 2]);
        let id = blob.create_id();
        assert_eq!(id, "PZBQ010FF079VVZPQG1RNFN6DR");
    }
}
