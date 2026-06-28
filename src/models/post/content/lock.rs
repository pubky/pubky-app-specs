use crate::limits::VALIDATION_LIMITS;
use crate::PROTOCOL;
use serde::{Deserialize, Serialize};
use url::Url;

#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use super::super::{PubkyAppPost, PubkyAppPostKind};

/// Post kinds permitted as a lock teaser's `header_kind`.
///
/// Restricted to the simple display kinds used to render the public lock card.
/// `long`, `collection`, `lock`, and unknown kinds are intentionally excluded.
pub const ALLOWED_LOCK_HEADER_KINDS: [PubkyAppPostKind; 5] = [
    PubkyAppPostKind::Short,
    PubkyAppPostKind::Image,
    PubkyAppPostKind::Video,
    PubkyAppPostKind::Link,
    PubkyAppPostKind::File,
];

/// Typed JSON envelope stored in the `content` of a `kind = lock` post.
///
/// A locked post advertises gate metadata (`header`, `title`, `header_kind`)
/// plus the `lock` URL in this public `content` JSON envelope. The full guarded
/// payload lives in private storage and is served by the Lock Server after
/// verification.
///
/// **Construction**: deserialized from the post's `content` JSON during validation.
/// Re-exported publicly (via `lib.rs`) for SDK/OpenAPI consumers.
///
/// Forward-compat: `#[serde(deny_unknown_fields)]` is intentionally NOT used so
/// future minor versions can add fields without breaking older parsers.
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[serde(rename_all = "snake_case")]
pub struct PubkyAppLockContent {
    /// Public teaser used as the lock post's body, interpreted per `header_kind`
    /// (any media it references comes from the post's `attachments`). Length
    /// bounded by `VALIDATION_LIMITS.lock_content_header_max_length` (unicode scalars);
    /// whitespace-only headers are rejected separately by the validator.
    pub header: String,
    /// Short label on the lock card ("Locked Article", "Premium episode").
    /// Length bounded by `VALIDATION_LIMITS.lock_content_title_{min,max}_length`.
    /// Whitespace-only titles are rejected separately by the validator.
    pub title: String,
    /// Display kind of the guarded content, used to render the public teaser
    /// card. Restricted to the simple display kinds: `short`, `image`, `video`,
    /// `link`, `file`. `long`, `collection`, `lock`, and unknown kinds are
    /// rejected by the validator.
    pub header_kind: PubkyAppPostKind,
    /// Lock Server URL the guarded bundle is served from. Must be a `pubky://`
    /// URL with a host, up to `VALIDATION_LIMITS.post_attachment_url_max_length`
    /// characters.
    pub lock: String,
}

/// Validates a `kind = Lock` post, including its JSON content envelope.
pub(crate) fn validate_lock_post(post: &PubkyAppPost) -> Result<(), String> {
    if post.parent.is_some() || post.embed.is_some() {
        return Err("Validation Error: Locked posts cannot have parent or embed".into());
    }
    // Teaser media (e.g. for an `image`/`file` header_kind) is carried in the
    // post's `attachments` and validated with the same rules as any other post.
    super::super::validate_attachments(&post.attachments)?;
    if post.content.chars().count() > VALIDATION_LIMITS.lock_content_max_length {
        return Err(format!(
            "Validation Error: Lock content exceeds max length {}",
            VALIDATION_LIMITS.lock_content_max_length
        ));
    }
    let envelope: PubkyAppLockContent = serde_json::from_str(&post.content).map_err(|e| {
        format!(
            "Validation Error: Lock content must be a valid JSON envelope: {}",
            e
        )
    })?;
    validate_lock_envelope(&envelope)
}

fn validate_lock_envelope(envelope: &PubkyAppLockContent) -> Result<(), String> {
    if envelope.header.trim().is_empty() {
        return Err(
            "Validation Error: Lock header must contain non-whitespace characters".into(),
        );
    }
    let header_chars = envelope.header.chars().count();
    if header_chars > VALIDATION_LIMITS.lock_content_header_max_length {
        return Err(format!(
            "Validation Error: Lock header exceeds {} characters",
            VALIDATION_LIMITS.lock_content_header_max_length
        ));
    }
    if envelope.title.trim().is_empty() {
        return Err(
            "Validation Error: Lock title must contain non-whitespace characters".into(),
        );
    }
    let title_chars = envelope.title.chars().count();
    let title_min = VALIDATION_LIMITS.lock_content_title_min_length;
    let title_max = VALIDATION_LIMITS.lock_content_title_max_length;
    if !(title_min..=title_max).contains(&title_chars) {
        return Err(format!(
            "Validation Error: Lock title must be {}..={} characters",
            title_min, title_max
        ));
    }

    // `header_kind` is restricted to the simple display kinds. `long`,
    // `collection`, `lock`, and `unknown` are rejected.
    if !ALLOWED_LOCK_HEADER_KINDS.contains(&envelope.header_kind) {
        return Err(
            "Validation Error: Lock header_kind must be one of short, image, video, link, file"
                .into(),
        );
    }

    validate_lock_url(&envelope.lock)
}

/// Validates the envelope `lock` URL: non-empty, length-capped, `pubky://`
/// scheme, and a present host. Mirrors the rules previously enforced on the
/// top-level `post.lock` field.
fn validate_lock_url(lock_url: &str) -> Result<(), String> {
    if lock_url.trim().is_empty() {
        return Err("Validation Error: Lock URL cannot be empty".into());
    }
    if lock_url.chars().count() > VALIDATION_LIMITS.post_attachment_url_max_length {
        return Err(format!(
            "Validation Error: Lock URL exceeds maximum length (max: {} characters)",
            VALIDATION_LIMITS.post_attachment_url_max_length
        ));
    }
    let parsed = Url::parse(lock_url)
        .map_err(|_| format!("Validation Error: Invalid lock URL format: {lock_url}"))?;
    if parsed.scheme() != PROTOCOL.trim_end_matches("://") {
        return Err(format!(
            "Validation Error: Lock URL must use the {PROTOCOL} scheme: {lock_url}"
        ));
    }
    // Reject opaque URLs like `pubky:lock-id` that carry the scheme but no
    // authority and so point at no resolvable lock server.
    if parsed.host().is_none() {
        return Err(format!(
            "Validation Error: Lock URL must include a host: {lock_url}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::{PubkyAppPost, PubkyAppPostKind};
    use crate::traits::{TimestampId, Validatable};

    const TEST_PUBKY_ID: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";

    fn default_lock() -> String {
        format!("pubky://{TEST_PUBKY_ID}/pub/locks/0034A0X7NJ52G")
    }

    fn lock_envelope_json(
        header: &str,
        title: &str,
        header_kind: PubkyAppPostKind,
        lock: &str,
    ) -> String {
        serde_json::to_string(&PubkyAppLockContent {
            header: header.to_string(),
            title: title.to_string(),
            header_kind,
            lock: lock.to_string(),
        })
        .unwrap()
    }

    /// Builds a `kind = Lock` post with a valid `header_kind` (Short) and the
    /// default lock URL, so the only variables under test are `header`/`title`.
    fn make_locked_post(header: &str, title: &str) -> PubkyAppPost {
        PubkyAppPost::new(
            lock_envelope_json(header, title, PubkyAppPostKind::Short, &default_lock()),
            PubkyAppPostKind::Lock,
            None,
            None,
            None,
        )
    }

    /// Builds a `kind = Lock` post from a raw `content` envelope string.
    fn make_lock_post_from_content(content: &str) -> PubkyAppPost {
        PubkyAppPost::new(
            content.to_string(),
            PubkyAppPostKind::Lock,
            None,
            None,
            None,
        )
    }

    #[test]
    fn test_lock_post_roundtrip_valid() {
        let lock = default_lock();
        let content = lock_envelope_json(
            "We were reckless adopting Lightning without understanding the tradeoffs.",
            "Locked Article",
            PubkyAppPostKind::Image,
            &lock,
        );
        let post = make_lock_post_from_content(&content);
        let id = post.create_id();
        let blob = serde_json::to_vec(&post).unwrap();
        let parsed = <PubkyAppPost as Validatable>::try_from(&blob, &id).unwrap();
        assert_eq!(parsed.kind, PubkyAppPostKind::Lock);
        assert!(parsed.attachments.is_none());
        let envelope: PubkyAppLockContent = serde_json::from_str(&parsed.content).unwrap();
        assert_eq!(
            envelope.header,
            "We were reckless adopting Lightning without understanding the tradeoffs."
        );
        assert_eq!(envelope.title, "Locked Article");
        assert_eq!(envelope.header_kind, PubkyAppPostKind::Image);
        assert_eq!(envelope.lock, lock);
    }

    #[test]
    fn test_lock_post_rejects_malformed_envelope() {
        let post = make_lock_post_from_content("not json");
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("JSON envelope"), "got: {err}");
    }

    #[test]
    fn test_lock_post_rejects_empty_header() {
        let post = make_locked_post("", "Locked Article");
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("header"), "got: {err}");
    }

    #[test]
    fn test_lock_post_rejects_oversized_header() {
        let oversized = "a".repeat(VALIDATION_LIMITS.lock_content_header_max_length + 1);
        let post = make_locked_post(&oversized, "Locked Article");
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("header"), "got: {err}");
    }

    #[test]
    fn test_lock_post_accepts_max_header() {
        let exactly_max = "a".repeat(VALIDATION_LIMITS.lock_content_header_max_length);
        let post = make_locked_post(&exactly_max, "Locked Article");
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_lock_post_rejects_whitespace_only_header() {
        let post = make_locked_post("    ", "Locked Article");
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("whitespace"), "got: {err}");
    }

    #[test]
    fn test_lock_post_rejects_empty_title() {
        let post = make_locked_post("Teaser text", "");
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("title"), "got: {err}");
    }

    #[test]
    fn test_lock_post_rejects_oversized_title() {
        let oversized = "a".repeat(99) + "🚀🚀";
        assert_eq!(oversized.chars().count(), 101);
        let post = make_locked_post("Teaser text", &oversized);
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("title"), "got: {err}");
    }

    #[test]
    fn test_lock_post_accepts_max_title() {
        let exactly_100 = "a".repeat(100);
        let post = make_locked_post("Teaser text", &exactly_100);
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_lock_post_rejects_whitespace_only_title() {
        let post = make_locked_post("Teaser text", "    ");
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("whitespace"), "got: {err}");
    }

    #[test]
    fn test_lock_post_counts_whitespace_in_title_length() {
        let padded = format!(" {} ", "a".repeat(99));
        assert_eq!(padded.chars().count(), 101);
        let post = make_locked_post("Teaser text", &padded);
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("1..=100"), "got: {err}");
    }

    #[test]
    fn test_lock_post_rejects_missing_header() {
        let lock = default_lock();
        let envelope =
            format!(r#"{{ "title": "Locked Article", "header_kind": "short", "lock": "{lock}" }}"#);
        let post = make_lock_post_from_content(&envelope);
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(
            err.contains("header") || err.to_lowercase().contains("missing"),
            "got: {err}"
        );
    }

    #[test]
    fn test_lock_post_rejects_missing_title() {
        let lock = default_lock();
        let envelope =
            format!(r#"{{ "header": "Teaser text", "header_kind": "short", "lock": "{lock}" }}"#);
        let post = make_lock_post_from_content(&envelope);
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(
            err.contains("title") || err.to_lowercase().contains("missing"),
            "got: {err}"
        );
    }

    #[test]
    fn test_lock_post_rejects_missing_header_kind() {
        let lock = default_lock();
        let envelope =
            format!(r#"{{ "header": "Teaser text", "title": "Locked Article", "lock": "{lock}" }}"#);
        let post = make_lock_post_from_content(&envelope);
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(
            err.contains("header_kind") || err.to_lowercase().contains("missing"),
            "got: {err}"
        );
    }

    #[test]
    fn test_lock_post_rejects_missing_lock() {
        let envelope = r#"{ "header": "Teaser text", "title": "Locked Article", "header_kind": "short" }"#;
        let post = make_lock_post_from_content(envelope);
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(
            err.contains("lock") || err.to_lowercase().contains("missing"),
            "got: {err}"
        );
    }

    #[test]
    fn test_lock_post_accepts_each_simple_header_kind() {
        for hk in [
            PubkyAppPostKind::Short,
            PubkyAppPostKind::Image,
            PubkyAppPostKind::Video,
            PubkyAppPostKind::Link,
            PubkyAppPostKind::File,
        ] {
            let content =
                lock_envelope_json("Teaser text", "Locked Article", hk.clone(), &default_lock());
            let post = make_lock_post_from_content(&content);
            let id = post.create_id();
            assert!(
                post.validate(Some(&id)).is_ok(),
                "header_kind {hk:?} should be accepted"
            );
        }
    }

    #[test]
    fn test_lock_post_rejects_disallowed_header_kind() {
        for hk in [
            PubkyAppPostKind::Long,
            PubkyAppPostKind::Collection,
            PubkyAppPostKind::Lock,
        ] {
            let content =
                lock_envelope_json("Teaser text", "Locked Article", hk.clone(), &default_lock());
            let post = make_lock_post_from_content(&content);
            let id = post.create_id();
            let err = post.validate(Some(&id)).unwrap_err();
            assert!(
                err.contains("header_kind"),
                "header_kind {hk:?} should be rejected, got: {err}"
            );
        }
    }

    #[test]
    fn test_lock_post_rejects_non_pubky_lock_url() {
        let content = lock_envelope_json(
            "Teaser text",
            "Locked Article",
            PubkyAppPostKind::Short,
            "https://locks.example.com/session/0034A0X7NJ52G",
        );
        let post = make_lock_post_from_content(&content);
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("must use the pubky:// scheme"), "got: {err}");
    }

    #[test]
    fn test_lock_post_rejects_invalid_lock_url() {
        let content = lock_envelope_json(
            "Teaser text",
            "Locked Article",
            PubkyAppPostKind::Short,
            "not a url",
        );
        let post = make_lock_post_from_content(&content);
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("Invalid lock URL format"), "got: {err}");
    }

    #[test]
    fn test_lock_post_rejects_hostless_lock_url() {
        let content = lock_envelope_json(
            "Teaser text",
            "Locked Article",
            PubkyAppPostKind::Short,
            "pubky:lock-id",
        );
        let post = make_lock_post_from_content(&content);
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("must include a host"), "got: {err}");
    }

    #[test]
    fn test_lock_post_rejects_empty_lock_url() {
        let content =
            lock_envelope_json("Teaser text", "Locked Article", PubkyAppPostKind::Short, "");
        let post = make_lock_post_from_content(&content);
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("Lock URL cannot be empty"), "got: {err}");
    }

    #[test]
    fn test_lock_post_rejects_parent() {
        let content =
            lock_envelope_json("Teaser", "Locked Article", PubkyAppPostKind::Short, &default_lock());
        let post = PubkyAppPost::new(
            content,
            PubkyAppPostKind::Lock,
            Some("pubky://userA/pub/pubky.app/posts/0034A0X7NJ52A".to_string()),
            None,
            None,
        );
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("parent or embed"), "got: {err}");
    }

    #[test]
    fn test_lock_post_accepts_valid_attachments() {
        // Teaser media (e.g. an `image` header_kind) is carried in post.attachments.
        let content =
            lock_envelope_json("Teaser", "Locked Article", PubkyAppPostKind::Image, &default_lock());
        let post = PubkyAppPost::new(
            content,
            PubkyAppPostKind::Lock,
            None,
            None,
            Some(vec!["pubky://userA/pub/pubky.app/files/0034A0X7NJ52A".to_string()]),
        );
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_lock_post_rejects_invalid_attachment_url() {
        let content =
            lock_envelope_json("Teaser", "Locked Article", PubkyAppPostKind::Image, &default_lock());
        let post = PubkyAppPost::new(
            content,
            PubkyAppPostKind::Lock,
            None,
            None,
            Some(vec!["not a valid url".to_string()]),
        );
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("attachment URL"), "got: {err}");
    }

    #[test]
    fn test_lock_envelope_tolerates_extra_fields() {
        let lock = default_lock();
        let envelope_json = format!(
            r#"{{"header":"Teaser","title":"Locked Article","header_kind":"image","lock":"{lock}","_forward_compat_canary":"future-only"}}"#
        );
        let post = make_lock_post_from_content(&envelope_json);
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_lock_post_envelope_at_max_size() {
        let max_header = "a".repeat(VALIDATION_LIMITS.lock_content_header_max_length);
        let max_title = "b".repeat(VALIDATION_LIMITS.lock_content_title_max_length);
        let content =
            lock_envelope_json(&max_header, &max_title, PubkyAppPostKind::Short, &default_lock());
        let post = make_lock_post_from_content(&content);
        assert!(
            post.content.chars().count() < VALIDATION_LIMITS.lock_content_max_length,
            "envelope at max field sizes must fit under lock_content_max_length"
        );
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn test_create_lock_post_wasm_builder() {
        // End-to-end via the JS-facing builder:
        //   PubkySpecsBuilder.createLockPost(header, title, header_kind, lock)
        // builds the {header, title, header_kind, lock} envelope internally,
        // packages it into a kind=Lock PubkyAppPost, and returns a PostResult.
        use crate::PubkySpecsBuilder;
        let pubky_id = TEST_PUBKY_ID.to_string();
        let builder = PubkySpecsBuilder::new(pubky_id).expect("Failed to construct builder");
        let result = builder
            .create_lock_post(
                "We were reckless adopting Lightning.".to_string(),
                "Locked Article".to_string(),
                PubkyAppPostKind::Image,
                default_lock(),
            )
            .expect("createLockPost should succeed");

        let post = result.post();
        assert_eq!(post.kind, PubkyAppPostKind::Lock);
        assert!(post.attachments.is_none());
        let envelope: PubkyAppLockContent = serde_json::from_str(&post.content)
            .expect("Lock content must deserialize as PubkyAppLockContent");
        assert_eq!(envelope.title, "Locked Article");
        assert_eq!(envelope.header_kind, PubkyAppPostKind::Image);
        assert_eq!(envelope.lock, default_lock());
    }
}
