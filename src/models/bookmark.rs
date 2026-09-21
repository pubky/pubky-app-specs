use crate::canonicalize::canonicalize_universal;
use crate::constants::social_path;
use crate::limits::VALIDATION_LIMITS;
use crate::traits::{hash_id_of, Root, ValidationCtx, ValidationError};
use crate::{
    common::{check_extra, timestamp, validate_safe_json_int},
    traits::{HasIdPath, Validatable},
};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use crate::traits::Json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Bookmarks are private, so their context is fixed whatever a caller passes.
const PRIV_CTX: ValidationCtx = ValidationCtx {
    root: PubkySocialBookmark::ROOT,
};

/// A bookmark of any URI.
/// URI: /priv/social/v1/bookmarks/:filename.json
///
/// The target lives in the FILENAME, so listing every bookmark costs one LIST and, for the
/// primary form, no GETs; an overflow entry costs one GET for its `target`.
/// Two forms, told apart by the first character:
///
/// - primary, a canonical target of at most 187 UTF-8 bytes: the filename is the target in
///   unpadded base64url (250 characters, plus `.json` the 255-character segment maximum), and
///   the content is `{"created_at"}` alone.
/// - overflow, longer than that: the filename is `~` plus the target's hash, which is one way,
///   so the content carries `target` and reading it costs one GET.
///
/// `Validatable` alone is NOT the full validation surface here: the target rules live on the
/// filename, so they run only when `validate` is given one. [`bookmark_target`] is the whole
/// read-side rule and [`create_bookmark`] the whole write-side one.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkySocialBookmark {
    pub created_at: i64,
    /// The canonical target, required in the overflow form and forbidden in the primary form,
    /// where the filename carries it.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Unknown members, preserved on rewrite; see the module contract in `models/mod.rs`.
    #[serde(flatten)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// What a bookmark write needs: the object, the filename carrying the target, and the path.
#[derive(Debug, Clone)]
pub struct CreatedBookmark {
    pub bookmark: PubkySocialBookmark,
    pub filename: String,
    pub path: String,
}

/// The object and the path a target is bookmarked at. Same target, same filename, so a second
/// call overwrites the first instead of forking a duplicate.
pub fn create_bookmark(target: &str) -> Result<CreatedBookmark, String> {
    let canonical = canonical_target(target)?;
    let filename = filename_of(&canonical);
    let bookmark = PubkySocialBookmark {
        created_at: timestamp(),
        target: filename.starts_with('~').then_some(canonical),
        extra: Default::default(),
    };
    bookmark.validate(Some(&filename), &PRIV_CTX)?;
    let path = PubkySocialBookmark::create_path(&filename);
    Ok(CreatedBookmark {
        bookmark,
        filename,
        path,
    })
}

/// The filename a target is stored under, without building the object.
pub fn bookmark_filename(target: &str) -> Result<String, ValidationError> {
    Ok(filename_of(&canonical_target(target)?))
}

/// One verdict for a target: the canonicalizer carries the reference cap, so junk and an
/// over-long value fail the same way.
fn canonical_target(target: &str) -> Result<String, ValidationError> {
    canonicalize_universal(target).map_err(|_| non_canonical(target))
}

fn non_canonical(target: &str) -> String {
    format!(
        "Validation Error: bookmark target must be a canonical URI of at most {} code points: {target}",
        VALIDATION_LIMITS.reference_uri_max_length
    )
}

/// The form split keys on UTF-8 BYTES; the 1024 cap above keys on code points. Never mix them.
fn filename_of(canonical: &str) -> String {
    if canonical.len() <= VALIDATION_LIMITS.bookmark_target_uri_max_bytes {
        URL_SAFE_NO_PAD.encode(canonical)
    } else {
        ["~", &hash_id_of(canonical)].concat()
    }
}

/// The target a stored entry names, or why the entry is invalid. A reader skips an invalid
/// entry; a writer fails the write. `filename` is the leaf with `.json` already stripped,
/// which is unambiguous because `.` is outside the base64url alphabet.
pub fn bookmark_target(
    filename: &str,
    content: &PubkySocialBookmark,
) -> Result<String, ValidationError> {
    match filename.strip_prefix('~') {
        Some(hash) => overflow_target(hash, content),
        None => primary_target(filename, content),
    }
}

fn primary_target(
    filename: &str,
    content: &PubkySocialBookmark,
) -> Result<String, ValidationError> {
    if content.target.is_some() {
        return Err(
            "Validation Error: a primary bookmark carries its target in the filename, not in the content"
                .to_string(),
        );
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(filename)
        .map_err(|_| not_base64(filename))?;
    // Re-encoding is the alias check: padding, a foreign alphabet and set trailing bits would
    // each name one target under a second homeserver key.
    if URL_SAFE_NO_PAD.encode(&bytes) != filename {
        return Err(not_base64(filename));
    }
    // Fatal on invalid UTF-8; a replacement character would invent a target.
    let target = String::from_utf8(bytes)
        .map_err(|_| format!("Validation Error: bookmark filename is not UTF-8: {filename}"))?;
    // The fixed point: a short-form or padded spelling of one target must not fork dedup.
    if canonical_target(&target)? != target {
        return Err(non_canonical(&target));
    }
    // The upper bound closes the same fork from the other side: without it one target has a
    // primary spelling AND an overflow one, and the leaf runs past the 255-character segment.
    if target.len() > VALIDATION_LIMITS.bookmark_target_uri_max_bytes {
        return Err(format!(
            "Validation Error: a target over {} bytes belongs in the overflow bookmark form",
            VALIDATION_LIMITS.bookmark_target_uri_max_bytes
        ));
    }
    Ok(target)
}

fn overflow_target(hash: &str, content: &PubkySocialBookmark) -> Result<String, ValidationError> {
    let target = content.target.as_deref().ok_or_else(|| {
        "Validation Error: an overflow bookmark requires target in the content".to_string()
    })?;
    check_stored_target(target)?;
    if hash != hash_id_of(target) {
        return Err(format!(
            "Validation Error: bookmark filename does not hash its target: {target}"
        ));
    }
    Ok(target.to_string())
}

/// The rules a stored `target` obeys wherever it is seen. Only the overflow form stores one,
/// so it is canonical and past what the primary filename can carry, whether or not the caller
/// brought the filename that would say so.
fn check_stored_target(target: &str) -> Result<(), ValidationError> {
    if canonical_target(target)? != target {
        return Err(non_canonical(target));
    }
    if target.len() <= VALIDATION_LIMITS.bookmark_target_uri_max_bytes {
        return Err(format!(
            "Validation Error: a target of {} bytes belongs in the primary bookmark form",
            target.len()
        ));
    }
    Ok(())
}

fn not_base64(filename: &str) -> String {
    format!("Validation Error: bookmark filename is not canonical base64url: {filename}")
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkySocialBookmark {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = fromJson))]
    pub fn from_json(js_value: &JsValue) -> Result<Self, String> {
        Self::import_json(js_value)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = toJson))]
    pub fn to_json(&self) -> Result<JsValue, String> {
        self.export_json()
    }

    /// Getter for `target`, set only in the overflow form.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn target(&self) -> Option<String> {
        self.target.clone()
    }
}

#[cfg(target_arch = "wasm32")]
impl Json for PubkySocialBookmark {}

impl HasIdPath for PubkySocialBookmark {
    const ROOT: Root = Root::Priv;
    const PATH_SEGMENT: &'static str = "bookmarks/";

    fn create_path(filename: &str) -> String {
        social_path(
            Self::ROOT,
            &format!("{}{filename}.json", Self::PATH_SEGMENT),
        )
    }
}

impl Validatable for PubkySocialBookmark {
    fn validate_fields(
        &self,
        id: Option<&str>,
        _ctx: &ValidationCtx,
    ) -> Result<(), ValidationError> {
        check_extra(&self.extra, &["created_at", "target"])?;
        validate_safe_json_int(self.created_at)?;
        match id {
            // The identity is the filename, so it brings the whole target surface with it.
            Some(filename) => bookmark_target(filename, self).map(|_| ())?,
            // Without one the form is unknown, but a stored target is always an overflow
            // target, so a JSON import cannot smuggle junk in past the filename rules.
            None => {
                if let Some(target) = &self.target {
                    check_stored_target(target)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bookmark_uri_builder, PubkySocialObject, Resource, Visibility};

    const PK: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";

    fn target() -> String {
        format!("pubky://{PK}/pub/social/v1/posts/0032SSN7Q4EVG")
    }

    /// A canonical web target of exactly `bytes` UTF-8 bytes.
    fn web_target(bytes: usize) -> String {
        let head = "https://example.com/";
        format!("{head}{}", "a".repeat(bytes - head.len()))
    }

    fn content(created_at: i64, target: Option<&str>) -> PubkySocialBookmark {
        PubkySocialBookmark {
            created_at,
            target: target.map(str::to_string),
            extra: Default::default(),
        }
    }

    #[test]
    fn primary_form_round_trips_at_the_segment_edge() {
        let target = web_target(VALIDATION_LIMITS.bookmark_target_uri_max_bytes);
        let created = create_bookmark(&target).unwrap();
        assert_eq!(created.filename.len(), 250);
        assert!(created.bookmark.target.is_none());
        assert_eq!(
            bookmark_target(&created.filename, &created.bookmark),
            Ok(target)
        );
        // 250 characters plus ".json" is the 255-character segment maximum
        let leaf = created.path.rsplit('/').next().unwrap();
        assert_eq!(leaf.len(), 255);
    }

    #[test]
    fn one_byte_over_the_edge_takes_the_overflow_form() {
        let target = web_target(VALIDATION_LIMITS.bookmark_target_uri_max_bytes + 1);
        let created = create_bookmark(&target).unwrap();
        assert_eq!(created.filename, format!("~{}", hash_id_of(&target)));
        assert_eq!(created.bookmark.target.as_deref(), Some(target.as_str()));
        assert_eq!(
            bookmark_target(&created.filename, &created.bookmark),
            Ok(target)
        );
    }

    #[test]
    fn the_reference_cap_counts_code_points() {
        let max = VALIDATION_LIMITS.reference_uri_max_length;
        // Multi-byte, so the byte length is far over the cap while the code points are not
        let target = format!("nostr:{}", "\u{4e2d}".repeat(max - 6));
        assert!(create_bookmark(&target).is_ok());
        let target = format!("nostr:{}", "\u{4e2d}".repeat(max - 5));
        assert!(create_bookmark(&target).unwrap_err().contains("1024"));
    }

    #[test]
    fn one_target_spelled_twice_gives_one_filename() {
        // The SDK short form and the full form
        assert_eq!(
            bookmark_filename(&format!("pubky{PK}/pub/social/v1/posts/0032SSN7Q4EVG")).unwrap(),
            bookmark_filename(&target()).unwrap()
        );
        // The web gate trims, so a trailing space is the same bookmark
        assert_eq!(
            bookmark_filename("https://example.com/a ").unwrap(),
            bookmark_filename("https://example.com/a").unwrap()
        );
        // A leading space defeats dispatch instead: the external arm refuses to claim http(s)
        assert!(bookmark_filename(" https://example.com/a").is_err());
        // Either non-canonical spelling stored AS the filename is not a fixed point, so it
        // cannot fork dedup by naming one target under a second key
        for raw in [
            "https://example.com/a ".to_string(),
            format!("pubky{PK}/pub/social/v1/posts/0032SSN7Q4EVG"),
        ] {
            let alias = URL_SAFE_NO_PAD.encode(&raw);
            assert!(bookmark_target(&alias, &content(1, None))
                .unwrap_err()
                .contains("canonical"));
        }
    }

    #[test]
    fn base64_aliases_of_one_filename_all_reject() {
        let good = bookmark_filename(&target()).unwrap();
        let empty = content(1, None);
        assert!(bookmark_target(&good, &empty).is_ok());
        // Padding, the standard base64 alphabet, and set trailing bits in the last sextet
        for alias in [
            format!("{good}="),
            format!("+{}", &good[1..]),
            format!("/{}", &good[1..]),
            format!("{}x", &good[..good.len() - 1]),
        ] {
            assert!(
                bookmark_target(&alias, &empty).is_err(),
                "{alias} was accepted"
            );
        }
        // A decode that is not UTF-8 is fatal, never repaired into replacement characters
        let junk = URL_SAFE_NO_PAD.encode([0xff]);
        assert!(bookmark_target(&junk, &empty)
            .unwrap_err()
            .contains("not UTF-8"));
    }

    #[test]
    fn the_two_forms_never_borrow_each_others_content() {
        let long = web_target(VALIDATION_LIMITS.bookmark_target_uri_max_bytes + 1);
        let overflow = format!("~{}", hash_id_of(&long));
        let primary = bookmark_filename(&target()).unwrap();
        // A primary content may not carry a target
        assert!(bookmark_target(&primary, &content(1, Some(&target())))
            .unwrap_err()
            .contains("in the filename"));
        // An overflow content must
        assert!(bookmark_target(&overflow, &content(1, None))
            .unwrap_err()
            .contains("requires target"));
        // whose hash is the filename
        assert!(
            bookmark_target(&overflow, &content(1, Some(&web_target(200))))
                .unwrap_err()
                .contains("does not hash")
        );
        // and which does not fit the primary form
        let short = format!("~{}", hash_id_of(&target()));
        assert!(bookmark_target(&short, &content(1, Some(&target())))
            .unwrap_err()
            .contains("primary bookmark form"));
        // The primary form is bounded from above too, or one target gets two valid filenames
        // and the leaf outgrows the segment
        let huge = web_target(500);
        let spelled_primary = URL_SAFE_NO_PAD.encode(&huge);
        assert!(bookmark_target(&spelled_primary, &content(1, None))
            .unwrap_err()
            .contains("overflow bookmark form"));
        let blob = br#"{"created_at":1727740800000000}"#;
        // Twice over: the parser will not classify a leaf that long, and the model refuses it
        // even when a caller hand-builds the resource
        let uri = bookmark_uri_builder(PK.into(), spelled_primary.clone());
        assert!(PubkySocialObject::from_uri(&uri, blob).is_err());
        let resource = Resource::Bookmark(spelled_primary);
        assert!(PubkySocialObject::from_resource(&resource, blob, &PRIV_CTX)
            .unwrap_err()
            .contains("overflow bookmark form"));
        // A stored target over the reference cap fails on READ, not only at the builder
        let over_cap = format!(
            "nostr:{}",
            "a".repeat(VALIDATION_LIMITS.reference_uri_max_length)
        );
        assert!(bookmark_target(
            &format!("~{}", hash_id_of(&over_cap)),
            &content(1, Some(&over_cap))
        )
        .unwrap_err()
        .contains("1024"));
    }

    #[test]
    fn a_content_without_its_filename_still_answers_for_the_target_it_stores() {
        // Only the overflow form stores a target, so a JSON import cannot smuggle one in
        assert!(content(1, Some("junk"))
            .validate(None, &PRIV_CTX)
            .unwrap_err()
            .contains("canonical"));
        assert!(content(1, Some(&target()))
            .validate(None, &PRIV_CTX)
            .unwrap_err()
            .contains("primary bookmark form"));
        // A real overflow content passes with no filename in hand
        let long = web_target(VALIDATION_LIMITS.bookmark_target_uri_max_bytes + 1);
        assert!(content(1, Some(&long)).validate(None, &PRIV_CTX).is_ok());
        assert!(content(1, None).validate(None, &PRIV_CTX).is_ok());
    }

    #[test]
    fn a_bookmark_is_never_read_under_the_public_root() {
        let created = create_bookmark(&target()).unwrap();
        let blob = serde_json::to_vec(&created.bookmark).unwrap();
        let resource = Resource::Bookmark(created.filename.clone());
        assert!(PubkySocialObject::from_resource(&resource, &blob, &PRIV_CTX).is_ok());
        assert!(
            PubkySocialObject::from_resource(&resource, &blob, &crate::PUB_CTX)
                .unwrap_err()
                .contains("never a public object")
        );
    }

    #[test]
    fn the_path_is_private_and_the_parser_reads_the_filename_back() {
        let created = create_bookmark(&target()).unwrap();
        assert_eq!(
            created.path,
            format!("/priv/social/v1/bookmarks/{}.json", created.filename)
        );
        let uri = bookmark_uri_builder(PK.into(), created.filename.clone());
        let parsed = crate::ParsedUri::try_from(uri.as_str()).unwrap();
        assert_eq!(parsed.visibility, Visibility::Private);
        assert_eq!(
            parsed.resource,
            Resource::Bookmark(created.filename.clone())
        );
        assert_eq!(parsed.try_to_uri_str().unwrap(), uri);
        // A public bookmark path is not a bookmark
        let public = uri.replace("/priv/", "/pub/");
        let parsed = crate::ParsedUri::try_from(public.as_str()).unwrap();
        assert!(matches!(parsed.resource, Resource::Unknown));
        // Ingest by URI recovers the target from the filename alone
        let blob = serde_json::to_vec(&created.bookmark).unwrap();
        match PubkySocialObject::from_uri(&uri, &blob).unwrap() {
            PubkySocialObject::Bookmark(b) => {
                assert_eq!(bookmark_target(&created.filename, &b), Ok(target()))
            }
            other => panic!("expected a Bookmark, got {other:?}"),
        }
        assert!(PubkySocialObject::from_uri(&public, &blob).is_err());
    }

    #[test]
    fn unknown_members_survive_and_created_at_is_safe() {
        let filename = bookmark_filename(&target()).unwrap();
        let blob = br#"{"created_at":1727740800000000,"ext":{"folder":"reading"}}"#;
        let bookmark =
            <PubkySocialBookmark as Validatable>::try_from(blob, &filename, &PRIV_CTX).unwrap();
        assert_eq!(bookmark.extra["ext"]["folder"], "reading");
        let back = serde_json::to_string(&bookmark).unwrap();
        assert_eq!(
            back,
            r#"{"created_at":1727740800000000,"ext":{"folder":"reading"}}"#
        );
        let mut shadow = content(1, None);
        shadow.extra.insert("target".into(), "x".into());
        assert!(shadow
            .validate(Some(&filename), &PRIV_CTX)
            .unwrap_err()
            .contains("shadow"));
        let huge = content(i64::MAX, None);
        assert!(huge
            .validate(Some(&filename), &PRIV_CTX)
            .unwrap_err()
            .contains("JSON-safe"));
        // Without a filename the primary content alone still validates
        assert!(content(1, None).validate(None, &PRIV_CTX).is_ok());
    }
}
