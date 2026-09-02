use crate::canonicalize::{check_pubky_reference, check_target_reference};
use crate::common::{check_extra_keys, code_point_len, frozen_trim};
use crate::constants::social_path;
use crate::limits::VALIDATION_LIMITS;
use crate::traits::{HasIdPath, Root, TimestampId, Validatable, ValidationCtx, ValidationError};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

pub mod content;

pub use content::{
    PubkySocialArticleContent, PubkySocialCollectionContent, PubkySocialCollectionLayout,
};

#[cfg(target_arch = "wasm32")]
use crate::traits::Json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Represents the type of pubky-app posted data
/// Used primarily to best display the content in UI
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[non_exhaustive]
pub enum PubkySocialPostKind {
    #[default]
    Note,
    Article,
    Image,
    Video,
    Link,
    File,
    Collection,
    #[serde(other)]
    Unknown,
}

impl fmt::Display for PubkySocialPostKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string_repr = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        write!(f, "{}", string_repr)
    }
}

impl FromStr for PubkySocialPostKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "note" => Ok(PubkySocialPostKind::Note),
            "article" => Ok(PubkySocialPostKind::Article),
            "image" => Ok(PubkySocialPostKind::Image),
            "video" => Ok(PubkySocialPostKind::Video),
            "link" => Ok(PubkySocialPostKind::Link),
            "file" => Ok(PubkySocialPostKind::File),
            "collection" => Ok(PubkySocialPostKind::Collection),
            _ => Err(format!("Invalid content kind: {}", s)),
        }
    }
}

impl PubkySocialPostKind {
    /// Returns `true` for every spec-recognized variant, `false` for `Unknown`.
    ///
    /// `Unknown` is the forwards-compat catch-all variant (via `#[serde(other)]`)
    /// that captures any post-kind string this version of the spec doesn't
    /// recognize yet. Most consumers, indexers, stream filters, search ranking,
    /// want to skip such posts, and this helper lets them write
    /// `if kind.is_known() { ... }` rather than
    /// `if !matches!(kind, PubkySocialPostKind::Unknown) { ... }`.
    pub fn is_known(&self) -> bool {
        !matches!(self, PubkySocialPostKind::Unknown)
    }

    #[cfg(target_arch = "wasm32")]
    fn wasm_name(&self) -> &'static str {
        match self {
            PubkySocialPostKind::Note => "Note",
            PubkySocialPostKind::Article => "Article",
            PubkySocialPostKind::Image => "Image",
            PubkySocialPostKind::Video => "Video",
            PubkySocialPostKind::Link => "Link",
            PubkySocialPostKind::File => "File",
            PubkySocialPostKind::Collection => "Collection",
            PubkySocialPostKind::Unknown => "Unknown",
        }
    }
}

/// One attached media reference. An object rather than a string so per-item metadata can
/// grow without a break. `name` is per reference: two posts may attach the same bytes under
/// different names.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkySocialAttachment {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub uri: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Unknown members, preserved on rewrite; see the module contract in `models/mod.rs`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkySocialAttachment {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(uri: String, alt: Option<String>, name: Option<String>) -> Self {
        PubkySocialAttachment {
            uri,
            alt,
            name,
            extra: Default::default(),
        }
        .sanitize()
    }
}

impl PubkySocialAttachment {
    /// Trim is the only documented canonicalization; the uri passes through verbatim
    fn sanitize(self) -> Self {
        PubkySocialAttachment {
            name: self.name.map(|n| frozen_trim(&n).to_string()),
            ..self
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl PubkySocialAttachment {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn uri(&self) -> String {
        self.uri.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn alt(&self) -> Option<String> {
        self.alt.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }
}

/// Represents raw post in homeserver with content and kind
/// URI: /pub/social/v1/posts/:post_id/:edit_id.json
/// Where both ids are CrockfordBase32 encodings of a timestamp
///
/// Example URI:
///
/// `/pub/social/v1/posts/00321FCW75ZFY/00321FCW75ZFY.json`
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkySocialPost {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub content: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub kind: PubkySocialPostKind,
    /// If a reply, the URI of the parent post. Pubky only: a reply is a thread edge.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub parent: Option<String>,
    /// A quoted resource, pubky or web. The kind is derivable from the target.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub embed: Option<String>,
    /// Always present on the wire, `[]` when empty. An absent field reads as `[]`; an
    /// explicit `null` is invalid.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    #[serde(default)]
    pub attachments: Vec<PubkySocialAttachment>,
    /// The lock file URI, a foreign-app pubky reference. Presence means "locked content"
    /// whatever the kind; the teaser envelope inside `content` is the client's.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lock: Option<String>,
    /// Unknown members, preserved on rewrite; see the module contract in `models/mod.rs`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkySocialPost {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn content(&self) -> String {
        self.content.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn kind(&self) -> String {
        self.kind.wasm_name().to_string()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn parent(&self) -> Option<String> {
        self.parent.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn embed(&self) -> Option<String> {
        self.embed.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn attachments(&self) -> Vec<PubkySocialAttachment> {
        self.attachments.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn lock(&self) -> Option<String> {
        self.lock.clone()
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
impl Json for PubkySocialPost {}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkySocialPost {
    /// Infallible; callers validate before writing.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(
        content: String,
        kind: PubkySocialPostKind,
        parent: Option<String>,
        embed: Option<String>,
        attachments: Vec<PubkySocialAttachment>,
    ) -> Self {
        Self::new_with_lock(content, kind, parent, embed, attachments, None)
    }

    pub fn new_with_lock(
        content: String,
        kind: PubkySocialPostKind,
        parent: Option<String>,
        embed: Option<String>,
        attachments: Vec<PubkySocialAttachment>,
        lock: Option<String>,
    ) -> Self {
        let post = PubkySocialPost {
            content,
            kind,
            parent,
            embed,
            attachments,
            lock,
            extra: Default::default(),
        };
        post.sanitize()
    }
}

impl TimestampId for PubkySocialPost {}

impl HasIdPath for PubkySocialPost {
    const ROOT: Root = Root::Pub;
    const PATH_SEGMENT: &'static str = "posts/";

    fn create_path(id: &str) -> String {
        social_path(Self::ROOT, &format!("{}{id}/{id}.json", Self::PATH_SEGMENT))
    }
}

impl Validatable for PubkySocialPost {
    const MAX_BYTES: usize = VALIDATION_LIMITS.post_max_bytes;

    fn sanitize(self) -> Self {
        // Trim is the only documented canonicalization here; references pass through verbatim
        PubkySocialPost {
            content: frozen_trim(&self.content).to_string(),
            attachments: self
                .attachments
                .into_iter()
                .map(PubkySocialAttachment::sanitize)
                .collect(),
            ..self
        }
    }

    fn validate(&self, id: Option<&str>, _ctx: &ValidationCtx) -> Result<(), ValidationError> {
        if let Some(id) = id {
            self.validate_id(id)?;
        }
        self.validate_size()?;
        check_extra_keys(
            &self.extra,
            &["content", "kind", "parent", "embed", "attachments", "lock"],
        )?;

        // `Unknown` is the forwards-compat catch-all: readable, never valid to write
        if !self.kind.is_known() {
            return Err("Validation Error: post kind is unknown".into());
        }

        if let Some(parent) = &self.parent {
            check_pubky_reference("parent", parent)?;
        }
        if let Some(lock) = &self.lock {
            check_pubky_reference("lock", lock)?;
        }
        if let Some(embed) = &self.embed {
            check_target_reference("embed", embed)?;
        }

        if matches!(self.kind, PubkySocialPostKind::Collection) {
            return content::collection::validate_collection_post(self);
        }

        if self.attachments.len() > VALIDATION_LIMITS.post_attachments_max_count {
            return Err(format!(
                "Validation Error: Too many attachments (max: {})",
                VALIDATION_LIMITS.post_attachments_max_count
            ));
        }
        for (index, attachment) in self.attachments.iter().enumerate() {
            check_extra_keys(&attachment.extra, &["uri", "alt", "name"])?;
            check_target_reference(&format!("attachments[{index}].uri"), &attachment.uri)?;
            if let Some(alt) = &attachment.alt {
                if code_point_len(alt) > VALIDATION_LIMITS.attachment_alt_max_length {
                    return Err(format!(
                        "Validation Error: attachments[{index}].alt must be at most {} code points",
                        VALIDATION_LIMITS.attachment_alt_max_length
                    ));
                }
            }
            if let Some(name) = &attachment.name {
                let len = code_point_len(name);
                if len == 0 || len > VALIDATION_LIMITS.attachment_name_max_length {
                    return Err(format!(
                        "Validation Error: attachments[{index}].name must be 1..={} code points",
                        VALIDATION_LIMITS.attachment_name_max_length
                    ));
                }
            }
        }

        if matches!(self.kind, PubkySocialPostKind::Article) {
            return content::article::validate_article_post(self);
        }

        // Note, Image, Video, Link, File: untyped content
        if frozen_trim(&self.content).is_empty()
            && self.embed.is_none()
            && self.attachments.is_empty()
        {
            return Err(
                "Validation Error: Post must have content, an embed, or attachments".into(),
            );
        }
        let max = VALIDATION_LIMITS.post_note_content_max_length;
        if code_point_len(&self.content) > max {
            return Err(format!(
                "Validation Error: content must be at most {max} code points for kind {}",
                self.kind
            ));
        }
        Ok(())
    }
}

impl PubkySocialPost {
    /// Builds the article envelope into `content` and wraps it in a `kind = Article` post.
    /// Infallible; callers validate before writing.
    pub fn new_article(
        title: String,
        body: String,
        cover_image: Option<String>,
        parent: Option<String>,
        embed: Option<String>,
        attachments: Vec<PubkySocialAttachment>,
        lock: Option<String>,
    ) -> Self {
        let envelope = PubkySocialArticleContent {
            title: frozen_trim(&title).to_string(),
            body,
            cover_image,
            extra: Default::default(),
        };
        let content =
            serde_json::to_string(&envelope).expect("a string-only envelope always serializes");
        Self::new_with_lock(
            content,
            PubkySocialPostKind::Article,
            parent,
            embed,
            attachments,
            lock,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::PUB_CTX;

    const PK: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";

    fn p(path: &str) -> String {
        format!("pubky://{PK}{path}")
    }

    fn post_uri() -> String {
        p("/pub/social/v1/posts/0032SSN7Q4EVG")
    }

    fn file_uri() -> String {
        p("/pub/social/v1/files/0034A0X7NJ52G")
    }

    fn note(content: &str) -> PubkySocialPost {
        PubkySocialPost::new(
            content.to_string(),
            PubkySocialPostKind::Note,
            None,
            None,
            vec![],
        )
    }

    fn att(uri: &str) -> PubkySocialAttachment {
        PubkySocialAttachment::new(uri.to_string(), None, None)
    }

    fn post(
        kind: PubkySocialPostKind,
        parent: Option<&str>,
        embed: Option<&str>,
        attachments: Vec<PubkySocialAttachment>,
    ) -> PubkySocialPost {
        PubkySocialPost::new(
            "".into(),
            kind,
            parent.map(str::to_string),
            embed.map(str::to_string),
            attachments,
        )
    }

    fn validate(post: &PubkySocialPost) -> Result<(), String> {
        let id = post.create_id();
        post.validate(Some(&id), &PUB_CTX)
    }

    fn err(post: &PubkySocialPost) -> String {
        validate(post).expect_err("expected a validation error")
    }

    // ---- ids, paths, builders ----

    #[test]
    fn test_create_id() {
        let id = note("Hello World!").create_id();
        assert_eq!(id.len(), 13);
        assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_new() {
        let post = note("Hello World!");
        assert_eq!(post.content, "Hello World!");
        assert_eq!(post.kind, PubkySocialPostKind::Note);
        assert!(post.parent.is_none());
        assert!(post.embed.is_none());
        assert!(post.attachments.is_empty());
        assert!(post.lock.is_none());
        assert!(post.extra.is_empty());
    }

    #[test]
    fn test_default_kind_is_note() {
        assert_eq!(PubkySocialPostKind::default(), PubkySocialPostKind::Note);
    }

    #[test]
    fn test_create_path() {
        let path = PubkySocialPost::create_path("0032SSN7Q4EVG");
        assert_eq!(
            path,
            "/pub/social/v1/posts/0032SSN7Q4EVG/0032SSN7Q4EVG.json"
        );
    }

    #[test]
    fn test_validate() {
        assert!(validate(&note("Hello World!")).is_ok());
    }

    #[test]
    fn test_validate_invalid_id() {
        let post = note("Hello World!");
        assert!(post.validate(Some("INVALIDID12345"), &PUB_CTX).is_err());
    }

    #[test]
    fn test_try_from_valid() {
        let post_json = r#"{"content":"Hello World!","kind":"note","parent":null,"embed":null,"attachments":[]}"#;
        let id = note("x").create_id();
        let post = <PubkySocialPost as Validatable>::try_from(post_json.as_bytes(), &id, &PUB_CTX)
            .unwrap();
        assert_eq!(post.content, "Hello World!");
    }

    // ---- sanitize and text ops ----

    #[test]
    fn test_sanitize() {
        let parent = format!("  {}  ", post_uri());
        let post = PubkySocialPost::new(
            "\u{3000}  hello  \u{3000}".to_string(),
            PubkySocialPostKind::Note,
            Some(parent.clone()),
            None,
            vec![PubkySocialAttachment::new(
                format!("  {}  ", file_uri()),
                None,
                Some("  cat.jpg  ".to_string()),
            )],
        );
        assert_eq!(post.content, "hello");
        // References are never rewritten; validation rejects a padded one
        assert_eq!(post.parent.as_deref(), Some(parent.as_str()));
        assert_eq!(post.attachments[0].uri, format!("  {}  ", file_uri()));
        assert_eq!(post.attachments[0].name.as_deref(), Some("cat.jpg"));
        assert!(err(&post).contains("parent"));
    }

    #[test]
    fn test_sanitize_keeps_zero_width_space() {
        let post = note("\u{200B}hello\u{200B}");
        assert_eq!(post.content, "\u{200B}hello\u{200B}");
    }

    #[test]
    fn test_content_length_counts_code_points() {
        let post = note(&"\u{1F600}".repeat(VALIDATION_LIMITS.post_note_content_max_length));
        assert!(validate(&post).is_ok());
        let post = note(&"\u{1F600}".repeat(VALIDATION_LIMITS.post_note_content_max_length + 1));
        assert!(err(&post).contains("at most"));
    }

    // v0 reserved this literal; v1 keys deletion on the indexer's flag instead
    #[test]
    fn test_deleted_literal_is_ordinary_content() {
        let post_json =
            r#"{"content":"[DELETED]","kind":"note","parent":null,"embed":null,"attachments":[]}"#;
        let id = note("x").create_id();
        let post = <PubkySocialPost as Validatable>::try_from(post_json.as_bytes(), &id, &PUB_CTX)
            .expect("no reserved literal in v1");
        assert_eq!(post.content, "[DELETED]");
    }

    // ---- preservation and size caps ----

    #[test]
    fn test_unknown_members_survive_rewrite() {
        let post_json = format!(
            r#"{{"content":"hello","kind":"note","parent":null,"embed":null,"attachments":[{{"uri":"{}","focus":"center"}}],"ext":{{"badge":1}},"later":"field"}}"#,
            file_uri()
        );
        let post: PubkySocialPost = serde_json::from_str(&post_json).unwrap();
        assert_eq!(post.extra.len(), 2);
        let out: serde_json::Value = serde_json::to_value(&post).unwrap();
        assert_eq!(out["ext"]["badge"], 1);
        assert_eq!(out["later"], "field");
        assert_eq!(out["attachments"][0]["focus"], "center");
        assert_eq!(out["content"], "hello");
    }

    #[test]
    fn test_empty_extra_emits_nothing() {
        let out = serde_json::to_string(&note("hello")).unwrap();
        assert!(!out.contains("extra"), "{out}");
    }

    fn padded_blob(len: usize) -> Vec<u8> {
        let head =
            r#"{"content":"x","kind":"note","parent":null,"embed":null,"attachments":[],"pad":""#;
        let tail = r#""}"#;
        let mut blob = head.to_string();
        blob.push_str(&"a".repeat(len - head.len() - tail.len()));
        blob.push_str(tail);
        assert_eq!(blob.len(), len);
        blob.into_bytes()
    }

    #[test]
    fn test_size_cap_is_checked_before_parsing() {
        let id = note("x").create_id();
        let cap = VALIDATION_LIMITS.post_max_bytes;
        assert!(
            <PubkySocialPost as Validatable>::try_from(&padded_blob(cap), &id, &PUB_CTX).is_ok()
        );
        // Malformed on purpose: only the pre-parse check can produce the size error
        let mut oversize = padded_blob(cap + 2);
        oversize.pop();
        let err = <PubkySocialPost as Validatable>::try_from(&oversize, &id, &PUB_CTX).unwrap_err();
        assert!(err.contains("exceeds"), "{err}");
    }

    #[test]
    fn test_extra_must_not_shadow_a_field() {
        let mut post = note("real");
        post.extra.insert("kind".into(), "article".into());
        assert!(err(&post).contains("shadow"));
        let mut post = note("real");
        post.attachments.push(att(&file_uri()));
        post.attachments[0].extra.insert("uri".into(), "x".into());
        assert!(err(&post).contains("shadow"));
    }

    #[test]
    fn test_size_cap_applies_to_built_objects() {
        let mut post = note("x");
        post.extra.insert(
            "pad".into(),
            serde_json::Value::String("a".repeat(VALIDATION_LIMITS.post_max_bytes)),
        );
        assert!(err(&post).contains("exceeds"));
    }

    // ---- kinds ----

    #[test]
    fn test_kinds_round_trip_and_retired_spellings_read_as_unknown() {
        for (kind, wire) in [
            (PubkySocialPostKind::Note, "\"note\""),
            (PubkySocialPostKind::Article, "\"article\""),
            (PubkySocialPostKind::Image, "\"image\""),
            (PubkySocialPostKind::Video, "\"video\""),
            (PubkySocialPostKind::Link, "\"link\""),
            (PubkySocialPostKind::File, "\"file\""),
            (PubkySocialPostKind::Collection, "\"collection\""),
        ] {
            assert_eq!(serde_json::to_string(&kind).unwrap(), wire);
            assert_eq!(
                serde_json::from_str::<PubkySocialPostKind>(wire).unwrap(),
                kind
            );
            assert_eq!(kind.to_string(), wire.trim_matches('"'));
            assert!(kind.is_known());
        }
        for retired in ["\"short\"", "\"long\"", "\"hologram\""] {
            let kind: PubkySocialPostKind = serde_json::from_str(retired).unwrap();
            assert_eq!(kind, PubkySocialPostKind::Unknown);
            assert!(!kind.is_known());
        }
        assert_eq!(PubkySocialPostKind::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_fromstr_never_produces_unknown() {
        assert_eq!(
            "note".parse::<PubkySocialPostKind>().unwrap(),
            PubkySocialPostKind::Note
        );
        assert_eq!(
            "article".parse::<PubkySocialPostKind>().unwrap(),
            PubkySocialPostKind::Article
        );
        assert_eq!(
            "collection".parse::<PubkySocialPostKind>().unwrap(),
            PubkySocialPostKind::Collection
        );
        for bad in ["short", "long", "unknown", "Note", ""] {
            assert!(bad.parse::<PubkySocialPostKind>().is_err(), "{bad}");
        }
    }

    #[test]
    fn test_unknown_kind_reads_then_fails_validation() {
        let post_json =
            r#"{"content":"hello","kind":"short","parent":null,"embed":null,"attachments":[]}"#;
        let post: PubkySocialPost = serde_json::from_str(post_json).unwrap();
        assert_eq!(post.kind, PubkySocialPostKind::Unknown);
        assert!(err(&post).contains("post kind is unknown"));
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn test_kind_wasm_getter() {
        let mut post = note("x");
        assert_eq!(post.kind(), "Note");
        post.kind = PubkySocialPostKind::Article;
        assert_eq!(post.kind(), "Article");
        post.kind = PubkySocialPostKind::Unknown;
        assert_eq!(post.kind(), "Unknown");
    }

    // ---- references ----

    #[test]
    fn test_parent_is_pubky_only_and_canonical() {
        let mut ok = post(PubkySocialPostKind::Note, Some(&post_uri()), None, vec![]);
        ok.content = "re".into();
        assert!(validate(&ok).is_ok());
        for bad in [
            "https://example.com/post".to_string(),
            format!("pubky{PK}/pub/social/v1/posts/0032SSN7Q4EVG"),
            p("/pub/social/v1/posts/../profile.json"),
            p("/pub/social/v1/posts/00%32"),
            String::new(),
        ] {
            let mut reply = post(PubkySocialPostKind::Note, Some(&bad), None, vec![]);
            reply.content = "re".into();
            assert!(err(&reply).contains("parent"), "{bad}");
        }
    }

    #[test]
    fn test_embed_accepts_pubky_and_web() {
        for ok in [post_uri(), "https://example.com/a?b=c".to_string()] {
            let quote = post(PubkySocialPostKind::Note, None, Some(&ok), vec![]);
            assert!(validate(&quote).is_ok(), "{ok}");
        }
        for bad in ["nostr:nevent1abc", "ftp://x/y", " https://example.com", ""] {
            let quote = post(PubkySocialPostKind::Note, None, Some(bad), vec![]);
            assert!(err(&quote).contains("embed"), "{bad}");
        }
    }

    #[test]
    fn test_lock_is_a_canonical_pubky_uri() {
        let lock = p("/pub/app.locks/0032SSN7Q4EVG.json");
        let post = PubkySocialPost::new_with_lock(
            "Visible preview".into(),
            PubkySocialPostKind::Note,
            None,
            None,
            vec![],
            Some(lock.clone()),
        );
        assert_eq!(post.lock.as_deref(), Some(lock.as_str()));
        assert!(validate(&post).is_ok());

        for bad in [
            "https://locks.example/0032SSN7Q4EVG".to_string(),
            "pubky:lock-id".to_string(),
            String::new(),
            "   ".to_string(),
            p(&format!(
                "/pub/{}",
                "a".repeat(VALIDATION_LIMITS.reference_uri_max_length)
            )),
        ] {
            let post = PubkySocialPost::new_with_lock(
                "Visible preview".into(),
                PubkySocialPostKind::Note,
                None,
                None,
                vec![],
                Some(bad.clone()),
            );
            assert!(err(&post).contains("lock"), "{bad}");
        }
    }

    #[test]
    fn test_missing_lock_deserializes_unlocked() {
        let post_json =
            r#"{"content":"hello","kind":"note","parent":null,"embed":null,"attachments":[]}"#;
        let post: PubkySocialPost = serde_json::from_str(post_json).unwrap();
        assert!(post.lock.is_none());
        let out = serde_json::to_string(&post).unwrap();
        assert!(!out.contains("lock"));
    }

    #[test]
    fn test_collection_post_lock_rule_applies() {
        let content = r#"{"name":"Favorites","items":[]}"#.to_string();
        let ok = PubkySocialPost::new_with_lock(
            content.clone(),
            PubkySocialPostKind::Collection,
            None,
            None,
            vec![],
            Some(p("/pub/app.locks/0032SSN7Q4EVG.json")),
        );
        assert!(validate(&ok).is_ok());
        let bad = PubkySocialPost::new_with_lock(
            content,
            PubkySocialPostKind::Collection,
            None,
            None,
            vec![],
            Some("https://locks.example/x".into()),
        );
        assert!(err(&bad).contains("lock"));
    }

    // ---- attachments ----

    #[test]
    fn test_attachments_absent_reads_empty_and_null_is_invalid() {
        let post: PubkySocialPost =
            serde_json::from_str(r#"{"content":"hello","kind":"note","parent":null,"embed":null}"#)
                .unwrap();
        assert!(post.attachments.is_empty());
        let null = serde_json::from_str::<PubkySocialPost>(
            r#"{"content":"hello","kind":"note","parent":null,"embed":null,"attachments":null}"#,
        );
        assert!(null.is_err());
    }

    #[test]
    fn test_attachments_serialize_as_objects() {
        let post = PubkySocialPost::new(
            "".into(),
            PubkySocialPostKind::Image,
            None,
            None,
            vec![PubkySocialAttachment::new(
                file_uri(),
                Some("a cat".into()),
                Some("cat.jpg".into()),
            )],
        );
        assert!(validate(&post).is_ok());
        let out: serde_json::Value = serde_json::to_value(&post).unwrap();
        assert_eq!(out["attachments"][0]["uri"], file_uri());
        assert_eq!(out["attachments"][0]["alt"], "a cat");
        assert_eq!(out["attachments"][0]["name"], "cat.jpg");
        let bare: serde_json::Value = serde_json::to_value(att(&file_uri())).unwrap();
        assert_eq!(bare.as_object().unwrap().len(), 1, "{bare}");
    }

    #[test]
    fn test_attachments_count_cap() {
        let max = VALIDATION_LIMITS.post_attachments_max_count;
        let many = |n: usize| {
            PubkySocialPost::new(
                "".into(),
                PubkySocialPostKind::Image,
                None,
                None,
                (0..n).map(|_| att(&file_uri())).collect(),
            )
        };
        assert!(validate(&many(max)).is_ok());
        assert!(err(&many(max + 1)).contains("Too many attachments"));
    }

    #[test]
    fn test_attachment_uri_rule() {
        let image = |uri: &str| post(PubkySocialPostKind::Image, None, None, vec![att(uri)]);
        for ok in [file_uri(), "https://example.com/cat.jpg".to_string()] {
            assert!(validate(&image(&ok)).is_ok(), "{ok}");
        }
        let max = VALIDATION_LIMITS.reference_uri_max_length;
        let long_ok = p(&format!(
            "/pub/{}",
            "a".repeat(max - code_point_len(&p("/pub/")))
        ));
        assert_eq!(code_point_len(&long_ok), max);
        assert!(validate(&image(&long_ok)).is_ok());

        for bad in [
            "ipfs://bafy".to_string(),
            format!("pubky{PK}/pub/social/v1/files/0034A0X7NJ52G"),
            format!("{long_ok}a"),
            "not a url".to_string(),
            String::new(),
        ] {
            assert!(err(&image(&bad)).contains("attachments[0].uri"), "{bad}");
        }
    }

    #[test]
    fn test_attachment_alt_and_name_caps() {
        let with = |alt: Option<String>, name: Option<String>| {
            PubkySocialPost::new(
                "".into(),
                PubkySocialPostKind::Image,
                None,
                None,
                vec![PubkySocialAttachment::new(file_uri(), alt, name)],
            )
        };
        let alt_max = VALIDATION_LIMITS.attachment_alt_max_length;
        assert!(validate(&with(Some("\u{1F600}".repeat(alt_max)), None)).is_ok());
        assert!(err(&with(Some("a".repeat(alt_max + 1)), None)).contains("alt"));

        let name_max = VALIDATION_LIMITS.attachment_name_max_length;
        assert!(validate(&with(None, Some("\u{1F600}".repeat(name_max)))).is_ok());
        assert!(err(&with(None, Some("a".repeat(name_max + 1)))).contains("name"));
        assert!(err(&with(None, Some("".into()))).contains("name"));
        assert!(err(&with(None, Some("   ".into()))).contains("name"));
    }

    #[test]
    fn test_attachment_unknown_members_survive() {
        let a: PubkySocialAttachment =
            serde_json::from_str(&format!(r#"{{"uri":"{}","focus":"center"}}"#, file_uri()))
                .unwrap();
        let out: serde_json::Value = serde_json::to_value(&a).unwrap();
        assert_eq!(out["focus"], "center");
    }

    // ---- at least one of ----

    #[test]
    fn test_empty_post_rejected() {
        assert!(err(&note("")).contains("must have content"));
        assert!(err(&note("   \u{3000}")).contains("must have content"));
    }

    #[test]
    fn test_embed_or_attachment_alone_is_enough() {
        let embed_only = PubkySocialPost::new(
            "".into(),
            PubkySocialPostKind::Note,
            None,
            Some(post_uri()),
            vec![],
        );
        assert!(validate(&embed_only).is_ok());
        let attachment_only = PubkySocialPost::new(
            "".into(),
            PubkySocialPostKind::Image,
            None,
            None,
            vec![att(&file_uri())],
        );
        assert!(validate(&attachment_only).is_ok());
    }
}
