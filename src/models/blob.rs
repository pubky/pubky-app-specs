use crate::{
    traits::{HasPath, HashId, Validatable},
    APP_PATH, PUBLIC_PATH,
};
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use crate::traits::ToJson;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

const SAMPLE_SIZE: usize = 2 * 1024;

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
    /// Serialize to JSON for WASM.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = toJson))]
    pub fn json(&self) -> Result<JsValue, JsValue> {
        self.to_json()
    }

    /// Getter for the blob data as a `Uint8Array`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn data(&self) -> js_sys::Uint8Array {
        js_sys::Uint8Array::from(&self.0[..])
    }
}

#[cfg(target_arch = "wasm32")]
impl ToJson for PubkyAppBlob {}

impl HashId for PubkyAppBlob {
    fn get_id_data(&self) -> String {
        // Get the start and end samples
        let start = &self.0[..SAMPLE_SIZE.min(self.0.len())];
        let end = if self.0.len() > SAMPLE_SIZE {
            &self.0[self.0.len() - SAMPLE_SIZE..]
        } else {
            &[]
        };

        // Combine the samples
        let mut combined = Vec::with_capacity(start.len() + end.len());
        combined.extend_from_slice(start);
        combined.extend_from_slice(end);

        base32::encode(base32::Alphabet::Crockford, &combined)
    }
}

impl HasPath for PubkyAppBlob {
    const PATH_SEGMENT: &'static str = "blobs/";

    fn create_path(&self) -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT, &self.create_id()].concat()
    }
}

impl Validatable for PubkyAppBlob {
    fn validate(&self, id: &str) -> Result<(), String> {
        // Validate the tag ID
        self.validate_id(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::HashId;

    #[test]
    fn test_get_id_data_size_is_smaller_than_sample() {
        let blob = PubkyAppBlob(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let id = blob.get_id_data();
        assert_eq!(id, "041061050R3GG28A");
    }
}
