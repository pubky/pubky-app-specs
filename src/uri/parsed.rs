use crate::canonicalize::canonicalize_pubky_uri;
use crate::common::{validate_hash_id_format, validate_timestamp_id_format};
use crate::constants::{
    epoch_segment, social_path, PRIVATE_ROOT, PROTOCOL, PUBLIC_ROOT, SOCIAL_NAMESPACE,
};
use crate::models::user::PubkySocialUser;
use crate::traits::{HasPath, Root};
use crate::types::PubkyId;
use crate::uri::resource::Resource;
use crate::VALIDATION_LIMITS;
use serde::{Deserialize, Serialize};

/// Which root a parsed path lives under.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Private,
}

impl Visibility {
    pub fn root(&self) -> Root {
        match self {
            Visibility::Public => Root::Pub,
            Visibility::Private => Root::Priv,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedUri {
    pub user_id: PubkyId,
    pub visibility: Visibility,
    pub resource: Resource,
}

impl ParsedUri {
    /// Rebuilds the canonical URI string. Round-trips every parser-produced value; `Unknown`
    /// and `UnsupportedVersion` cannot be spelled honestly and return `Err`.
    pub fn try_to_uri_str(&self) -> Result<String, String> {
        let root = self.visibility.root();
        let leaf = match &self.resource {
            Resource::User => PubkySocialUser::PATH_SEGMENT.to_string(),
            Resource::Post { id, version, label } => match (version, label) {
                (None, _) => format!("posts/{id}"),
                (Some(v), None) => format!("posts/{id}/{v}.json"),
                (Some(v), Some(l)) => format!("posts/{id}/{v}-{l}.json"),
            },
            Resource::Follow(pk) => format!("follows/{pk}.json"),
            Resource::Mute(pk) => format!("mutes/{pk}.json"),
            Resource::Bookmark(id) => format!("bookmarks/{id}.json"),
            Resource::Tag(id) => format!("tags/{id}.json"),
            Resource::File(raw) => format!("files/{raw}"),
            Resource::Blob(id) => format!("blobs/{id}"),
            Resource::Feed(id) => format!("feeds/{id}.json"),
            Resource::Foreign {
                namespace,
                version,
                rest,
            } => {
                let mut segs = vec![namespace.clone()];
                if let Some(v) = version {
                    segs.push(v.clone());
                }
                segs.extend(rest.iter().cloned());
                let path = ["/", root.segment(), "/", &segs.join("/")].concat();
                return Ok([PROTOCOL, self.user_id.as_ref(), &path].concat());
            }
            Resource::UnsupportedVersion { .. } => {
                return Err("an unsupported epoch does not carry its path".to_string())
            }
            Resource::Unknown => return Err("Cannot convert Unknown resource to URI".to_string()),
        };
        let path = social_path(root, &leaf);
        Ok([PROTOCOL, self.user_id.as_ref(), &path].concat())
    }
}

fn is_epoch_segment(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() >= 2 && b[0] == b'v' && b[1..].iter().all(u8::is_ascii_digit)
}

/// The optional readable tail of a post version leaf: 1 to 64 chars of `[a-z0-9-]`.
fn is_valid_label(l: &str) -> bool {
    !l.is_empty()
        && l.len() <= VALIDATION_LIMITS.post_slug_max_length
        && l.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Splits a post version leaf: strips `.json`, then splits at the FIRST `-`. The head must be
/// a canonical TimestampId; the optional tail is the label.
fn parse_version_leaf(leaf: &str) -> Option<(String, Option<String>)> {
    let stem = leaf.strip_suffix(".json")?;
    let (version, label) = match stem.split_once('-') {
        Some((v, l)) if is_valid_label(l) => (v, Some(l.to_string())),
        Some(_) => return None,
        None => (stem, None),
    };
    validate_timestamp_id_format(version).ok()?;
    Some((version.to_string(), label))
}

fn dispatch(root: Root, rest: &[&str]) -> Resource {
    match (root, rest) {
        (Root::Pub, ["profile.json"]) => Resource::User,
        // The versionless reference takes no extension: posts/{id}.json is Unknown.
        (_, ["posts", id]) if validate_timestamp_id_format(id).is_ok() => Resource::Post {
            id: (*id).into(),
            version: None,
            label: None,
        },
        (_, ["posts", id, leaf]) if validate_timestamp_id_format(id).is_ok() => {
            match parse_version_leaf(leaf) {
                Some((version, label)) => Resource::Post {
                    id: (*id).into(),
                    version: Some(version),
                    label,
                },
                None => Resource::Unknown,
            }
        }
        // v0-shaped media at this stage: a metadata JSON keyed by TimestampId. The media
        // collapse replaces this arm with the hash + known-extension strip.
        (_, ["files", leaf]) => match leaf.strip_suffix(".json") {
            Some(id) if validate_timestamp_id_format(id).is_ok() => Resource::File((*leaf).into()),
            _ => Resource::Unknown,
        },
        (Root::Pub, ["blobs", id]) if validate_hash_id_format(id).is_ok() => {
            Resource::Blob((*id).into())
        }
        (_, ["feeds", leaf]) => match leaf.strip_suffix(".json") {
            Some(id) if validate_hash_id_format(id).is_ok() => Resource::Feed(id.into()),
            _ => Resource::Unknown,
        },
        (Root::Pub, ["tags", leaf]) => match leaf.strip_suffix(".json") {
            Some(id) if validate_hash_id_format(id).is_ok() => Resource::Tag(id.into()),
            _ => Resource::Unknown,
        },
        (Root::Pub, ["follows", leaf]) => match leaf.strip_suffix(".json") {
            Some(pk) => match PubkyId::try_from(pk) {
                Ok(pk) => Resource::Follow(pk),
                Err(_) => Resource::Unknown,
            },
            _ => Resource::Unknown,
        },
        (Root::Pub, ["mutes", leaf]) => match leaf.strip_suffix(".json") {
            Some(pk) => match PubkyId::try_from(pk) {
                Ok(pk) => Resource::Mute(pk),
                Err(_) => Resource::Unknown,
            },
            _ => Resource::Unknown,
        },
        // The bookmark id is still the v0 HashId form here; the private move re-keys it.
        (Root::Pub, ["bookmarks", leaf]) => match leaf.strip_suffix(".json") {
            Some(id) if validate_hash_id_format(id).is_ok() => Resource::Bookmark(id.into()),
            _ => Resource::Unknown,
        },
        // Wrong root, missing or extra segments, `_`-prefixed leaves, unrecognized names.
        _ => Resource::Unknown,
    }
}

impl TryFrom<&str> for ParsedUri {
    type Error = String;

    fn try_from(uri: &str) -> Result<Self, Self::Error> {
        let canonical =
            canonicalize_pubky_uri(uri).map_err(|_| format!("Not a canonical pubky URI: {uri}"))?;
        // Offsets are safe: the prefix is ASCII and the host was validated as a PubkyId.
        let rest = &canonical["pubky://".len()..];
        let (host, path) = match rest.find('/') {
            Some(i) => (&rest[..i], Some(&rest[i + 1..])),
            None => (rest, None),
        };
        let user_id = PubkyId::try_from(host).expect("host was validated by the canonicalizer");
        let Some(path) = path else {
            return Ok(ParsedUri {
                user_id,
                visibility: Visibility::Public,
                resource: Resource::User,
            });
        };
        let segments: Vec<&str> = path.split('/').collect();
        let visibility = match segments.first() {
            Some(&s) if s == PUBLIC_ROOT => Visibility::Public,
            Some(&s) if s == PRIVATE_ROOT => Visibility::Private,
            _ => return Err(format!("Unknown root in URI: {uri}")),
        };
        let resource = match segments.get(1) {
            None => Resource::Unknown,
            Some(&ns) if ns != SOCIAL_NAMESPACE => Resource::Foreign {
                namespace: ns.to_string(),
                version: segments.get(2).map(|s| s.to_string()),
                rest: segments
                    .get(3..)
                    .unwrap_or(&[])
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            },
            Some(_) => match segments.get(2) {
                None => Resource::Unknown,
                Some(&e) if e == epoch_segment() => {
                    dispatch(visibility.root(), segments.get(3..).unwrap_or(&[]))
                }
                Some(&e) if is_epoch_segment(e) => Resource::UnsupportedVersion {
                    version: e.to_string(),
                },
                Some(_) => Resource::Unknown,
            },
        };
        Ok(ParsedUri {
            user_id,
            visibility,
            resource,
        })
    }
}

impl TryFrom<String> for ParsedUri {
    type Error = String;

    fn try_from(uri: String) -> Result<Self, Self::Error> {
        ParsedUri::try_from(uri.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{post::PubkySocialPost, user::PubkySocialUser};
    use crate::traits::{HasIdPath, HasPath, PUB_CTX};
    use crate::uri::builders::{base_uri_builder, post_uri_builder};
    use crate::PubkySocialObject;

    const HOST: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";
    const TS: &str = "0032SSN7Q4EVG";
    const TS2: &str = "0034A0X7NJ52G";
    const H26: &str = "8Z8CWH8NVYQY39ZEBFGKQWWEKG";

    fn p(path: &str) -> String {
        format!("pubky://{HOST}{path}")
    }

    fn pk() -> PubkyId {
        PubkyId::try_from(HOST).unwrap()
    }

    fn post(id: &str, version: Option<&str>, label: Option<&str>) -> Resource {
        Resource::Post {
            id: id.into(),
            version: version.map(Into::into),
            label: label.map(Into::into),
        }
    }

    /// The classification table. `None` expected = hard error.
    #[rustfmt::skip]
    fn vectors() -> Vec<(String, Option<(Visibility, Resource)>)> {
        use Visibility::{Private, Public};
        let label64 = "a".repeat(64);
        let label65 = "a".repeat(65);
        vec![
            // Accepted forms and the post family
            (p(""), Some((Public, Resource::User))),
            (format!("pubky{HOST}"), Some((Public, Resource::User))),
            (format!("pubky{HOST}/pub/social/v1/profile.json"), Some((Public, Resource::User))),
            (p("/pub/social/v1/profile.json"), Some((Public, Resource::User))),
            (p(&format!("/pub/social/v1/posts/{TS}")), Some((Public, post(TS, None, None)))),
            (p(&format!("/pub/social/v1/posts/{TS}/{TS}.json")), Some((Public, post(TS, Some(TS), None)))),
            (p(&format!("/priv/social/v1/posts/{TS}/{TS2}.json")), Some((Private, post(TS, Some(TS2), None)))),
            (p(&format!("/pub/social/v1/posts/{TS}.json")), Some((Public, Resource::Unknown))),
            (p(&format!("/pub/social/v1/posts/{TS}/{TS2}")), Some((Public, Resource::Unknown))),
            (p("/pub/social/v1/posts/0032ssn7q4evg"), Some((Public, Resource::Unknown))),
            (p("/pub/social/v1/posts/O032SSN7Q4EVG"), Some((Public, Resource::Unknown))),
            (p(&format!("/pub/social/v1/posts/{TS}/{TS}.json/x")), Some((Public, Resource::Unknown))),
            // The optional version label
            (p(&format!("/pub/social/v1/posts/{TS}/{TS2}-hello-world.json")), Some((Public, post(TS, Some(TS2), Some("hello-world"))))),
            (p(&format!("/pub/social/v1/posts/{TS}/{TS2}-a-b.json")), Some((Public, post(TS, Some(TS2), Some("a-b"))))),
            (p(&format!("/pub/social/v1/posts/{TS}/{TS}-hello.json")), Some((Public, post(TS, Some(TS), Some("hello"))))),
            (p(&format!("/pub/social/v1/posts/{TS}/{TS2}-Hello.json")), Some((Public, Resource::Unknown))),
            (p(&format!("/pub/social/v1/posts/{TS}/{TS2}-.json")), Some((Public, Resource::Unknown))),
            (p(&format!("/pub/social/v1/posts/{TS}/{TS2}-{label64}.json")), Some((Public, post(TS, Some(TS2), Some(&label64))))),
            (p(&format!("/pub/social/v1/posts/{TS}/{TS2}-{label65}.json")), Some((Public, Resource::Unknown))),
            (p(&format!("/pub/social/v1/posts/{TS}/{TS2}-h_llo.json")), Some((Public, Resource::Unknown))),
            (p(&format!("/pub/social/v1/posts/{TS}/{TS2}-héllo.json")), Some((Public, Resource::Unknown))),
            // Tags, follows, mutes, bookmarks
            (p(&format!("/pub/social/v1/tags/{H26}.json")), Some((Public, Resource::Tag(H26.into())))),
            (p(&format!("/pub/social/v1/tags/{H26}")), Some((Public, Resource::Unknown))),
            (p(&format!("/priv/social/v1/tags/{H26}.json")), Some((Private, Resource::Unknown))),
            (p("/pub/social/v1/tags/8Z8CWH8NVYQY39ZEBFGKQWWEK.json"), Some((Public, Resource::Unknown))),
            (p(&format!("/pub/social/v1/follows/{HOST}.json")), Some((Public, Resource::Follow(pk())))),
            (p(&format!("/pub/social/v1/follows/{}.json", HOST.to_uppercase())), Some((Public, Resource::Unknown))),
            (p(&format!("/pub/social/v1/mutes/{HOST}.json")), Some((Public, Resource::Mute(pk())))),
            (p(&format!("/pub/social/v1/bookmarks/{H26}.json")), Some((Public, Resource::Bookmark(H26.into())))),
            // v0-shaped media under the epoch (collapsed later)
            (p(&format!("/pub/social/v1/files/{TS}.json")), Some((Public, Resource::File(format!("{TS}.json"))))),
            (p(&format!("/priv/social/v1/files/{TS}.json")), Some((Private, Resource::File(format!("{TS}.json"))))),
            (p(&format!("/pub/social/v1/blobs/{H26}")), Some((Public, Resource::Blob(H26.into())))),
            (p(&format!("/priv/social/v1/blobs/{H26}")), Some((Private, Resource::Unknown))),
            // Feeds are dual-root already
            (p(&format!("/pub/social/v1/feeds/{H26}.json")), Some((Public, Resource::Feed(H26.into())))),
            (p(&format!("/priv/social/v1/feeds/{H26}.json")), Some((Private, Resource::Feed(H26.into())))),
            // Gone or reserved leaves
            (p("/pub/social/v1/last_read.json"), Some((Public, Resource::Unknown))),
            (p("/priv/social/v1/settings.json"), Some((Private, Resource::Unknown))),
            (p("/priv/social/v1/_migrated.json"), Some((Private, Resource::Unknown))),
            // Epoch handling
            (p(&format!("/pub/social/v2/posts/{TS}")), Some((Public, Resource::UnsupportedVersion { version: "v2".into() }))),
            (p(&format!("/pub/social/V1/posts/{TS}")), Some((Public, Resource::Unknown))),
            (p("/pub/social/version2/x"), Some((Public, Resource::Unknown))),
            (p("/pub/social"), Some((Public, Resource::Unknown))),
            (p("/pub"), Some((Public, Resource::Unknown))),
            // Foreign namespaces
            (p(&format!("/pub/pubky.app/posts/{TS}")), Some((Public, Resource::Foreign { namespace: "pubky.app".into(), version: Some("posts".into()), rest: vec![TS.into()] }))),
            (p("/priv/app.pubky/v1/settings.json"), Some((Private, Resource::Foreign { namespace: "app.pubky".into(), version: Some("v1".into()), rest: vec!["settings.json".into()] }))),
            (p("/priv/app.pubky/v1/last_read.json"), Some((Private, Resource::Foreign { namespace: "app.pubky".into(), version: Some("v1".into()), rest: vec!["last_read.json".into()] }))),
            (p("/pub/日本語/データ"), Some((Public, Resource::Foreign { namespace: "日本語".into(), version: Some("データ".into()), rest: vec![] }))),
            // Hard errors
            (p("/dav/social/v1/profile.json"), None),
            (format!("Pubky://{HOST}/pub/social/v1/profile.json"), None),
            (format!("https://{HOST}/pub/social/v1/profile.json"), None),
            (format!("pubky://user@{HOST}/pub/x"), None),
            (format!("pubky://{HOST}:8080/pub/x"), None),
            (format!("pubky://{}", &HOST[..51]), None),
            (p("/"), None),
            (p("/pub//social/v1"), None),
            (p("/pub/social/v1/posts/../profile.json"), None),
            (p("/pub/social/v1/ta%67s/x.json"), None),
            (p("/pub/social/v1/tags/a?b.json"), None),
            (p("/pub/social/v1/tags/a#b.json"), None),
            (p("/pub/social/v1/tags/a b.json"), None),
            (p("/pub/social/v1/tags/a\u{3000}b.json"), None),
            (p("/pub/social/v1/tags/a\u{0009}b.json"), None),
        ]
    }

    #[test]
    fn classification_table() {
        for (input, expected) in vectors() {
            let got = ParsedUri::try_from(input.as_str());
            match expected {
                None => assert!(got.is_err(), "expected a hard error: {input}"),
                Some((vis, res)) => {
                    let parsed = got.unwrap_or_else(|e| panic!("{input}: {e}"));
                    assert_eq!(parsed.visibility, vis, "{input}");
                    assert_eq!(parsed.resource, res, "{input}");
                }
            }
        }
    }

    #[test]
    fn round_trips() {
        for (input, expected) in vectors() {
            let Some((_, res)) = expected else { continue };
            let parsed = ParsedUri::try_from(input.as_str()).unwrap();
            match res {
                Resource::Unknown | Resource::UnsupportedVersion { .. } => {
                    assert!(parsed.try_to_uri_str().is_err(), "{input}");
                }
                Resource::User => {
                    // Bare-host inputs cannot string-round-trip to profile.json; the value must.
                    let emitted = parsed.try_to_uri_str().unwrap();
                    let reparsed = ParsedUri::try_from(emitted.as_str()).unwrap();
                    assert_eq!(reparsed, parsed, "{input}");
                }
                _ => {
                    let canonical = crate::canonicalize_pubky_uri(&input).unwrap();
                    assert_eq!(parsed.try_to_uri_str().unwrap(), canonical, "{input}");
                }
            }
        }
    }

    #[test]
    fn no_panic_on_junk() {
        let long_seg = format!("pubky://{HOST}/{}", "x".repeat(5000));
        for junk in [
            "",
            "pubky",
            "pubky:",
            "pubky://",
            "///",
            "pubkyé",
            "pubky://\u{0000}",
            long_seg.as_str(),
        ] {
            let _ = ParsedUri::try_from(junk);
        }
    }

    #[test]
    fn versionless_reference_is_never_a_stored_object() {
        let err =
            PubkySocialObject::from_resource(&post(TS, None, None), b"{}", &PUB_CTX).unwrap_err();
        assert!(err.contains("versionless"), "{err}");
        let body =
            br#"{"content":"x","kind":"short","parent":null,"embed":null,"attachments":null}"#;
        assert!(
            PubkySocialObject::from_resource(&post(TS, Some(TS), None), body, &PUB_CTX).is_ok()
        );
    }

    #[test]
    fn create_paths_and_builders() {
        assert_eq!(
            PubkySocialUser::create_path(),
            "/pub/social/v1/profile.json"
        );
        assert_eq!(
            PubkySocialPost::create_path(TS),
            format!("/pub/social/v1/posts/{TS}/{TS}.json")
        );
        assert_eq!(
            post_uri_builder(HOST.into(), TS.into()),
            format!("pubky://{HOST}/pub/social/v1/posts/{TS}")
        );
        assert_eq!(
            base_uri_builder(HOST.into()),
            format!("pubky://{HOST}/pub/social/v1/")
        );
    }

    #[test]
    fn display_and_id() {
        assert_eq!(post(TS, Some(TS2), Some("x")).to_string(), "posts");
        assert_eq!(post(TS, Some(TS2), Some("x")).id(), Some(TS.to_string()));
        assert_eq!(
            Resource::File(format!("{TS}.json")).id(),
            Some(TS.to_string())
        );
        assert_eq!(
            Resource::Foreign {
                namespace: "x".into(),
                version: None,
                rest: vec![]
            }
            .to_string(),
            "foreign"
        );
        assert_eq!(
            Resource::UnsupportedVersion {
                version: "v2".into()
            }
            .to_string(),
            "unsupported_version"
        );
        assert_eq!(
            Resource::UnsupportedVersion {
                version: "v2".into()
            }
            .id(),
            None
        );
        assert_eq!(
            pk().to_uri().try_to_uri_str().unwrap(),
            p("/pub/social/v1/profile.json")
        );
    }
}
