//! `PubkyAppLockedPost`: the private single-blob bundle backing a locked post.
//! Holds the full unlocked [`PubkyAppPost`] plus its attachment files, released
//! by the Lock Server after it verifies access.
//!
//! Wire format (raw bytes, no base64, so no ~33% inflation and no full decode to
//! read just the manifest):
//!
//! ```text
//! magic "PALP" (4) | version (1) | manifest_len: u32 LE (4) | manifest JSON | file bytes…
//! ```
//!
//! manifest = `{ post, files: [{ name, content_type, size }] }`; file bytes
//! follow concatenated and are sliced back by `size`. The manifest is the index.
//!
//! Content-addressed: ID = Crockford-Base32 of the first half of the blake3 of
//! the exact stored bytes. The Lock Server is untrusted, so a client MUST re-hash
//! the fetched bytes against the `lock` URL's ID and `validate()` before showing
//! anything.
//!
//! ponytail: custom container, not zip. zip's timestamps/ordering/compression
//! are non-deterministic and break content-addressing, and the dep adds wasm
//! weight for ~0 gain on already-compressed media. Switch to zip-as-blob only if
//! an external reader (e.g. checkstep) must parse the payload without this spec.

use crate::limits::VALIDATION_LIMITS;
use crate::traits::Validatable;
use crate::{PubkyAppPost, APP_PATH, VALID_MIME_TYPES};
use base32::{encode, Alphabet};
use blake3::Hasher;
use mime::Mime;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

const MAGIC: &[u8; 4] = b"PALP";
const FORMAT_VERSION: u8 = 1;
const HEADER_LEN: usize = 4 + 1 + 4; // magic + version + manifest_len

/// One attachment file packed in a [`PubkyAppLockedPost`], as raw bytes.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct LockedFile {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub name: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub content_type: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub bytes: Vec<u8>,
}

impl LockedFile {
    pub fn new(name: String, content_type: String, bytes: Vec<u8>) -> Self {
        Self {
            name,
            content_type,
            bytes,
        }
        .sanitize()
    }

    fn sanitize(self) -> Self {
        Self {
            name: self.name.trim().to_string(),
            content_type: self.content_type.trim().to_string(),
            bytes: self.bytes,
        }
    }

    fn validate(&self) -> Result<(), String> {
        let name_len = self.name.chars().count();
        if !(VALIDATION_LIMITS.file_name_min_length..=VALIDATION_LIMITS.file_name_max_length)
            .contains(&name_len)
        {
            return Err("file name length is invalid".into());
        }
        match Mime::from_str(&self.content_type) {
            Ok(mime) if VALID_MIME_TYPES.contains(&mime.essence_str()) => {}
            _ => return Err("invalid content type".into()),
        }
        if self.bytes.is_empty() {
            return Err("file cannot be empty".into());
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl LockedFile {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn content_type(&self) -> String {
        self.content_type.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> js_sys::Uint8Array {
        js_sys::Uint8Array::from(&self.bytes[..])
    }
}

/// Manifest portion of the container (everything except the raw file bytes).
#[derive(Serialize, Deserialize)]
struct Manifest {
    post: PubkyAppPost,
    files: Vec<FileEntry>,
}

#[derive(Serialize, Deserialize)]
struct FileEntry {
    name: String,
    content_type: String,
    size: u64,
}

/// The private, single-blob bundle backing a locked post. See the module docs.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct PubkyAppLockedPost {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub post: PubkyAppPost,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub files: Vec<LockedFile>,
}

impl PubkyAppLockedPost {
    pub fn new(post: PubkyAppPost, files: Vec<LockedFile>) -> Self {
        Self {
            post: post.sanitize(),
            files: files.into_iter().map(LockedFile::sanitize).collect(),
        }
    }

    /// Homeserver path for the bundle: `/priv/pubky.app/posts/:id`.
    pub fn create_path(id: &str) -> String {
        ["/priv/", APP_PATH, "posts/", id].concat()
    }

    /// Canonical bytes: what gets stored on the homeserver and hashed for the ID.
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let manifest = Manifest {
            post: self.post.clone(),
            files: self
                .files
                .iter()
                .map(|f| FileEntry {
                    name: f.name.clone(),
                    content_type: f.content_type.clone(),
                    size: f.bytes.len() as u64,
                })
                .collect(),
        };
        let manifest_json = serde_json::to_vec(&manifest)
            .map_err(|e| format!("Failed to serialize lock manifest: {e}"))?;
        let manifest_len = u32::try_from(manifest_json.len())
            .map_err(|_| "Lock manifest exceeds 4 GiB".to_string())?;
        let heap_len: usize = self.files.iter().map(|f| f.bytes.len()).sum();

        let mut out = Vec::with_capacity(HEADER_LEN + manifest_json.len() + heap_len);
        out.extend_from_slice(MAGIC);
        out.push(FORMAT_VERSION);
        out.extend_from_slice(&manifest_len.to_le_bytes());
        out.extend_from_slice(&manifest_json);
        for f in &self.files {
            out.extend_from_slice(&f.bytes);
        }
        Ok(out)
    }

    /// Does NOT sanitize/validate: bytes are preserved exactly so the caller can
    /// verify the content hash. Run [`Self::validate`] afterwards.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        // Fail fast on untrusted server bytes before copying any file slices.
        if bytes.len() > VALIDATION_LIMITS.max_blob_size_bytes {
            return Err(format!(
                "Invalid lock bundle: size ({} bytes) exceeds maximum of {} bytes",
                bytes.len(),
                VALIDATION_LIMITS.max_blob_size_bytes
            ));
        }
        if bytes.len() < HEADER_LEN {
            return Err("Invalid lock bundle: truncated header".into());
        }
        if &bytes[0..4] != MAGIC {
            return Err("Invalid lock bundle: bad magic".into());
        }
        if bytes[4] != FORMAT_VERSION {
            return Err(format!(
                "Invalid lock bundle: unsupported version {}",
                bytes[4]
            ));
        }
        let manifest_len = u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]) as usize;
        let heap_start = HEADER_LEN
            .checked_add(manifest_len)
            .ok_or("Invalid lock bundle: manifest length overflow")?;
        if bytes.len() < heap_start {
            return Err("Invalid lock bundle: manifest truncated".into());
        }
        let manifest: Manifest = serde_json::from_slice(&bytes[HEADER_LEN..heap_start])
            .map_err(|e| format!("Invalid lock bundle manifest: {e}"))?;

        let mut files = Vec::with_capacity(manifest.files.len());
        let mut cursor = heap_start;
        for entry in &manifest.files {
            let size = usize::try_from(entry.size)
                .map_err(|_| "Invalid lock bundle: file size overflow")?;
            let end = cursor
                .checked_add(size)
                .ok_or("Invalid lock bundle: file size overflow")?;
            if end > bytes.len() {
                return Err("Invalid lock bundle: file data truncated".into());
            }
            files.push(LockedFile {
                name: entry.name.clone(),
                content_type: entry.content_type.clone(),
                bytes: bytes[cursor..end].to_vec(),
            });
            cursor = end;
        }
        if cursor != bytes.len() {
            return Err("Invalid lock bundle: trailing bytes after file data".into());
        }
        Ok(Self {
            post: manifest.post,
            files,
        })
    }

    /// Content-addressed ID over the canonical bytes.
    pub fn create_id(&self) -> Result<String, String> {
        Ok(id_from_bytes(&self.to_bytes()?))
    }

    /// When `id` is supplied, verifies it equals the content hash.
    pub fn validate(&self, id: Option<&str>) -> Result<(), String> {
        let bytes = self.to_bytes()?;
        if let Some(id) = id {
            let expected = id_from_bytes(&bytes);
            if expected != id {
                return Err(format!("Invalid ID: expected {expected}, found {id}"));
            }
        }

        // `files` is the single source of truth for the post's attachments, so
        // the packed post must not carry its own; nor may it itself be locked.
        if matches!(&self.post.attachments, Some(a) if !a.is_empty()) {
            return Err(
                "Validation Error: Locked post must not set attachments; packed files are listed in `files`"
                    .into(),
            );
        }
        if self.post.lock.is_some() {
            return Err(
                "Validation Error: Locked post payload must not set a `lock` (nested locks are not allowed)"
                    .into(),
            );
        }
        if self.files.len() > VALIDATION_LIMITS.post_attachments_max_count {
            return Err(format!(
                "Validation Error: Locked post cannot contain more than {} files",
                VALIDATION_LIMITS.post_attachments_max_count
            ));
        }
        for (index, file) in self.files.iter().enumerate() {
            file.validate()
                .map_err(|e| format!("Validation Error: Locked post file at index {index}: {e}"))?;
        }

        // Packed files stand in for the post's not-yet-written attachments.
        self.post
            .validate_in_lock_bundle(!self.files.is_empty())
            .map_err(|e| format!("Validation Error: Locked post payload is invalid: {e}"))?;

        if bytes.len() > VALIDATION_LIMITS.max_blob_size_bytes {
            return Err(format!(
                "Validation Error: Locked post bundle size ({} bytes) exceeds maximum of {} bytes",
                bytes.len(),
                VALIDATION_LIMITS.max_blob_size_bytes
            ));
        }
        Ok(())
    }
}

/// Crockford-Base32 of the first half of the blake3 hash of `bytes`.
fn id_from_bytes(bytes: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    let half = &hash.as_bytes()[..hash.as_bytes().len() / 2];
    encode(Alphabet::Crockford, half)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl PubkyAppLockedPost {
    #[wasm_bindgen(getter)]
    pub fn post(&self) -> PubkyAppPost {
        self.post.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn files(&self) -> Vec<LockedFile> {
        self.files.clone()
    }
    #[wasm_bindgen(js_name = toBytes)]
    pub fn to_bytes_js(&self) -> Result<js_sys::Uint8Array, String> {
        Ok(js_sys::Uint8Array::from(&self.to_bytes()?[..]))
    }
    #[wasm_bindgen(js_name = fromBytes)]
    pub fn from_bytes_js(bytes: &[u8]) -> Result<PubkyAppLockedPost, String> {
        Self::from_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PubkyAppPostKind;

    fn lf(name: &str, ct: &str, bytes: &[u8]) -> LockedFile {
        LockedFile::new(name.to_string(), ct.to_string(), bytes.to_vec())
    }

    fn sample() -> PubkyAppLockedPost {
        let post = PubkyAppPost::new(
            "The full unlocked article body.".to_string(),
            PubkyAppPostKind::Long,
            None,
            None,
            None,
        );
        PubkyAppLockedPost::new(
            post,
            vec![
                lf("cover.png", "image/png", &[7u8; 2048]),
                lf("episode.mp3", "audio/mpeg", &[3u8; 4096]),
            ],
        )
    }

    #[test]
    fn test_roundtrip_bytes_preserves_content() {
        let bundle = sample();
        let bytes = bundle.to_bytes().unwrap();
        let parsed = PubkyAppLockedPost::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.post.content, "The full unlocked article body.");
        assert_eq!(parsed.files.len(), 2);
        assert_eq!(parsed.files[0].name, "cover.png");
        assert_eq!(parsed.files[0].bytes, vec![7u8; 2048]);
        assert_eq!(parsed.files[1].bytes, vec![3u8; 4096]);
        // Byte-exact round-trip is what makes the content hash stable on read.
        assert_eq!(parsed.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn test_id_is_content_addressed_and_validates() {
        let bundle = sample();
        let id = bundle.create_id().unwrap();
        assert!(bundle.validate(Some(&id)).is_ok());
        // Same content -> same id; different content -> different id.
        assert_eq!(id, sample().create_id().unwrap());
        let other =
            PubkyAppLockedPost::new(sample().post, vec![lf("a.png", "image/png", &[1u8; 8])]);
        assert_ne!(id, other.create_id().unwrap());
    }

    #[test]
    fn test_validate_rejects_tampered_id() {
        let bundle = sample();
        let err = bundle.validate(Some("0000000000000")).unwrap_err();
        assert!(err.contains("Invalid ID"), "got: {err}");
    }

    #[test]
    fn test_no_base64_overhead_payload_stored_raw() {
        // The raw file bytes appear verbatim in the container (no base64).
        let bundle = PubkyAppLockedPost::new(
            PubkyAppPost::new("hi".to_string(), PubkyAppPostKind::Short, None, None, None),
            vec![lf("a.bin", "application/octet-stream", b"RAWBYTES")],
        );
        let bytes = bundle.to_bytes().unwrap();
        assert!(
            bytes.windows(8).any(|w| w == b"RAWBYTES"),
            "raw file bytes must be embedded verbatim, not base64-encoded"
        );
    }

    #[test]
    fn test_accepts_attachment_only_media_without_placeholder_hack() {
        // Empty-content image post is valid in a bundle because the packed file
        // stands in for the attachment (no synthesized placeholder URLs).
        for kind in [
            PubkyAppPostKind::Image,
            PubkyAppPostKind::Video,
            PubkyAppPostKind::File,
        ] {
            let post = PubkyAppPost::new("".to_string(), kind, None, None, None);
            let bundle = PubkyAppLockedPost::new(
                post,
                vec![lf("p.bin", "application/octet-stream", &[0u8; 16])],
            );
            let id = bundle.create_id().unwrap();
            assert!(bundle.validate(Some(&id)).is_ok());
        }
    }

    #[test]
    fn test_rejects_post_with_attachments() {
        let post = PubkyAppPost::new(
            "Body".to_string(),
            PubkyAppPostKind::Long,
            None,
            None,
            Some(vec![
                "pubky://userA/pub/pubky.app/files/0034A0X7NJ52A".to_string()
            ]),
        );
        let bundle = PubkyAppLockedPost::new(post, vec![lf("a.png", "image/png", &[0u8; 16])]);
        let err = bundle
            .validate(Some(&bundle.create_id().unwrap()))
            .unwrap_err();
        assert!(err.contains("must not set attachments"), "got: {err}");
    }

    #[test]
    fn test_rejects_nested_lock() {
        let mut post =
            PubkyAppPost::new("Body".to_string(), PubkyAppPostKind::Long, None, None, None);
        post.lock = Some("pubky://server/pub/locks/x".to_string());
        let bundle = PubkyAppLockedPost::new(post, vec![]);
        let err = bundle
            .validate(Some(&bundle.create_id().unwrap()))
            .unwrap_err();
        assert!(err.contains("nested locks"), "got: {err}");
    }

    #[test]
    fn test_rejects_empty_payload() {
        // No content, no files -> the inner post has nothing meaningful.
        let bundle = PubkyAppLockedPost::new(
            PubkyAppPost::new("".to_string(), PubkyAppPostKind::Short, None, None, None),
            vec![],
        );
        let err = bundle
            .validate(Some(&bundle.create_id().unwrap()))
            .unwrap_err();
        assert!(err.contains("payload is invalid"), "got: {err}");
    }

    #[test]
    fn test_rejects_too_many_files() {
        let files: Vec<LockedFile> = (0..VALIDATION_LIMITS.post_attachments_max_count + 1)
            .map(|i| lf(&format!("f{i}.bin"), "application/octet-stream", &[0u8; 4]))
            .collect();
        let post = PubkyAppPost::new(
            "body".to_string(),
            PubkyAppPostKind::Short,
            None,
            None,
            None,
        );
        let bundle = PubkyAppLockedPost::new(post, files);
        let err = bundle
            .validate(Some(&bundle.create_id().unwrap()))
            .unwrap_err();
        assert!(err.contains("more than"), "got: {err}");
    }

    #[test]
    fn test_from_bytes_rejects_truncation_and_bad_magic() {
        let bytes = sample().to_bytes().unwrap();
        assert!(PubkyAppLockedPost::from_bytes(&bytes[..bytes.len() - 1]).is_err());
        let mut bad = bytes.clone();
        bad[0] = b'X';
        assert!(PubkyAppLockedPost::from_bytes(&bad).is_err());
    }

    // Golden vector: fixed inputs pin the exact wire bytes and ID, so any change
    // to the manifest serialization that would break cross-version interop fails
    // here loudly.
    #[test]
    fn test_golden_vector() {
        let post = PubkyAppPost::new(
            "hello".to_string(),
            PubkyAppPostKind::Short,
            None,
            None,
            None,
        );
        let bundle = PubkyAppLockedPost::new(post, vec![lf("a.txt", "text/plain", b"hi")]);
        let bytes = bundle.to_bytes().unwrap();

        assert_eq!(&bytes[0..4], MAGIC);
        assert_eq!(bytes[4], FORMAT_VERSION);
        let manifest_len = u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]) as usize;
        let manifest = std::str::from_utf8(&bytes[HEADER_LEN..HEADER_LEN + manifest_len]).unwrap();
        assert_eq!(
            manifest,
            r#"{"post":{"content":"hello","kind":"short","parent":null,"embed":null,"attachments":null},"files":[{"name":"a.txt","content_type":"text/plain","size":2}]}"#
        );
        // Heap is the raw file bytes, verbatim.
        assert_eq!(&bytes[HEADER_LEN + manifest_len..], b"hi");
        assert_eq!(bundle.create_id().unwrap(), "11PMFD92YZWX5QCTP1T9FF6YH4");
    }
}
