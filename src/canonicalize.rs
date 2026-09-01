//! The engine-free URI canonicalizers. The parser and the collection item check route through
//! [`canonicalize_pubky_uri`] today; the model reference fields still sit on their v0 `url::Url`
//! validation and move here with their own changes, field by field, because re-wiring a hashed
//! field re-ids data. No `url::Url` here or in the parser: an engine parser repairs junk into
//! acceptance (userinfo stripped, `..` collapsed, query and fragment ignored) and its behavior
//! cannot be pinned across versions.

use crate::common::{code_point_len, frozen_trim, is_frozen_whitespace};
use crate::limits::VALIDATION_LIMITS;
use crate::types::PubkyId;

/// One fold-point for every pubky URI. Accepts the full form `pubky://<pk>[/<path>]` and the
/// SDK short form `pubky<pk>[/<path>]`; the canonical output is always the full form.
/// Idempotent: canonicalizing the output returns it unchanged.
// The error is deliberately unit: every rejection is the same "not a pubky URI" verdict, and
// the callers classify, they do not report.
#[allow(clippy::result_unit_err)]
pub fn canonicalize_pubky_uri(raw: &str) -> Result<String, ()> {
    // Scheme, case-sensitive. starts_with on an ASCII prefix keeps the later slicing safe.
    let rest = if let Some(r) = raw.strip_prefix("pubky://") {
        r
    } else if let Some(r) = raw.strip_prefix("pubky") {
        r
    } else {
        return Err(());
    };
    // Host: up to the first '/'. No userinfo, no port, then a canonical PubkyId.
    let (host, path) = match rest.find('/') {
        Some(i) => (&rest[..i], Some(&rest[i + 1..])),
        None => (rest, None),
    };
    if host.contains('@') || host.contains(':') || PubkyId::try_from(host).is_err() {
        return Err(());
    }
    let Some(path) = path else {
        // A bare host is a user reference and is canonical.
        return Ok(["pubky://", host].concat());
    };
    // Segments: no empty segment (kills `//`, leading and trailing slashes), no `.` or `..`,
    // and nowhere a `%`, `?`, `#`, an ASCII control, or a frozen-whitespace code point.
    // Everything else, including non-ASCII, passes: foreign apps may use it.
    for seg in path.split('/') {
        if seg.is_empty() || seg == "." || seg == ".." {
            return Err(());
        }
        for c in seg.chars() {
            if c == '%' || c == '?' || c == '#' || c.is_ascii_control() || is_frozen_whitespace(c) {
                return Err(());
            }
        }
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
    match after {
        Some(rest) if rest.chars().next().is_some_and(|c| c != '/') => Ok(s.to_string()),
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
}
