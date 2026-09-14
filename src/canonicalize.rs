//! The engine-free URI canonicalizers. The parser and the collection item check route through
//! [`canonicalize_pubky_uri`] today; the model reference fields still sit on their v0 `url::Url`
//! validation and move here with their own changes, field by field, because re-wiring a hashed
//! field re-ids data. No `url::Url` here or in the parser: an engine parser repairs junk into
//! acceptance (userinfo stripped, `..` collapsed, query and fragment ignored) and its behavior
//! cannot be pinned across versions.

use crate::common::{ascii_fold, code_point_len, frozen_trim, is_frozen_whitespace};
use crate::limits::VALIDATION_LIMITS;
use crate::types::PubkyId;

/// One fold-point for every pubky URI. Accepts the full form `pubky://<pk>[/<path>]` and the
/// SDK short form `pubky<pk>[/<path>]`; the canonical output is always the full form.
/// Idempotent: canonicalizing the output returns it unchanged.
// The error is deliberately unit: every rejection is the same "not a pubky URI" verdict, and
// the callers classify, they do not report.
#[allow(clippy::result_unit_err)]
pub fn canonicalize_pubky_uri(raw: &str) -> Result<String, ()> {
    // Scheme, case-sensitive. The prefix is ASCII, which keeps the later slicing safe.
    let rest = raw
        .strip_prefix("pubky://")
        .or_else(|| raw.strip_prefix("pubky"))
        .ok_or(())?;
    // Host: up to the first '/'. No userinfo, no port, then a canonical PubkyId.
    let (host, path) = match rest.find('/') {
        Some(i) => (&rest[..i], Some(&rest[i + 1..])),
        None => (rest, None),
    };
    if host.contains(['@', ':']) || PubkyId::try_from(host).is_err() {
        return Err(());
    }
    let Some(path) = path else {
        // A bare host is a user reference and is canonical.
        return Ok(["pubky://", host].concat());
    };
    // Segments: no empty segment (kills `//`, leading and trailing slashes), no `.` or `..`,
    // and nowhere a `%`, `?`, `#`, an ASCII control, or a frozen-whitespace code point.
    // Everything else, including non-ASCII, passes: foreign apps may use it.
    if path.split('/').any(|seg| {
        seg.is_empty()
            || seg == "."
            || seg == ".."
            || seg.chars().any(|c| {
                matches!(c, '%' | '?' | '#') || c.is_ascii_control() || is_frozen_whitespace(c)
            })
    }) {
        return Err(());
    }
    Ok(["pubky://", host, "/", path].concat())
}

/// The web gate: the stored and hashed form of an `http`/`https` reference is the trimmed raw
/// string. Stricter than a browser in one direction: an embedded tab or newline rejects here,
/// where WHATWG would silently strip it.
#[allow(clippy::result_unit_err)]
pub fn canonicalize_web_uri(raw: &str) -> Result<String, ()> {
    let s = frozen_trim(raw);
    if s.chars()
        .any(|c| c.is_ascii_control() || is_frozen_whitespace(c))
    {
        return Err(());
    }
    let after = s
        .strip_prefix("http://")
        .or_else(|| s.strip_prefix("https://"));
    // The authority runs to the first `/`, `?` or `#` and must not be empty
    match after {
        Some(rest) if !rest.starts_with(['/', '?', '#']) && !rest.is_empty() => Ok(s.to_string()),
        _ => Err(()),
    }
}

/// Bookmark-target dispatch: pubky URIs (either form) through the pubky canonicalizer,
/// `http`/`https` through the web gate. Inspects the raw string untrimmed, so a pasted
/// leading space defeats dispatch on purpose; UIs pre-trim. Caps the canonical output at
/// `reference_uri_max_length` code points.
#[allow(clippy::result_unit_err)]
pub fn canonicalize_target(raw: &str) -> Result<String, ()> {
    let canonical = if raw.starts_with("pubky") {
        canonicalize_pubky_uri(raw)?
    } else if raw.starts_with("http://") || raw.starts_with("https://") {
        canonicalize_web_uri(raw)?
    } else {
        return Err(());
    };
    if code_point_len(&canonical) > VALIDATION_LIMITS.reference_uri_max_length {
        return Err(());
    }
    Ok(canonical)
}

/// The universal tier's third arm: any scheme-shaped URI that is not pubky, http or https
/// (nostr, geo, ipfs, magnet, did). The scheme folds to lowercase; the rest is opaque, an
/// identifier rather than a location this crate resolves.
#[allow(clippy::result_unit_err)]
pub fn canonicalize_external_uri(raw: &str) -> Result<String, ()> {
    let s = frozen_trim(raw);
    if s.chars()
        .any(|c| c.is_ascii_control() || is_frozen_whitespace(c))
    {
        return Err(());
    }
    let colon = s.find(':').ok_or(())?;
    if colon == 0 || colon + 1 == s.len() {
        return Err(());
    }
    let (scheme, rest) = s.split_at(colon);
    let mut cs = scheme.chars();
    if !cs.next().ok_or(())?.is_ascii_alphabetic()
        || !cs.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-'))
    {
        return Err(());
    }
    let folded = ascii_fold(scheme);
    // The pubky scheme space is reserved and http(s) has its own gate; dispatch claims both
    // before this arm, so seeing one here means a caller bypassed it.
    if folded.starts_with("pubky") || folded == "http" || folded == "https" {
        return Err(());
    }
    Ok([&folded, rest].concat())
}

/// Universal dispatch: pubky (either form) and web through their own gates, everything else
/// through the external arm. Capped like `canonicalize_target`.
#[allow(clippy::result_unit_err)]
pub fn canonicalize_universal(raw: &str) -> Result<String, ()> {
    let canonical = if raw.starts_with("pubky") {
        canonicalize_pubky_uri(raw)?
    } else if raw.starts_with("http://") || raw.starts_with("https://") {
        canonicalize_web_uri(raw)?
    } else {
        canonicalize_external_uri(raw)?
    };
    if code_point_len(&canonical) > VALIDATION_LIMITS.reference_uri_max_length {
        return Err(());
    }
    Ok(canonical)
}

/// A stored reference is the fixed point of its own canonical spelling. Nothing rewrites it
/// on the way in or out, so the SDK short form and any padding reject here.
pub(crate) fn check_pubky_reference(field: &str, raw: &str) -> Result<(), String> {
    let max = VALIDATION_LIMITS.reference_uri_max_length;
    let ok = canonicalize_pubky_uri(raw).is_ok_and(|c| c == raw) && code_point_len(raw) <= max;
    if ok {
        return Ok(());
    }
    Err(format!(
        "Validation Error: {field} must be a canonical pubky URI of at most {max} code points: {raw}"
    ))
}

/// A reference to a post: public, versionless, and a fixed point of the parser's own emitter
/// (which rejects the short form). Replies and collection items share it.
pub(crate) fn check_post_reference(raw: &str) -> Result<(), String> {
    let parsed = crate::ParsedUri::try_from(raw)
        .map_err(|e| format!("must be a canonical post URI: {e}"))?;
    match (parsed.visibility, &parsed.resource) {
        (crate::Visibility::Public, crate::Resource::Post { version: None, .. }) => {
            if parsed.try_to_uri_str().as_deref() == Ok(raw) {
                Ok(())
            } else {
                Err(format!("must be spelled in canonical form: {raw}"))
            }
        }
        _ => Err(format!(
            "must be a public, versionless post reference: {raw}"
        )),
    }
}

/// Same rule for the fields that also accept `http`/`https`; `canonicalize_target` caps.
pub(crate) fn check_target_reference(field: &str, raw: &str) -> Result<(), String> {
    if canonicalize_target(raw).is_ok_and(|c| c == raw) {
        return Ok(());
    }
    Err(format!(
        "Validation Error: {field} must be a canonical pubky or web URI of at most {} code points: {raw}",
        VALIDATION_LIMITS.reference_uri_max_length
    ))
}

/// Same rule for the universal fields; `canonicalize_universal` caps.
pub(crate) fn check_universal_reference(field: &str, raw: &str) -> Result<(), String> {
    if canonicalize_universal(raw).is_ok_and(|c| c == raw) {
        return Ok(());
    }
    Err(format!(
        "Validation Error: {field} must be a canonical URI of at most {} code points: {raw}",
        VALIDATION_LIMITS.reference_uri_max_length
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOST: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";

    fn p(path: &str) -> String {
        format!("pubky://{HOST}{path}")
    }

    #[test]
    fn pubky_uri_accepts_and_canonicalizes() {
        let cases = [
            (p(""), p("")),
            (format!("pubky{HOST}"), p("")),
            (
                format!("pubky{HOST}/pub/social/v1/profile.json"),
                p("/pub/social/v1/profile.json"),
            ),
            (
                p("/pub/social/v1/posts/0032SSN7Q4EVG"),
                p("/pub/social/v1/posts/0032SSN7Q4EVG"),
            ),
            (p("/pub/日本語/データ"), p("/pub/日本語/データ")),
        ];
        for (input, expected) in cases {
            let got = canonicalize_pubky_uri(&input).unwrap();
            assert_eq!(got, expected, "{input}");
            // Idempotent.
            assert_eq!(canonicalize_pubky_uri(&got).unwrap(), got);
        }
    }

    #[test]
    fn pubky_uri_rejections() {
        let short_host = &HOST[..51];
        for bad in [
            format!("Pubky://{HOST}/pub/social/v1/profile.json"),
            format!("https://{HOST}/pub/social/v1/profile.json"),
            format!("pubky://user@{HOST}/pub/x"),
            format!("pubky://{HOST}:8080/pub/x"),
            format!("pubky://{short_host}"),
            p("/"),
            p("/pub//social"),
            p("/pub/social/v1/posts/../profile.json"),
            p("/pub/social/v1/./x"),
            p("/pub/social/v1/ta%67s/x.json"),
            p("/pub/social/v1/tags/a?b.json"),
            p("/pub/social/v1/tags/a#b.json"),
            p("/pub/social/v1/tags/a b.json"),
            p("/pub/social/v1/tags/a\u{3000}b.json"),
            p("/pub/social/v1/tags/a\u{0009}b.json"),
            p("/pub/social/v1/tags/a\u{0000}b.json"),
            "pubky".to_string(),
            "pubky:".to_string(),
            "pubky://".to_string(),
            String::new(),
        ] {
            assert!(canonicalize_pubky_uri(&bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn web_gate_vectors() {
        assert_eq!(
            canonicalize_web_uri(" https://example.com "),
            Ok("https://example.com".into())
        );
        // Both spellings are accepted and distinct: the documented fork.
        assert_eq!(
            canonicalize_web_uri("http://x.com"),
            Ok("http://x.com".into())
        );
        assert_eq!(
            canonicalize_web_uri("http://x.com/"),
            Ok("http://x.com/".into())
        );
        // U+200B is not whitespace and survives, pinned.
        assert_eq!(
            canonicalize_web_uri("http://x\u{200B}y"),
            Ok("http://x\u{200B}y".into())
        );
        for bad in [
            "https://exam ple.com",
            "https://x\u{0009}y",
            "HTTPS://x.com",
            "https://",
            "https:///path",
            "https://?q=1",
            "https://#frag",
            "ftp://x",
            "",
        ] {
            assert!(canonicalize_web_uri(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn target_dispatch() {
        // Untrimmed on purpose: a leading space defeats dispatch.
        assert!(canonicalize_target(" https://x.com").is_err());
        assert_eq!(canonicalize_target(&format!("pubky{HOST}")), Ok(p("")));
        // Any other scheme rejects at this stage; the universal third arm comes with the
        // reference-tier validators and flips this assert.
        assert!(canonicalize_target("ipfs://x").is_err());
        assert!(canonicalize_target("nostr:abc").is_err());
        // Over the reference cap in code points.
        let long = p(&format!("/pub/{}", "a".repeat(1100)));
        assert!(canonicalize_pubky_uri(&long).is_ok());
        assert!(canonicalize_target(&long).is_err());
    }

    #[test]
    fn test_canonicalize_external_uri() {
        for (raw, want) in [
            ("nostr:nevent1abc", "nostr:nevent1abc"),
            ("geo:1,2", "geo:1,2"),
            ("IPFS://x", "ipfs://x"),
            ("did:key:z6Mk", "did:key:z6Mk"),
            ("magnet:?xt=y", "magnet:?xt=y"),
            (" ftp://x/y ", "ftp://x/y"),
        ] {
            let got = canonicalize_external_uri(raw).unwrap();
            assert_eq!(got, want, "{raw}");
            assert_eq!(
                canonicalize_external_uri(&got),
                Ok(got.clone()),
                "idempotent {raw}"
            );
        }
        for bad in [
            "",
            ":x",
            "x:",
            "1abc:x",
            "a b:c",
            "nostr:nev\tent",
            "http://x",
            "HTTPS://x",
            "pubky://x",
            "PUBKY:x",
            "nocolon",
        ] {
            assert!(canonicalize_external_uri(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn test_canonicalize_universal_dispatch() {
        assert_eq!(canonicalize_universal(&format!("pubky{HOST}")), Ok(p("")));
        assert_eq!(
            canonicalize_universal("https://x.com/a?b"),
            Ok("https://x.com/a?b".into())
        );
        assert_eq!(canonicalize_universal("Nostr:abc"), Ok("nostr:abc".into()));
        // A pubky-prefixed value never falls through to the external arm
        assert!(canonicalize_universal("pubkyjunk:abc").is_err());
        assert!(canonicalize_universal("https://?q").is_err());
        let long = format!(
            "nostr:{}",
            "a".repeat(VALIDATION_LIMITS.reference_uri_max_length)
        );
        assert!(canonicalize_universal(&long).is_err());
    }
}
