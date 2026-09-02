use crate::common::{code_point_len, frozen_trim};
use crate::limits::VALIDATION_LIMITS;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use super::super::{check_target_reference, PubkySocialPost};

/// Typed JSON envelope stored in `PubkySocialPost::content` when `kind == Article`.
///
/// Parsed and validated by the spec, never stored as a top-level object. Build one through
/// `PubkySocialPost::create_article_post`; the struct is public so consumers can inspect the
/// shape. No `deny_unknown_fields`: later minors may add members, and `extra` keeps them.
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[serde(rename_all = "snake_case")]
pub struct PubkySocialArticleContent {
    /// Trimmed by the builder; `[1, article_title_max_length]` code points.
    pub title: String,
    /// Markdown; at most `article_body_max_length` code points.
    pub body: String,
    /// Optional cover, a canonical pubky or web URI of at most `image_url_max_length`
    /// code points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_image: Option<String>,
    /// Members this version does not know, preserved member-for-member on rewrite so an
    /// older client never drops a newer client's data. Opaque: never validated, never
    /// written by builders. Deliberate extensions live under the reserved `ext` member and
    /// are hostile input until the extension's own rules have checked them.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Validates the envelope of a `kind = Article` post. The post-level rules (references,
/// attachments) run before this in `PubkySocialPost::validate`.
pub(crate) fn validate_article_post(post: &PubkySocialPost) -> Result<(), String> {
    if code_point_len(&post.content) > VALIDATION_LIMITS.article_content_max_length {
        return Err(format!(
            "Validation Error: Article content exceeds max length {}",
            VALIDATION_LIMITS.article_content_max_length
        ));
    }
    let envelope: PubkySocialArticleContent = serde_json::from_str(&post.content).map_err(|e| {
        format!("Validation Error: Article content must be a valid JSON envelope: {e}")
    })?;
    if frozen_trim(&envelope.title).is_empty() {
        return Err(
            "Validation Error: Article title must contain non-whitespace characters".into(),
        );
    }
    // The untrimmed length, so a padded title cannot slip past the cap
    if code_point_len(&envelope.title) > VALIDATION_LIMITS.article_title_max_length {
        return Err(format!(
            "Validation Error: Article title must be at most {} characters",
            VALIDATION_LIMITS.article_title_max_length
        ));
    }
    if code_point_len(&envelope.body) > VALIDATION_LIMITS.article_body_max_length {
        return Err(format!(
            "Validation Error: Article body must be at most {} characters",
            VALIDATION_LIMITS.article_body_max_length
        ));
    }
    if let Some(cover) = &envelope.cover_image {
        check_target_reference("cover_image", cover)?;
        if code_point_len(cover) > VALIDATION_LIMITS.image_url_max_length {
            return Err(format!(
                "Validation Error: cover_image must be at most {} characters",
                VALIDATION_LIMITS.image_url_max_length
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{TimestampId, Validatable, PUB_CTX};
    use crate::{PubkySocialAttachment, PubkySocialPostKind};

    const PK: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";

    fn p(path: &str) -> String {
        format!("pubky://{PK}{path}")
    }

    fn article(title: &str, body: &str, cover: Option<&str>) -> PubkySocialPost {
        PubkySocialPost::create_article_post(
            title.to_string(),
            body.to_string(),
            cover.map(str::to_string),
            None,
            None,
            vec![],
        )
    }

    fn validate(post: &PubkySocialPost) -> Result<(), String> {
        let id = post.create_id();
        post.validate(Some(&id), &PUB_CTX)
    }

    fn err(post: &PubkySocialPost) -> String {
        validate(post).expect_err("expected a validation error")
    }

    #[test]
    fn builder_round_trips_and_trims_the_title() {
        let post = article("\u{3000} Hello \u{3000}", "# body", None);
        assert_eq!(post.kind, PubkySocialPostKind::Article);
        assert!(validate(&post).is_ok());
        let envelope: PubkySocialArticleContent = serde_json::from_str(&post.content).unwrap();
        assert_eq!(envelope.title, "Hello");
        assert_eq!(envelope.body, "# body");
        assert!(envelope.cover_image.is_none());
        assert_eq!(post.content, r##"{"title":"Hello","body":"# body"}"##);
    }

    #[test]
    fn title_rules() {
        assert!(err(&article("   ", "b", None)).contains("title"));
        let max = VALIDATION_LIMITS.article_title_max_length;
        assert!(validate(&article(&"\u{1F600}".repeat(max), "b", None)).is_ok());
        assert!(err(&article(&"a".repeat(max + 1), "b", None)).contains("title"));
        // A padded title inside a hand-written envelope counts untrimmed
        let padded = format!(r#"{{"title":"{} ","body":"b"}}"#, "a".repeat(max));
        let post = PubkySocialPost::new(padded, PubkySocialPostKind::Article, None, None, vec![]);
        assert!(err(&post).contains("title"));
    }

    #[test]
    fn body_rules() {
        let max = VALIDATION_LIMITS.article_body_max_length;
        assert!(validate(&article("t", &"b".repeat(max), None)).is_ok());
        assert!(err(&article("t", &"b".repeat(max + 1), None)).contains("body"));
    }

    #[test]
    fn raw_cap_is_checked_before_parsing() {
        let raw = "x".repeat(VALIDATION_LIMITS.article_content_max_length + 1);
        let post = PubkySocialPost::new(raw, PubkySocialPostKind::Article, None, None, vec![]);
        let e = err(&post);
        assert!(e.contains("exceeds max length"), "{e}");
        assert!(!e.contains("JSON"), "{e}");
    }

    #[test]
    fn malformed_envelope_rejects() {
        let post = PubkySocialPost::new(
            "not json".into(),
            PubkySocialPostKind::Article,
            None,
            None,
            vec![],
        );
        assert!(err(&post).contains("JSON envelope"));
        let post = PubkySocialPost::new(
            r#"{"title":"t"}"#.into(),
            PubkySocialPostKind::Article,
            None,
            None,
            vec![],
        );
        assert!(err(&post).contains("JSON envelope"));
    }

    #[test]
    fn cover_image_rules() {
        let file = p("/pub/social/v1/files/0034A0X7NJ52G");
        assert!(validate(&article("t", "b", Some(&file))).is_ok());
        assert!(validate(&article("t", "b", Some("https://example.com/c.png"))).is_ok());
        for bad in ["ftp://x/c.png", " https://example.com/c.png", ""] {
            assert!(
                err(&article("t", "b", Some(bad))).contains("cover_image"),
                "{bad}"
            );
        }
        let max = VALIDATION_LIMITS.image_url_max_length;
        let long = format!("https://e.com/{}", "a".repeat(max - 14));
        assert_eq!(code_point_len(&long), max);
        assert!(validate(&article("t", "b", Some(&long))).is_ok());
        assert!(err(&article("t", "b", Some(&format!("{long}a")))).contains("cover_image"));
    }

    #[test]
    fn article_may_reply_quote_and_attach() {
        let post = PubkySocialPost::create_article_post(
            "t".into(),
            "b".into(),
            None,
            Some(p("/pub/social/v1/posts/0032SSN7Q4EVG")),
            Some("https://example.com/source".into()),
            vec![PubkySocialAttachment::new(
                p("/pub/social/v1/files/0034A0X7NJ52G"),
                None,
                None,
            )],
        );
        assert!(validate(&post).is_ok());
    }

    #[test]
    fn attachment_rules_still_apply_to_articles() {
        let many = (0..=VALIDATION_LIMITS.post_attachments_max_count)
            .map(|_| {
                PubkySocialAttachment::new(p("/pub/social/v1/files/0034A0X7NJ52G"), None, None)
            })
            .collect();
        let post =
            PubkySocialPost::create_article_post("t".into(), "b".into(), None, None, None, many);
        assert!(err(&post).contains("Too many attachments"));
    }

    #[test]
    fn unknown_members_survive_rewrite() {
        let envelope: PubkySocialArticleContent =
            serde_json::from_str(r#"{"title":"t","body":"b","ext":{"toc":true}}"#).unwrap();
        let out: serde_json::Value = serde_json::to_value(&envelope).unwrap();
        assert_eq!(out["ext"]["toc"], true);
        assert_eq!(out["title"], "t");
    }
}
