use crate::{common::validate_crockford_id, limits::VALIDATION_LIMITS, types::PubkyId};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use url::Url;

#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use super::super::PubkyAppPost;

/// Creator-chosen default layout for experiencing a collection.
///
/// Unrecognized values deserialize as `Unknown` so future layouts never
/// invalidate the whole post (same policy as `PubkyAppPostKind`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[serde(rename_all = "snake_case")]
pub enum PubkyAppCollectionLayout {
    Grid,
    List,
    Visual,
    #[serde(other)]
    Unknown,
}

impl FromStr for PubkyAppCollectionLayout {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "grid" => Ok(Self::Grid),
            "list" => Ok(Self::List),
            "visual" => Ok(Self::Visual),
            _ => Err(format!("Invalid collection layout: {}", s)),
        }
    }
}

/// Typed JSON envelope stored in `PubkyAppPost::content` when `kind == Collection`.
///
/// A collection post curates an ordered list of URIs (via `items`)
/// under a `name` and optional `description`. The envelope is parsed and validated
/// by the spec but never re-serialized as a top-level homeserver object.
///
/// **Construction**: this struct is deserialized from the post's `content` JSON
/// envelope during validation and is **not** intended to be constructed by
/// callers directly. It is re-exported publicly (via `lib.rs`) so SDK consumers
/// can inspect the envelope shape (OpenAPI schema, type definitions), but the
/// authoritative way to produce a Collection post is to author a `PubkyAppPost`
/// with `kind: Collection` and a `content` string that JSON-parses into this
/// shape.
///
/// Forward-compat: `#[serde(deny_unknown_fields)]` is intentionally NOT used so
/// future minor versions can add fields (e.g. `cover_image`) without breaking
/// older parsers. New fields must be additive and ignorable.
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[serde(rename_all = "snake_case")]
pub struct PubkyAppCollectionContent {
    /// Display name of the collection. Length bounded by
    /// `VALIDATION_LIMITS.collection_name_{min,max}_length` (unicode scalars).
    /// Whitespace-only names are rejected separately by the validator.
    pub name: String,
    /// Optional human-readable description. Length bounded by
    /// `VALIDATION_LIMITS.collection_description_max_length` (unicode scalars).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Ordered list of Post URIs this collection curates. Count bounded by
    /// `VALIDATION_LIMITS.collection_items_max_count`; each URI must be in
    /// exact canonical form (see `validate_collection_item_uri`).
    #[serde(default)]
    pub items: Vec<String>,
    /// Optional hero/cover image URL. Length bounded by
    /// `VALIDATION_LIMITS.post_attachment_url_max_length`; protocol must be in
    /// `VALIDATION_LIMITS.post_allowed_attachment_protocols`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_image: Option<String>,
    /// Creator's preferred default layout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<PubkyAppCollectionLayout>,
}

/// Validates a `kind = Collection` post, including its JSON content envelope.
pub(crate) fn validate_collection_post(post: &PubkyAppPost) -> Result<(), String> {
    if post.parent.is_some() || post.embed.is_some() {
        return Err("Validation Error: Collection posts cannot have parent or embed".into());
    }
    // Anti-misuse guard: items belong in the envelope, not in `post.attachments`.
    if post.attachments.is_some() {
        return Err(
            "Validation Error: Collection posts must not use post.attachments — items belong in the content envelope"
                .into(),
        );
    }
    if post.content.chars().count() > VALIDATION_LIMITS.collection_content_max_length {
        return Err(format!(
            "Validation Error: Collection content exceeds max length {}",
            VALIDATION_LIMITS.collection_content_max_length
        ));
    }
    let envelope: PubkyAppCollectionContent = serde_json::from_str(&post.content).map_err(|e| {
        format!(
            "Validation Error: Collection content must be a valid JSON envelope: {}",
            e
        )
    })?;
    validate_collection_envelope(&envelope)
}

fn validate_collection_envelope(envelope: &PubkyAppCollectionContent) -> Result<(), String> {
    if envelope.name.trim().is_empty() {
        return Err(
            "Validation Error: Collection name must contain non-whitespace characters".into(),
        );
    }
    let name_chars = envelope.name.chars().count();
    let name_min = VALIDATION_LIMITS.collection_name_min_length;
    let name_max = VALIDATION_LIMITS.collection_name_max_length;
    if !(name_min..=name_max).contains(&name_chars) {
        return Err(format!(
            "Validation Error: Collection name must be {}..={} characters",
            name_min, name_max
        ));
    }
    if let Some(desc) = &envelope.description {
        if desc.chars().count() > VALIDATION_LIMITS.collection_description_max_length {
            return Err(format!(
                "Validation Error: Collection description exceeds {} characters",
                VALIDATION_LIMITS.collection_description_max_length
            ));
        }
    }
    if let Some(cover) = &envelope.cover_image {
        if cover.chars().count() > VALIDATION_LIMITS.post_attachment_url_max_length {
            return Err(format!(
                "Validation Error: Collection cover_image URL exceeds {} characters",
                VALIDATION_LIMITS.post_attachment_url_max_length
            ));
        }
        let parsed = Url::parse(cover).map_err(|_| {
            "Validation Error: Collection cover_image must be a valid URL".to_string()
        })?;
        if !VALIDATION_LIMITS
            .post_allowed_attachment_protocols
            .iter()
            .any(|&protocol| parsed.scheme().eq_ignore_ascii_case(protocol))
        {
            let allowed = VALIDATION_LIMITS
                .post_allowed_attachment_protocols
                .iter()
                .map(|p| format!("{p}://"))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!(
                "Validation Error: Collection cover_image must use one of the allowed protocols: {allowed}"
            ));
        }
    }
    if envelope.items.len() > VALIDATION_LIMITS.collection_items_max_count {
        return Err(format!(
            "Validation Error: Collection cannot have more than {} items",
            VALIDATION_LIMITS.collection_items_max_count
        ));
    }
    for (index, uri) in envelope.items.iter().enumerate() {
        validate_collection_item_uri(uri)
            .map_err(|e| format!("Validation Error: Collection item at index {index}: {e}"))?;
    }
    Ok(())
}

/// Strict canonical post-URI check for Collection items. Accepts only the
/// exact form `pubky://<pubky-id>/pub/pubky.app/posts/<post-id>`.
///
/// Deliberately avoids `Url::parse`: it silently strips userinfo and collapses
/// `..` path segments, smuggling non-canonical strings past a parse-and-recheck
/// approach. Splitting the raw string and delegating to `PubkyId::try_from`
/// (52-char z-base-32) and `validate_crockford_id` (13-char Crockford) enforces
/// the canonical 94-char form structurally.
fn validate_collection_item_uri(uri: &str) -> Result<(), String> {
    const PREFIX: &str = "pubky://";
    const MIDDLE: &str = "/pub/pubky.app/posts/";
    let rest = uri
        .strip_prefix(PREFIX)
        .ok_or_else(|| format!("must start with pubky://: {uri}"))?;
    let (host, post_id) = rest
        .split_once(MIDDLE)
        .ok_or_else(|| format!("must be a canonical post URI: {uri}"))?;
    PubkyId::try_from(host).map_err(|e| format!("invalid pubky-id in host: {e}"))?;
    validate_crockford_id(post_id).map_err(|e| format!("invalid post id: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::super::{PubkyAppPost, PubkyAppPostEmbed, PubkyAppPostKind};
    use super::*;
    use crate::traits::{TimestampId, Validatable};

    const TEST_PUBKY_ID: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";

    fn collection_envelope_json(name: &str, description: Option<&str>, items: &[String]) -> String {
        serde_json::to_string(&PubkyAppCollectionContent {
            name: name.to_string(),
            description: description.map(|d| d.to_string()),
            items: items.to_vec(),
            ..Default::default()
        })
        .unwrap()
    }

    fn make_collection_post(
        name: &str,
        description: Option<&str>,
        items: Option<Vec<String>>,
    ) -> PubkyAppPost {
        let items = items.unwrap_or_default();
        PubkyAppPost::new(
            collection_envelope_json(name, description, &items),
            PubkyAppPostKind::Collection,
            None,
            None,
            None,
        )
    }

    fn make_collection_post_with_cover(cover_image: Option<&str>) -> PubkyAppPost {
        let envelope = PubkyAppCollectionContent {
            name: "X".to_string(),
            cover_image: cover_image.map(|s| s.to_string()),
            ..Default::default()
        };
        let content = serde_json::to_string(&envelope).expect("envelope serialization");
        PubkyAppPost::new(content, PubkyAppPostKind::Collection, None, None, None)
    }

    #[test]
    fn test_collection_post_roundtrip_valid() {
        let post = make_collection_post(
            "AI papers",
            Some("Best stuff"),
            Some(vec![
                format!("pubky://{TEST_PUBKY_ID}/pub/pubky.app/posts/0034A0X7NJ52A"),
                format!("pubky://{TEST_PUBKY_ID}/pub/pubky.app/posts/0034A0X7NJ52B"),
                format!("pubky://{TEST_PUBKY_ID}/pub/pubky.app/posts/0034A0X7NJ52C"),
            ]),
        );
        let id = post.create_id();
        let blob = serde_json::to_vec(&post).unwrap();
        let parsed = <PubkyAppPost as Validatable>::try_from(&blob, &id).unwrap();
        assert_eq!(parsed.kind, PubkyAppPostKind::Collection);
        assert!(parsed.attachments.is_none());
        let envelope: PubkyAppCollectionContent = serde_json::from_str(&parsed.content).unwrap();
        assert_eq!(envelope.items.len(), 3);
    }

    #[test]
    fn test_collection_post_rejects_malformed_envelope() {
        let post = PubkyAppPost::new(
            "this is not JSON".to_string(),
            PubkyAppPostKind::Collection,
            None,
            None,
            None,
        );
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("JSON envelope"),
            "expected JSON envelope error, got: {}",
            err
        );
    }

    #[test]
    fn test_collection_post_rejects_empty_name() {
        let post = make_collection_post("", None, None);
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name"));
    }

    #[test]
    fn test_collection_post_rejects_oversized_name() {
        // 101 grapheme-ish chars; mix in emoji to confirm we count by unicode scalars, not bytes.
        let oversized = "a".repeat(99) + "🚀🚀";
        assert_eq!(oversized.chars().count(), 101);
        let post = make_collection_post(&oversized, None, None);
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name"));
    }

    #[test]
    fn test_collection_post_accepts_max_name() {
        let exactly_100 = "a".repeat(100);
        assert_eq!(exactly_100.chars().count(), 100);
        let post = make_collection_post(&exactly_100, None, None);
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_collection_post_rejects_whitespace_only_name() {
        // Whitespace-only names pass `min_length=1` purely by char count, so
        // we reject them with a dedicated guard. Without that guard, a name
        // of `"    "` would be a 4-char valid name with no meaningful content.
        let post = make_collection_post("    ", None, None);
        let id = post.create_id();
        let err = post
            .validate(Some(&id))
            .expect_err("whitespace-only name must fail validation");
        assert!(
            err.contains("whitespace"),
            "error should mention whitespace, got: {err}"
        );
    }

    #[test]
    fn test_collection_post_counts_whitespace_in_name_length() {
        // Regression guard: the validator does NOT trim before counting. A
        // 99-char name padded with one space on each side is 101 chars and
        // must fail max=100. With the previous trim-then-count behavior this
        // would have been 99 chars and passed.
        let padded = format!(" {} ", "a".repeat(99));
        assert_eq!(padded.chars().count(), 101);
        let post = make_collection_post(&padded, None, None);
        let id = post.create_id();
        let err = post
            .validate(Some(&id))
            .expect_err("101-char padded name must fail max length");
        assert!(
            err.contains("1..=100"),
            "error should report the length range, got: {err}"
        );
    }

    #[test]
    fn test_collection_post_accepts_cover_image_pubky_uri() {
        let cover = format!("pubky://{TEST_PUBKY_ID}/pub/pubky.app/files/0034A0X7NJ52A");
        let post = make_collection_post_with_cover(Some(&cover));
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_collection_post_accepts_cover_image_https() {
        let post = make_collection_post_with_cover(Some("https://example.com/cover.png"));
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_collection_post_rejects_cover_image_invalid_url() {
        let post = make_collection_post_with_cover(Some("not a url"));
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(
            err.contains("cover_image must be a valid URL"),
            "got: {err}"
        );
    }

    #[test]
    fn test_collection_post_rejects_cover_image_disallowed_protocol() {
        let post = make_collection_post_with_cover(Some("ftp://example.com/cover.png"));
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(
            err.contains("cover_image must use one of the allowed protocols"),
            "got: {err}"
        );
    }

    #[test]
    fn test_collection_post_rejects_cover_image_too_long() {
        // post_attachment_url_max_length is 200; this URL exceeds it.
        let too_long = format!("https://example.com/{}", "a".repeat(200));
        let post = make_collection_post_with_cover(Some(&too_long));
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("cover_image URL exceeds"), "got: {err}");
    }

    #[test]
    fn test_collection_post_rejects_oversized_description() {
        let too_long = "a".repeat(501);
        let post = make_collection_post("X", Some(&too_long), None);
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("description"));
    }

    #[test]
    fn test_collection_post_accepts_empty_description() {
        // Explicit empty-string description is valid (the field is optional and
        // 0..=500 chars allowed).
        let post = make_collection_post("X", Some(""), None);
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_collection_post_accepts_max_description() {
        let exactly_500 = "a".repeat(500);
        assert_eq!(exactly_500.chars().count(), 500);
        let post = make_collection_post("X", Some(&exactly_500), None);
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_collection_post_rejects_missing_name() {
        // Envelope JSON without a `name` field at all (description-only).
        // Distinct from `test_collection_post_rejects_empty_name`, which sends
        // an empty string; this sends a missing key entirely.
        let envelope = r#"{ "description": "no name here" }"#.to_string();
        let post = PubkyAppPost::new(envelope, PubkyAppPostKind::Collection, None, None, None);
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("name") || err.to_lowercase().contains("missing"),
            "expected name-required error, got: {err}"
        );
    }

    #[test]
    fn test_collection_post_rejects_parent() {
        let post = PubkyAppPost::new(
            collection_envelope_json("X", None, &[]),
            PubkyAppPostKind::Collection,
            Some("pubky://userA/pub/pubky.app/posts/0034A0X7NJ52A".to_string()),
            None,
            None,
        );
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("parent or embed"),
            "expected parent-or-embed error, got: {}",
            err
        );
    }

    #[test]
    fn test_collection_post_rejects_embed() {
        let post = PubkyAppPost::new(
            collection_envelope_json("X", None, &[]),
            PubkyAppPostKind::Collection,
            None,
            Some(PubkyAppPostEmbed {
                kind: PubkyAppPostKind::Short,
                uri: "pubky://userA/pub/pubky.app/posts/0034A0X7NJ52A".to_string(),
            }),
            None,
        );
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("parent or embed"));
    }

    #[test]
    fn test_collection_post_accepts_100_items() {
        let items: Vec<String> = (0..100)
            .map(|i| format!("pubky://{TEST_PUBKY_ID}/pub/pubky.app/posts/{:013}", i))
            .collect();
        let post = make_collection_post("Big list", None, Some(items));
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_collection_post_rejects_101_items() {
        let items: Vec<String> = (0..101)
            .map(|i| format!("pubky://userA/pub/pubky.app/posts/{:013}", i))
            .collect();
        let post = make_collection_post("Too big", None, Some(items));
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("100 items"));
    }
    #[test]
    fn test_collection_post_accepts_zero_items() {
        // Curators may create a draft and add items later via edits.
        let post = make_collection_post("Drafts", None, None);
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_collection_post_roundtrip_layout() {
        let envelope_json = r#"{"name":"Photos","layout":"visual"}"#;
        let post = PubkyAppPost::new(
            envelope_json.to_string(),
            PubkyAppPostKind::Collection,
            None,
            None,
            None,
        );
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
        let envelope: PubkyAppCollectionContent = serde_json::from_str(&post.content).unwrap();
        assert_eq!(envelope.layout, Some(PubkyAppCollectionLayout::Visual));
    }

    #[test]
    fn test_collection_post_unknown_layout_tolerated() {
        // Forward-compat: a layout variant from a future spec version must not
        // invalidate the whole post; it degrades to Unknown.
        let envelope_json = r#"{"name":"X","layout":"spiral"}"#;
        let post = PubkyAppPost::new(
            envelope_json.to_string(),
            PubkyAppPostKind::Collection,
            None,
            None,
            None,
        );
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
        let envelope: PubkyAppCollectionContent = serde_json::from_str(&post.content).unwrap();
        assert_eq!(envelope.layout, Some(PubkyAppCollectionLayout::Unknown));
    }

    #[test]
    fn test_collection_envelope_tolerates_extra_fields() {
        // Forward-compat: the envelope intentionally does NOT use deny_unknown_fields,
        // so future minor versions can add fields without breaking older parsers.
        // Use a deliberately-fictional canary field name so this test stays
        // meaningful even after real fields land.
        let envelope_json = r#"{"name":"X","_forward_compat_canary":"future-only"}"#;
        let post = PubkyAppPost::new(
            envelope_json.to_string(),
            PubkyAppPostKind::Collection,
            None,
            None,
            None,
        );
        let id = post.create_id();
        assert!(
            post.validate(Some(&id)).is_ok(),
            "unknown envelope fields must be tolerated"
        );
    }

    #[test]
    fn test_collection_post_rejects_non_post_uri() {
        let post =
            make_collection_post("X", None, Some(vec!["ftp://example.com/file".to_string()]));
        let id = post.create_id();
        let result = post.validate(Some(&id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Collection item"));
    }

    #[test]
    fn test_collection_post_rejects_post_uri_with_invalid_post_id() {
        // 13 chars but not valid Crockford: contains hyphens which aren't in the alphabet.
        let uri = format!("pubky://{TEST_PUBKY_ID}/pub/pubky.app/posts/abc-def-ghi-j");
        let post = make_collection_post("X", None, Some(vec![uri]));
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("invalid post id"), "got: {err}");
    }

    #[test]
    fn test_collection_post_rejects_post_uri_with_extra_path_segment() {
        // Extra segment lands inside the post-id slot, failing the 13-char
        // Crockford check.
        let uri = format!("pubky://{TEST_PUBKY_ID}/pub/pubky.app/posts/0034A0X7NJ52A/extra");
        let post = make_collection_post("X", None, Some(vec![uri]));
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("invalid post id"), "got: {err}");
    }

    #[test]
    fn test_collection_post_rejects_post_uri_with_query_string() {
        // Query string lands inside the post-id slot and fails the Crockford
        // check (`?` and `=` aren't in the alphabet, and the length is wrong).
        let uri = format!("pubky://{TEST_PUBKY_ID}/pub/pubky.app/posts/0034A0X7NJ52A?foo=bar");
        let post = make_collection_post("X", None, Some(vec![uri]));
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("invalid post id"), "got: {err}");
    }

    #[test]
    fn test_collection_post_rejects_post_uri_with_fragment() {
        // Same as the query-string case: `#` and the fragment body land in the
        // post-id slot and fail Crockford.
        let uri = format!("pubky://{TEST_PUBKY_ID}/pub/pubky.app/posts/0034A0X7NJ52A#frag");
        let post = make_collection_post("X", None, Some(vec![uri]));
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("invalid post id"), "got: {err}");
    }

    #[test]
    fn test_collection_post_rejects_post_uri_with_trailing_slash() {
        // A trailing slash bloats the post-id past 13 chars.
        let uri = format!("pubky://{TEST_PUBKY_ID}/pub/pubky.app/posts/0034A0X7NJ52A/");
        let post = make_collection_post("X", None, Some(vec![uri]));
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("invalid post id"), "got: {err}");
    }

    #[test]
    fn test_collection_post_rejects_post_uri_with_empty_post_id() {
        // Empty post-id segment fails the 13-char Crockford check.
        let uri = format!("pubky://{TEST_PUBKY_ID}/pub/pubky.app/posts/");
        let post = make_collection_post("X", None, Some(vec![uri]));
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("invalid post id"), "got: {err}");
    }

    #[test]
    fn test_collection_post_rejects_userinfo_padding_bypass() {
        // `Url::parse(...).host_str()` strips userinfo, which would smuggle
        // arbitrary bytes past a parse-and-recheck approach. The strict
        // validator keeps the `JUNK@` in the host slot, failing the 52-char
        // PubkyId length check.
        let uri = format!(
            "pubky://AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA@{TEST_PUBKY_ID}/pub/pubky.app/posts/0034A0X7NJ52A"
        );
        let post = make_collection_post("X", None, Some(vec![uri]));
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("invalid pubky-id"), "got: {err}");
    }

    #[test]
    fn test_collection_post_rejects_dot_dot_path_bypass() {
        // `Url::parse(...)` collapses `..` segments before path inspection,
        // which would smuggle a non-canonical raw path past a parse-and-recheck
        // approach. The strict validator splits the raw string, so the extra
        // segments land in the host slot and fail PubkyId.
        let uri = format!("pubky://{TEST_PUBKY_ID}/aa/bb/../../pub/pubky.app/posts/0034A0X7NJ52A");
        let post = make_collection_post("X", None, Some(vec![uri]));
        let id = post.create_id();
        let err = post.validate(Some(&id)).unwrap_err();
        assert!(err.contains("invalid pubky-id"), "got: {err}");
    }

    #[test]
    fn test_collection_post_accepts_canonical_max_length_uri() {
        // Success-side boundary: the longest valid canonical post URI is
        // pubky://<52-char-pubky-id>/pub/pubky.app/posts/<13-char-crockford>
        // which is exactly 94 chars. Validator must accept this.
        let uri = format!("pubky://{TEST_PUBKY_ID}/pub/pubky.app/posts/0034A0X7NJ52A");
        assert_eq!(uri.chars().count(), 94);
        let post = make_collection_post("X", None, Some(vec![uri]));
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_collection_post_rejects_non_empty_attachments() {
        let post = PubkyAppPost::new(
            collection_envelope_json("X", None, &[]),
            PubkyAppPostKind::Collection,
            None,
            None,
            Some(vec![
                "pubky://userA/pub/pubky.app/posts/0034A0X7NJ52A".to_string()
            ]),
        );
        let id = post.create_id();
        let err = post
            .validate(Some(&id))
            .expect_err("Collection with non-empty post.attachments must be rejected");
        assert!(
            err.contains("post.attachments"),
            "expected anti-misuse error, got: {err}"
        );
    }

    #[test]
    fn test_collection_post_rejects_empty_attachments() {
        let post = PubkyAppPost::new(
            collection_envelope_json("X", None, &[]),
            PubkyAppPostKind::Collection,
            None,
            None,
            Some(vec![]),
        );
        let id = post.create_id();
        let err = post
            .validate(Some(&id))
            .expect_err("Collection with empty post.attachments must be rejected");
        assert!(
            err.contains("post.attachments"),
            "expected anti-misuse error, got: {err}"
        );
    }

    #[test]
    fn test_collection_post_accepts_missing_items_field() {
        let envelope_json = r#"{"name":"X"}"#;
        let post = PubkyAppPost::new(
            envelope_json.to_string(),
            PubkyAppPostKind::Collection,
            None,
            None,
            None,
        );
        let id = post.create_id();
        assert!(
            post.validate(Some(&id)).is_ok(),
            "missing `items` field must deserialize as empty list via serde(default)"
        );
    }

    #[test]
    fn test_collection_post_envelope_at_max_size() {
        // 100 distinct valid pubky post URIs (max-count). Each exactly 94 chars.
        let items: Vec<String> = (0..VALIDATION_LIMITS.collection_items_max_count)
            .map(|i| format!("pubky://{TEST_PUBKY_ID}/pub/pubky.app/posts/{:013}", i))
            .collect();
        let max_name = "a".repeat(VALIDATION_LIMITS.collection_name_max_length);
        let max_desc = "b".repeat(VALIDATION_LIMITS.collection_description_max_length);
        let post = make_collection_post(&max_name, Some(&max_desc), Some(items));
        assert!(
            post.content.chars().count() < VALIDATION_LIMITS.collection_content_max_length,
            "envelope at max field sizes must fit under collection_content_max_length"
        );
        let id = post.create_id();
        assert!(post.validate(Some(&id)).is_ok());
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn test_postkind_collection_wasm_getter() {
        let post = PubkyAppPost {
            content: collection_envelope_json("X", None, &[]),
            kind: PubkyAppPostKind::Collection,
            parent: None,
            embed: None,
            attachments: None,
            lock: None,
        };
        assert_eq!(post.kind(), "Collection");
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn test_create_collection_post_wasm_builder() {
        // End-to-end via the JS-facing builder:
        //   PubkySpecsBuilder.createCollectionPost(name, description?, items?, cover_image?, layout?)
        // builds the {name, description} envelope internally, packages it
        // into a kind=Collection PubkyAppPost, and returns a PostResult
        // ready to ship to the homeserver. JS callers don't have to
        // JSON-stringify the envelope themselves.
        use crate::PubkySpecsBuilder;
        let pubky_id = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".to_string();
        let builder = PubkySpecsBuilder::new(pubky_id).expect("Failed to construct builder");
        let result = builder
            .create_collection_post(
                "My favorites".to_string(),
                Some("Best things".to_string()),
                Some(vec![
                    "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/posts/0034A0X7NJ52A".to_string(),
                ]),
                Some("https://example.com/cover.png".to_string()),
                Some("list".to_string()),
            )
            .expect("createCollectionPost should succeed");

        let post = result.post();
        assert_eq!(post.kind, PubkyAppPostKind::Collection);
        assert!(post.attachments.is_none());
        let envelope: PubkyAppCollectionContent = serde_json::from_str(&post.content)
            .expect("Collection content must deserialize as PubkyAppCollectionContent");
        assert_eq!(envelope.name, "My favorites");
        assert_eq!(envelope.description.as_deref(), Some("Best things"));
        assert_eq!(envelope.items.len(), 1);
        assert_eq!(
            envelope.cover_image.as_deref(),
            Some("https://example.com/cover.png")
        );
        assert_eq!(envelope.layout, Some(PubkyAppCollectionLayout::List));
    }
}
