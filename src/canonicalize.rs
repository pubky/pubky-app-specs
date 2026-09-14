//! The engine-free URI canonicalizers. The parser and the collection item check route through
//! [`canonicalize_pubky_uri`] today; the model reference fields still sit on their v0 `url::Url`
//! validation and move here with their own changes, field by field, because re-wiring a hashed
//! field re-ids data. No `url::Url` here or in the parser: an engine parser repairs junk into
//! acceptance (userinfo stripped, `..` collapsed, query and fragment ignored) and its behavior
//! cannot be pinned across versions.

use crate::common::{ascii_fold, code_point_len, frozen_trim, is_frozen_whitespace};
use crate::limits::VALIDATION_LIMITS;
use crate::traits::{Root, ValidationCtx};
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

/// The per-field scheme sets of the reference tier. An API argument, never a wire value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AllowedSchemes {
    /// `lock`, and collection items until they move to the universal tier
    PubkyOnly,
    /// `attachments[].uri`, cover images, `user.image`
    PubkyHttpHttps,
    /// `user.links[].url`
    HttpHttps,
    /// `parent`, `embed`, `tag.uri`, bookmark target: any scheme through the external gate
    Universal,
}

impl AllowedSchemes {
    fn describe(self) -> &'static str {
        match self {
            AllowedSchemes::PubkyOnly => "pubky",
            AllowedSchemes::PubkyHttpHttps => "pubky or web",
            AllowedSchemes::HttpHttps => "web",
            AllowedSchemes::Universal => "",
        }
    }
}

/// The one reference gate: scheme dispatch under `schemes`, canonical form, the cap on that
/// form, then for pubky values the root rule (a pub-rooted object never references a priv
/// URI), the ownership rule (a priv URI whose host is not the author resolves for nobody, so it
/// is invalid in any object) and the versionless rule (a social post reference names the
/// logical post, never a stored version). `owner: None` skips only the ownership rule: plain
/// `validate(id, ctx)` has no author in scope, so builders and ingest pass `Some`. Returns the
/// canonical form.
pub fn validate_reference(
    uri: &str,
    schemes: AllowedSchemes,
    max_code_points: usize,
    ctx: &ValidationCtx,
    owner: Option<&PubkyId>,
) -> Result<String, String> {
    use AllowedSchemes::*;
    let shape = || {
        let what = schemes.describe();
        let sep = if what.is_empty() { "" } else { " " };
        format!(
            "must be a canonical{sep}{what} URI of at most {max_code_points} code points: {uri}"
        )
    };
    let is_pubky = uri.starts_with("pubky");
    let canonical = if is_pubky {
        if schemes == HttpHttps {
            return Err(shape());
        }
        canonicalize_pubky_uri(uri).map_err(|_| shape())?
    } else if uri.starts_with("http://") || uri.starts_with("https://") {
        if schemes == PubkyOnly {
            return Err(shape());
        }
        canonicalize_web_uri(uri).map_err(|_| shape())?
    } else {
        if schemes != Universal {
            return Err(shape());
        }
        canonicalize_external_uri(uri).map_err(|_| shape())?
    };
    if code_point_len(&canonical) > max_code_points {
        return Err(shape());
    }
    if !is_pubky {
        return Ok(canonical);
    }
    // ASCII-safe: the canonicalizer validated the host
    let rest = &canonical["pubky://".len()..];
    let (host, path) = rest.split_once('/').unwrap_or((rest, ""));
    let priv_rooted = path == crate::constants::PRIVATE_ROOT
        || path.starts_with(&format!("{}/", crate::constants::PRIVATE_ROOT));
    if priv_rooted {
        if let Some(owner) = owner {
            if host != owner.as_ref() {
                return Err(format!(
                    "references a private object of another user: {uri}"
                ));
            }
        }
        if ctx.root == Root::Pub {
            return Err(format!(
                "a public object cannot reference a private one: {uri}"
            ));
        }
    }
    if let Ok(parsed) = crate::ParsedUri::try_from(canonical.as_str()) {
        if let crate::Resource::Post {
            version: Some(_), ..
        } = parsed.resource
        {
            return Err(format!("a post reference must be versionless: {uri}"));
        }
    }
    Ok(canonical)
}

/// A stored reference is the fixed point of its own canonical spelling: nothing rewrites it on
/// the way in or out, so the SDK short form and any padding reject here.
pub(crate) fn checked(
    field: &str,
    uri: &str,
    schemes: AllowedSchemes,
    max_code_points: usize,
    ctx: &ValidationCtx,
    owner: Option<&PubkyId>,
) -> Result<(), String> {
    match validate_reference(uri, schemes, max_code_points, ctx, owner) {
        Ok(c) if c == uri => Ok(()),
        Ok(_) => Err(format!(
            "Validation Error: {field} must be spelled in canonical form: {uri}"
        )),
        Err(e) => Err(format!("Validation Error: {field} {e}")),
    }
}

/// A reference to a post: public, versionless, and a fixed point of the parser's own emitter
/// (which rejects the short form). Collection items use it until they join the universal tier.
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

    fn ctx(root: Root) -> ValidationCtx {
        ValidationCtx { root }
    }

    #[test]
    fn reference_scheme_matrix() {
        use AllowedSchemes::*;
        let pk = p("/pub/social/v1/posts/0032SSN7Q4EVG");
        let web = "https://x.com/a";
        let ext = "ipfs://bafy";
        let bad = "https://?q";
        let max = VALIDATION_LIMITS.reference_uri_max_length;
        let ok = |s: AllowedSchemes, u: &str| {
            validate_reference(u, s, max, &ctx(Root::Pub), None).is_ok()
        };
        for (set, want) in [
            (PubkyOnly, [true, false, false, false]),
            (PubkyHttpHttps, [true, true, false, false]),
            (HttpHttps, [false, true, false, false]),
            (Universal, [true, true, true, false]),
        ] {
            let got = [ok(set, &pk), ok(set, web), ok(set, ext), ok(set, bad)];
            assert_eq!(got, want, "{set:?}");
        }
    }

    #[test]
    fn reference_cap_is_measured_on_the_canonical_form() {
        let max = 60 + HOST.len();
        // short form: canonical spelling is three code points longer than the input
        let path = "a".repeat(max - "pubky://".len() - HOST.len() - 1);
        let short = format!("pubky{HOST}/{path}");
        let got = validate_reference(
            &short,
            AllowedSchemes::PubkyOnly,
            max,
            &ctx(Root::Pub),
            None,
        );
        assert_eq!(got, Ok(format!("pubky://{HOST}/{path}")));
        let short = format!("pubky{HOST}/{path}a");
        assert!(validate_reference(
            &short,
            AllowedSchemes::PubkyOnly,
            max,
            &ctx(Root::Pub),
            None
        )
        .is_err());
    }

    #[test]
    fn reference_root_rule() {
        let max = VALIDATION_LIMITS.reference_uri_max_length;
        let owner = PubkyId::try_from(HOST).unwrap();
        let private = p("/priv/social/v1/posts/0032SSN7Q4EVG");
        for owner in [None, Some(&owner)] {
            assert!(validate_reference(
                &private,
                AllowedSchemes::Universal,
                max,
                &ctx(Root::Priv),
                owner
            )
            .is_ok());
            let e = validate_reference(
                &private,
                AllowedSchemes::Universal,
                max,
                &ctx(Root::Pub),
                owner,
            )
            .unwrap_err();
            assert!(e.contains("public object"), "{e}");
        }
        let public = p("/pub/social/v1/posts/0032SSN7Q4EVG");
        assert!(validate_reference(
            &public,
            AllowedSchemes::Universal,
            max,
            &ctx(Root::Pub),
            None
        )
        .is_ok());
        assert!(validate_reference(
            &public,
            AllowedSchemes::Universal,
            max,
            &ctx(Root::Priv),
            None
        )
        .is_ok());
        // bare host is not priv-rooted; the priv root itself is
        assert!(validate_reference(
            &p(""),
            AllowedSchemes::Universal,
            max,
            &ctx(Root::Pub),
            None
        )
        .is_ok());
        assert!(validate_reference(
            &p("/priv"),
            AllowedSchemes::Universal,
            max,
            &ctx(Root::Pub),
            None
        )
        .is_err());
        assert!(validate_reference(
            &p("/private/x"),
            AllowedSchemes::Universal,
            max,
            &ctx(Root::Pub),
            None
        )
        .is_ok());
    }

    #[test]
    fn reference_ownership_rule() {
        let max = VALIDATION_LIMITS.reference_uri_max_length;
        let other =
            PubkyId::try_from("8pinxxgqs41n4aididenw5apqp1urfmzdztr8jt4abrkdn435ewo").unwrap();
        let private = p("/priv/social/v1/posts/0032SSN7Q4EVG");
        for root in [Root::Pub, Root::Priv] {
            let e = validate_reference(
                &private,
                AllowedSchemes::Universal,
                max,
                &ctx(root),
                Some(&other),
            )
            .unwrap_err();
            assert!(e.contains("another user"), "{e}");
        }
        assert!(validate_reference(
            &private,
            AllowedSchemes::Universal,
            max,
            &ctx(Root::Priv),
            None
        )
        .is_ok());
    }

    #[test]
    fn reference_post_must_be_versionless() {
        let max = VALIDATION_LIMITS.reference_uri_max_length;
        let versioned = p("/pub/social/v1/posts/0032SSN7Q4EVG/0032SSN7Q4EVG.json");
        for set in [AllowedSchemes::Universal, AllowedSchemes::PubkyOnly] {
            let e = validate_reference(&versioned, set, max, &ctx(Root::Pub), None).unwrap_err();
            assert!(e.contains("versionless"), "{e}");
        }
        assert!(validate_reference(
            &p("/pub/social/v1/posts/0032SSN7Q4EVG"),
            AllowedSchemes::Universal,
            max,
            &ctx(Root::Pub),
            None
        )
        .is_ok());
        // a file path carries no version and is not a post
        assert!(validate_reference(
            &p("/pub/social/v1/files/0034A0X7NJ52G"),
            AllowedSchemes::Universal,
            max,
            &ctx(Root::Pub),
            None
        )
        .is_ok());
    }

    #[test]
    fn checked_requires_the_fixed_point() {
        let max = VALIDATION_LIMITS.reference_uri_max_length;
        assert!(checked(
            "f",
            &p(""),
            AllowedSchemes::PubkyOnly,
            max,
            &ctx(Root::Pub),
            None
        )
        .is_ok());
        let e = checked(
            "f",
            &format!("pubky{HOST}"),
            AllowedSchemes::PubkyOnly,
            max,
            &ctx(Root::Pub),
            None,
        )
        .unwrap_err();
        assert!(e.starts_with("Validation Error: f must be spelled"), "{e}");
        let e = checked(
            "f",
            "IPFS://x",
            AllowedSchemes::Universal,
            max,
            &ctx(Root::Pub),
            None,
        )
        .unwrap_err();
        assert!(e.contains("spelled"), "{e}");
        let e = checked(
            "f",
            "ipfs://x",
            AllowedSchemes::PubkyOnly,
            max,
            &ctx(Root::Pub),
            None,
        )
        .unwrap_err();
        assert!(
            e.starts_with("Validation Error: f must be a canonical pubky URI"),
            "{e}"
        );
    }
}
