use crate::{
    limits::VALIDATION_LIMITS,
    models::file::VALID_MIME_TYPES,
    traits::{HashId, Validatable},
    PubkyAppPost, PubkyAppPostKind,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use mime::Mime;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[cfg(target_arch = "wasm32")]
use crate::traits::Json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// One file inlined inside a [`PubkyAppLock`] resource, with its bytes
/// base64-encoded in `content_base64`. See [`PubkyAppLock`] for the security
/// model and the resource-wide size cap.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(deny_unknown_fields)]
pub struct PubkyAppLockFile {
    /// Original display name of the file (e.g. `episode.mp3`). Length bounded by
    /// `VALIDATION_LIMITS.file_name_{min,max}_length`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub name: String,
    /// MIME type of the file. Must be one of `VALID_MIME_TYPES`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub content_type: String,
    /// Decoded size of the file in bytes. Must be non-zero and equal to the
    /// actual decoded length of `content_base64`.
    pub size: usize,
    /// The file bytes, base64-encoded (standard alphabet). Decoded length must
    /// equal `size`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub content_base64: String,
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppLockFile {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn name(&self) -> String {
        self.name.clone()
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn content_type(&self) -> String {
        self.content_type.clone()
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter, js_name = contentBase64))]
    pub fn content_base64(&self) -> String {
        self.content_base64.clone()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppLockFile {
    /// Creates a new `PubkyAppLockFile` and sanitizes it.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(name: String, content_type: String, size: usize, content_base64: String) -> Self {
        Self {
            name,
            content_type,
            size,
            content_base64,
        }
        .sanitize()
    }
}

impl Validatable for PubkyAppLockFile {
    fn sanitize(self) -> Self {
        Self {
            name: self.name.trim().to_string(),
            content_type: self.content_type.trim().to_string(),
            size: self.size,
            content_base64: self.content_base64.trim().to_string(),
        }
    }

    fn validate(&self, _id: Option<&str>) -> Result<(), String> {
        let name_length = self.name.chars().count();
        if !(VALIDATION_LIMITS.file_name_min_length..=VALIDATION_LIMITS.file_name_max_length)
            .contains(&name_length)
        {
            return Err("Validation Error: Lock file name length is invalid".into());
        }

        match Mime::from_str(&self.content_type) {
            Ok(mime) => {
                if !VALID_MIME_TYPES.contains(&mime.essence_str()) {
                    return Err("Validation Error: Lock file has invalid content type".into());
                }
            }
            Err(_) => return Err("Validation Error: Lock file has invalid content type".into()),
        }

        if self.size == 0 {
            return Err("Validation Error: Lock file size cannot be zero".into());
        }

        // Decode the inlined bytes and cross-check their length against `size`.
        // The overall size cap is enforced on the whole serialized resource in
        // `PubkyAppLock::validate`, so there is no per-file size limit here.
        let decoded = BASE64
            .decode(self.content_base64.as_bytes())
            .map_err(|_| "Validation Error: Lock file content_base64 is not valid base64")?;
        if decoded.len() != self.size {
            return Err(format!(
                "Validation Error: Lock file decoded length ({}) does not match size ({})",
                decoded.len(),
                self.size
            ));
        }

        Ok(())
    }
}

/// A locked content resource served by the Lock Server.
///
/// A locked post advertises a `lock` URL (in its public
/// `PubkyAppLockContent` envelope) pointing at this resource.
/// `PubkyAppLock` is a single, self-contained JSON resource that
/// bundles the full, unlocked `PubkyAppPost` together with all
/// of its attachment files, each inlined as base64 in
/// `files[].content_base64`. This resource is released only after
/// the Lock Server verifies access.
///
/// # Identity
///
/// `PubkyAppLock` is content-addressed via [`HashId`]: its ID is the blake3
/// hash of the serialized resource. The on-homeserver storage path is owned by
/// the Locks application/SDK and is intentionally not derived here.
///
/// # Security
///
/// There is no homeserver/backend validation of locks — anything the Lock
/// Server returns is untrusted. Clients MUST run [`Validatable::validate`] on
/// the parsed resource (which decodes every `content_base64` and checks its
/// length against `size`) and verify the resource ID matches its content hash
/// before presenting any content.
///
/// `deny_unknown_fields` is intentional: lock resources are content-addressed by
/// their canonical structured representation, so accepting ignored fields would
/// allow bytes outside the ID and size checks.
///
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(deny_unknown_fields)]
pub struct PubkyAppLock {
    /// The full, unlocked post packaged inside the resource. Its `attachments`
    /// MUST be empty — packed attachments are listed in `files`, which is the
    /// single source of truth; the client rebuilds `post.attachments` from
    /// `files` on unlock.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub post: PubkyAppPost,
    /// Ordered list of attachment files inlined alongside the post. Count
    /// bounded by `VALIDATION_LIMITS.post_attachments_max_count`; combined
    /// decoded size by `VALIDATION_LIMITS.lock_max_size_bytes`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub files: Vec<PubkyAppLockFile>,
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppLock {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn post(&self) -> PubkyAppPost {
        self.post.clone()
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn files(&self) -> Vec<PubkyAppLockFile> {
        self.files.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = fromJson))]
    pub fn from_json(js_value: &JsValue) -> Result<Self, String> {
        Self::import_json(js_value)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = toJson))]
    pub fn to_json(&self) -> Result<JsValue, String> {
        self.export_json()
    }
}

#[cfg(target_arch = "wasm32")]
impl Json for PubkyAppLock {}

impl PubkyAppLock {
    /// Creates a new `PubkyAppLock` resource and sanitizes it.
    pub fn new(post: PubkyAppPost, files: Vec<PubkyAppLockFile>) -> Self {
        Self { post, files }.sanitize()
    }
}

impl HashId for PubkyAppLock {
    /// Content-addresses the lock by hashing its full serialized form (post +
    /// inlined files). Computed on the sanitized resource so the ID is stable
    /// across the create/parse round-trip.
    fn get_id_data(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

impl Validatable for PubkyAppLock {
    fn sanitize(self) -> Self {
        Self {
            post: self.post.sanitize(),
            files: self.files.into_iter().map(|f| f.sanitize()).collect(),
        }
    }

    fn validate(&self, id: Option<&str>) -> Result<(), String> {
        if let Some(id) = id {
            self.validate_id(id)?;
        }

        // The `files` list is the single source of truth for packed
        // attachments, so the guarded post must not carry its own `attachments`
        // (they would duplicate/contradict `files`). On unlock, the client
        // reconstructs the post's attachments from `files`.
        if matches!(&self.post.attachments, Some(a) if !a.is_empty()) {
            return Err(
                "Validation Error: Lock post must not set attachments; packed files are listed in `files`"
                    .into(),
            );
        }

        // A lock guards real content; it must not itself be a `lock` post
        // (nested locks are not allowed).
        if self.post.kind == PubkyAppPostKind::Lock {
            return Err(
                "Validation Error: Lock post must not be a `lock` kind (nested locks are not allowed)"
                    .into(),
            );
        }

        // TODO: Check with the design team whether the private unlocked payload
        // should allow `parent` and `embed`, unlike the public lock teaser post.
        if self.files.len() > VALIDATION_LIMITS.post_attachments_max_count {
            return Err(format!(
                "Validation Error: Lock cannot contain more than {} files",
                VALIDATION_LIMITS.post_attachments_max_count
            ));
        }

        for (index, file) in self.files.iter().enumerate() {
            file.validate(None)
                .map_err(|e| format!("Validation Error: Lock file at index {index}: {e}"))?;
        }

        let mut post_for_validation = self.post.clone();
        if self.files.is_empty() {
            // Treat `None` and `Some([])` as equivalent for the stored lock
            // shape. An empty attachment list must not satisfy the generic
            // post "has attachments" requirement when there are no packed
            // files to reconstruct.
            post_for_validation.attachments = None;
        } else {
            // Packed files are semantically attachments after unlock, but the
            // serialized lock resource keeps `post.attachments` empty. Use
            // validation-only placeholder URLs so the regular post validator
            // still enforces kind/content/parent/embed rules without persisting
            // duplicate attachment pointers.
            post_for_validation.attachments = Some(
                (0..self.files.len())
                    .map(|index| format!("https://example.com/pubky-app-lock-validation/{index}"))
                    .collect(),
            );
        }
        post_for_validation
            .validate(None)
            .map_err(|e| format!("Validation Error: Lock post is invalid: {e}"))?;

        // The artifact written to the homeserver is the serialized JSON
        // resource (base64-inlined files + post + JSON overhead), so the size
        // cap is enforced against that, not against the raw decoded bytes.
        let serialized_len = serde_json::to_vec(self)
            .map(|v| v.len())
            .map_err(|e| format!("Validation Error: Lock failed to serialize: {e}"))?;
        if serialized_len > VALIDATION_LIMITS.lock_max_size_bytes {
            return Err(format!(
                "Validation Error: Lock resource size ({serialized_len} bytes) exceeds maximum of {} bytes",
                VALIDATION_LIMITS.lock_max_size_bytes
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::post::PubkyAppPostKind;
    use crate::traits::{HashId, Validatable};

    fn b64(bytes: &[u8]) -> String {
        BASE64.encode(bytes)
    }

    fn lock_file(name: &str, content_type: &str, bytes: &[u8]) -> PubkyAppLockFile {
        PubkyAppLockFile::new(
            name.to_string(),
            content_type.to_string(),
            bytes.len(),
            b64(bytes),
        )
    }

    fn sample_post() -> PubkyAppPost {
        PubkyAppPost::new(
            "The full unlocked article body.".to_string(),
            PubkyAppPostKind::Long,
            None,
            None,
            None,
        )
    }

    fn sample_lock() -> PubkyAppLock {
        PubkyAppLock::new(
            sample_post(),
            vec![
                lock_file("cover.png", "image/png", &[0u8; 2048]),
                lock_file("episode.mp3", "audio/mpeg", &[1u8; 4096]),
            ],
        )
    }

    #[test]
    fn test_new_keeps_files() {
        let lock = sample_lock();
        assert_eq!(lock.files.len(), 2);
    }

    #[test]
    fn test_validate_ok() {
        let lock = sample_lock();
        let id = lock.create_id();
        assert!(lock.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_id_is_stable_and_content_addressed() {
        let lock = sample_lock();
        // Same content -> same id.
        assert_eq!(lock.create_id(), sample_lock().create_id());
        // Different content -> different id.
        let other = PubkyAppLock::new(sample_post(), vec![]);
        assert_ne!(lock.create_id(), other.create_id());
    }

    #[test]
    fn test_validate_rejects_tampered_id() {
        let lock = sample_lock();
        let err = lock.validate(Some("0000000000000")).unwrap_err();
        assert!(err.contains("Invalid ID"), "got: {err}");
    }

    #[test]
    fn test_roundtrip_via_try_from() {
        let lock = sample_lock();
        let id = lock.create_id();
        let json = serde_json::to_vec(&lock).unwrap();
        let parsed = <PubkyAppLock as Validatable>::try_from(&json, &id).unwrap();
        assert_eq!(parsed.files.len(), 2);
        assert_eq!(parsed.post.content, "The full unlocked article body.");
    }

    #[test]
    fn test_rejects_invalid_inner_post() {
        // Empty post (no content/embed/attachments) fails PubkyAppPost validation.
        let lock = PubkyAppLock::new(
            PubkyAppPost::new("".to_string(), PubkyAppPostKind::Short, None, None, None),
            vec![],
        );
        let id = lock.create_id();
        let err = lock.validate(Some(&id)).unwrap_err();
        assert!(err.contains("post"), "got: {err}");
    }

    #[test]
    fn test_rejects_inner_post_with_attachments() {
        // `files` is the source of truth for packed attachments, so the guarded
        // post must not carry its own `attachments`.
        let post = PubkyAppPost::new(
            "Body".to_string(),
            PubkyAppPostKind::Long,
            None,
            None,
            Some(vec!["pubky://userA/pub/pubky.app/files/0034A0X7NJ52A".to_string()]),
        );
        let lock = PubkyAppLock::new(post, vec![lock_file("a.png", "image/png", &[0u8; 16])]);
        let id = lock.create_id();
        let err = lock.validate(Some(&id)).unwrap_err();
        assert!(err.contains("attachments"), "got: {err}");
    }

    #[test]
    fn test_accepts_attachment_only_media_posts_with_files() {
        for kind in [
            PubkyAppPostKind::Image,
            PubkyAppPostKind::Video,
            PubkyAppPostKind::File,
        ] {
            let post = PubkyAppPost::new("".to_string(), kind.clone(), None, None, None);
            let lock = PubkyAppLock::new(
                post,
                vec![lock_file("payload.bin", "application/octet-stream", &[0u8; 16])],
            );
            let id = lock.create_id();
            assert!(
                lock.validate(Some(&id)).is_ok(),
                "attachment-only {kind:?} post should validate with files"
            );
            assert!(
                lock.post.attachments.is_none(),
                "synthetic validation attachments must not mutate the stored post"
            );
            let json = serde_json::to_value(&lock).expect("lock serialization");
            assert!(json["post"]["attachments"].is_null());
        }
    }

    #[test]
    fn test_rejects_attachment_only_image_post_without_files() {
        let post = PubkyAppPost::new(
            "".to_string(),
            PubkyAppPostKind::Image,
            None,
            None,
            None,
        );
        let lock = PubkyAppLock::new(post, vec![]);
        let id = lock.create_id();
        let err = lock.validate(Some(&id)).unwrap_err();
        assert!(
            err.contains("content, an embed, or attachments"),
            "got: {err}"
        );
    }

    #[test]
    fn test_rejects_attachment_only_image_post_with_empty_attachment_list_and_no_files() {
        let post = PubkyAppPost::new(
            "".to_string(),
            PubkyAppPostKind::Image,
            None,
            None,
            Some(vec![]),
        );
        let lock = PubkyAppLock::new(post, vec![]);
        let id = lock.create_id();
        let err = lock.validate(Some(&id)).unwrap_err();
        assert!(
            err.contains("content, an embed, or attachments"),
            "got: {err}"
        );
    }

    #[test]
    fn test_rejects_attachment_only_post_with_invalid_parent() {
        let post = PubkyAppPost::new(
            "".to_string(),
            PubkyAppPostKind::Image,
            Some("not a url".to_string()),
            None,
            None,
        );
        let lock = PubkyAppLock::new(post, vec![lock_file("cover.png", "image/png", &[0u8; 16])]);
        let id = lock.create_id();
        let err = lock.validate(Some(&id)).unwrap_err();
        assert!(err.contains("Invalid parent URI format"), "got: {err}");
    }

    #[test]
    fn test_rejects_nested_lock_post() {
        // The guarded post must not itself be a `lock` kind. We build the inner
        // post from a valid lock envelope so it would otherwise pass, isolating
        // the nested-lock rejection.
        let envelope = serde_json::json!({
            "header": "Teaser",
            "title": "Locked",
            "header_kind": "short",
            "lock": "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/locks/0034A0X7NJ52G"
        })
        .to_string();
        let inner = PubkyAppPost::new(envelope, PubkyAppPostKind::Lock, None, None, None);
        let lock = PubkyAppLock::new(inner, vec![]);
        let id = lock.create_id();
        let err = lock.validate(Some(&id)).unwrap_err();
        assert!(err.contains("nested locks"), "got: {err}");
    }

    #[test]
    fn test_accepts_zero_files() {
        let lock = PubkyAppLock::new(sample_post(), vec![]);
        let id = lock.create_id();
        assert!(lock.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_rejects_too_many_files() {
        let files: Vec<PubkyAppLockFile> = (0..VALIDATION_LIMITS.post_attachments_max_count + 1)
            .map(|_| lock_file("f.png", "image/png", &[0u8; 16]))
            .collect();
        let lock = PubkyAppLock::new(sample_post(), files);
        let id = lock.create_id();
        let err = lock.validate(Some(&id)).unwrap_err();
        assert!(err.contains("more than"), "got: {err}");
    }

    #[test]
    fn test_rejects_invalid_content_type() {
        let lock = PubkyAppLock::new(
            sample_post(),
            vec![lock_file("a.bin", "not/a-real-type", &[0u8; 16])],
        );
        let id = lock.create_id();
        let err = lock.validate(Some(&id)).unwrap_err();
        assert!(err.contains("content type"), "got: {err}");
    }

    #[test]
    fn test_rejects_zero_size_file() {
        let lock = PubkyAppLock::new(
            sample_post(),
            vec![PubkyAppLockFile::new(
                "a.png".to_string(),
                "image/png".to_string(),
                0,
                String::new(),
            )],
        );
        let id = lock.create_id();
        let err = lock.validate(Some(&id)).unwrap_err();
        assert!(err.contains("size"), "got: {err}");
    }

    #[test]
    fn test_rejects_malformed_base64() {
        let lock = PubkyAppLock::new(
            sample_post(),
            vec![PubkyAppLockFile::new(
                "a.png".to_string(),
                "image/png".to_string(),
                3,
                "!!!not base64!!!".to_string(),
            )],
        );
        let id = lock.create_id();
        let err = lock.validate(Some(&id)).unwrap_err();
        assert!(err.contains("base64"), "got: {err}");
    }

    #[test]
    fn test_rejects_size_decoded_mismatch() {
        // `size` claims 100 bytes but the decoded content is only 4.
        let lock = PubkyAppLock::new(
            sample_post(),
            vec![PubkyAppLockFile::new(
                "a.png".to_string(),
                "image/png".to_string(),
                100,
                b64(&[0u8; 4]),
            )],
        );
        let id = lock.create_id();
        let err = lock.validate(Some(&id)).unwrap_err();
        assert!(err.contains("does not match size"), "got: {err}");
    }

    #[test]
    fn test_rejects_over_resource_size() {
        // The cap is on the serialized resource. Raw bytes equal to the cap
        // become ~1.33x larger once base64-encoded, so a single file of
        // `lock_max_size_bytes` raw bytes pushes the resource over the limit.
        // Validate with `None` to skip the (expensive) content-hash ID check on
        // this large payload — we only exercise the resource-size logic.
        let raw = vec![0u8; VALIDATION_LIMITS.lock_max_size_bytes];
        let lock = PubkyAppLock::new(
            sample_post(),
            vec![lock_file("a.bin", "application/octet-stream", &raw)],
        );
        let err = lock.validate(None).unwrap_err();
        assert!(err.contains("resource size"), "got: {err}");
    }

    #[test]
    fn test_sanitize_trims_fields() {
        let lock = PubkyAppLock::new(
            sample_post(),
            vec![PubkyAppLockFile::new(
                "  a.png  ".to_string(),
                "  image/png  ".to_string(),
                16,
                format!("  {}  ", b64(&[0u8; 16])),
            )],
        );
        assert_eq!(lock.files[0].name, "a.png");
        assert_eq!(lock.files[0].content_type, "image/png");
        assert_eq!(lock.files[0].content_base64, b64(&[0u8; 16]));
    }

    #[test]
    fn test_rejects_unknown_top_level_fields() {
        let json = r#"{"post":{"content":"hi","kind":"short","parent":null,"embed":null,"attachments":null},"files":[],"_future_field":"ignored"}"#;
        let err = serde_json::from_str::<PubkyAppLock>(json).unwrap_err();
        assert!(err.to_string().contains("unknown field"), "got: {err}");
    }

    #[test]
    fn test_rejects_unknown_file_fields() {
        let json = r#"{"post":{"content":"hi","kind":"short","parent":null,"embed":null,"attachments":null},"files":[{"name":"a.png","content_type":"image/png","size":4,"content_base64":"AAAAAA==","_future_field":"ignored"}]}"#;
        let err = serde_json::from_str::<PubkyAppLock>(json).unwrap_err();
        assert!(err.to_string().contains("unknown field"), "got: {err}");
    }
}
