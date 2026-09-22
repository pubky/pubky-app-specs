use crate::constants::social_path;
use crate::mime::mime_to_ext;
use crate::traits::{Root, ValidationError};
use crate::{limits::VALIDATION_LIMITS, traits::HashId};
use base32::{encode, Alphabet};
use blake3::Hasher;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Advisory client hint only; gates nothing. The upload pipeline maps ANY declared type via
/// mime_to_ext.
pub const VALID_MIME_TYPES: &[&str] = &[
    "application/javascript",
    "application/json",
    "application/octet-stream",
    "application/pdf",
    "application/x-www-form-urlencoded",
    "application/xml",
    "application/zip",
    "audio/mpeg",
    "audio/wav",
    "image/gif",
    "image/jpeg",
    "image/png",
    "image/svg+xml",
    "image/webp",
    "multipart/form-data",
    "text/css",
    "text/csv",
    "text/html",
    "text/plain",
    "text/xml",
    "video/mp4",
    "video/mpeg",
];

/// A media file: the raw bytes, written as raw bytes and never as JSON. The id is the hash of
/// those bytes, so identical uploads collapse to one object and the extension, which is
/// path-only, cannot fork identity.
/// URI: /{pub|priv}/social/v1/files/:hash.:ext
///
/// Not a `Validatable`: that trait is the JSON-resource contract (parse, size cap on the
/// serialized form) and a media object has no JSON form at all, so it carries no serde derives
/// and no schema. Reading one is `from_bytes`.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct PubkySocialFile(#[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))] pub Vec<u8>);

/// What an upload needs: the object, its id, and the path carrying the extension.
#[derive(Debug, Clone)]
pub struct CreatedFile {
    pub file: PubkySocialFile,
    pub id: String,
    pub path: String,
}

impl PubkySocialFile {
    pub const ROOT: Root = Root::Pub;
    pub const PATH_SEGMENT: &'static str = "files/";

    /// The leaf is the full `{hash}.{ext}` filename, never the id alone.
    pub fn create_path_in(root: Root, filename: &str) -> String {
        social_path(root, &[Self::PATH_SEGMENT, filename].concat())
    }

    pub fn create_path(filename: &str) -> String {
        Self::create_path_in(Self::ROOT, filename)
    }

    /// The declared type is consumed exactly once, here, and never stored.
    pub fn create_file(
        bytes: Vec<u8>,
        declared_type: &str,
        root: Root,
    ) -> Result<CreatedFile, String> {
        let file = Self(bytes);
        file.validate(None)?;
        let id = file.create_id();
        let path = Self::create_path_in(root, &format!("{id}.{}", mime_to_ext(declared_type)));
        Ok(CreatedFile { file, id, path })
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkySocialFile {
    /// Getter for the file bytes as a `Uint8Array`. Media is bytes, so there is no
    /// `toJson`/`fromJson` pair.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn data(&self) -> js_sys::Uint8Array {
        js_sys::Uint8Array::from(&self.0[..])
    }
}

impl HashId for PubkySocialFile {
    fn get_id_data(&self) -> String {
        // data string id hashing is not needed for PubkySocialFile as we hash the entire file
        "".to_string()
    }

    fn create_id(&self) -> String {
        // Create a Blake3 hash of the file bytes
        let mut hasher = Hasher::new();
        hasher.update(&self.0);
        let blake3_hash = hasher.finalize();

        // Get the first half of the hash bytes
        let half_hash_length = blake3_hash.as_bytes().len() / 2;
        let half_hash = &blake3_hash.as_bytes()[..half_hash_length];

        // Encode the first half of the hash in Base32 using the Crockford alphabet
        encode(Alphabet::Crockford, half_hash)
    }
}

impl PubkySocialFile {
    /// Reads a stored media object: the bytes as served, checked against the id in its path.
    pub fn from_bytes(bytes: &[u8], id: &str) -> Result<Self, String> {
        let file = Self(bytes.to_vec());
        file.validate(Some(id))?;
        Ok(file)
    }

    /// Non-empty, within the media cap, and the id (when given) is the hash of the bytes.
    pub fn validate(&self, id: Option<&str>) -> Result<(), ValidationError> {
        if self.0.is_empty() {
            return Err("Validation Error: File size cannot be zero".to_string());
        }
        if self.0.len() > VALIDATION_LIMITS.max_file_size_bytes {
            return Err("Validation Error: File size exceeds maximum limit of 100MB".to_string());
        }
        if let Some(id) = id {
            self.validate_id(id)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uri::file_uri_builder;

    /// blake3 over [1, 2], first 16 bytes, Crockford.
    const KAT: &str = "PZBQ010FF079VVZPQG1RNFN6DR";

    #[test]
    fn test_create_id() {
        let file = PubkySocialFile(vec![1, 2]);
        let id = file.create_id();
        assert_eq!(id, KAT);

        // Test that same data produces same ID
        let file2 = PubkySocialFile(vec![1, 2]);
        assert_eq!(file2.create_id(), id);

        // Test that different data produces different ID
        let file3 = PubkySocialFile(vec![1, 2, 3]);
        assert_ne!(file3.create_id(), id);
    }

    #[test]
    fn test_validate() {
        let file = PubkySocialFile(vec![1, 2, 3]);
        let id = file.create_id();
        assert!(file.validate(Some(&id)).is_ok());

        // Test without ID
        assert!(file.validate(None).is_ok());
    }

    #[test]
    fn test_validate_size_errors() {
        let max_size_file = PubkySocialFile(vec![0; VALIDATION_LIMITS.max_file_size_bytes]);
        let id = max_size_file.create_id();
        let result = max_size_file.validate(Some(&id));
        assert!(result.is_ok(), "a file at max size should be valid");

        let zero_size_file = PubkySocialFile(vec![]);
        let id = zero_size_file.create_id();
        let result = zero_size_file.validate(Some(&id));
        assert!(result.is_err(), "a zero-size file should be invalid");
        assert!(result.unwrap_err().contains("cannot be zero"));

        let oversized_file = PubkySocialFile(vec![0; VALIDATION_LIMITS.max_file_size_bytes + 1]);
        let id = oversized_file.create_id();
        let result = oversized_file.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds maximum limit"));
    }

    #[test]
    fn test_validate_invalid_id() {
        let file = PubkySocialFile(vec![1, 2, 3]);
        assert!(file.validate(Some("INVALIDID")).is_err());
    }

    #[test]
    fn test_try_from_valid() {
        let bytes = vec![1, 2, 3, 4, 5];
        let id = PubkySocialFile(bytes.clone()).create_id();

        let result = PubkySocialFile::from_bytes(&bytes, &id);
        assert_eq!(result.unwrap().0, bytes);
    }

    #[test]
    fn test_try_from_invalid_id() {
        let result = PubkySocialFile::from_bytes(&[1, 2, 3], "INVALIDID");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_file() {
        let created =
            PubkySocialFile::create_file(vec![1, 2], "image/svg+xml", Root::Priv).unwrap();
        assert_eq!(created.id, KAT);
        assert_eq!(created.path, format!("/priv/social/v1/files/{KAT}.svg"));
        assert_eq!(created.file.0, vec![1, 2]);

        // A typeless upload and one the map does not carry both land on .bin
        let created = PubkySocialFile::create_file(vec![1, 2], "", Root::Pub).unwrap();
        assert_eq!(created.path, format!("/pub/social/v1/files/{KAT}.bin"));
        let created =
            PubkySocialFile::create_file(vec![1, 2], "application/octet-stream", Root::Pub)
                .unwrap();
        assert_eq!(created.path, format!("/pub/social/v1/files/{KAT}.bin"));

        assert!(PubkySocialFile::create_file(vec![], "image/png", Root::Pub).is_err());
    }

    #[test]
    fn test_create_path_and_builder() {
        assert_eq!(
            PubkySocialFile::create_path(&format!("{KAT}.png")),
            format!("/pub/social/v1/files/{KAT}.png")
        );
        assert_eq!(
            file_uri_builder("user_id".into(), format!("{KAT}.png")),
            format!("pubky://user_id/pub/social/v1/files/{KAT}.png")
        );
    }
}
