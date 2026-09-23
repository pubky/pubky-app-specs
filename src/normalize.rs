//! The one normalization every consumer shares, so an indexer, a migrator and a client
//! cannot disagree about which stored paths are the same object.

use crate::constants::{epoch_segment, PRIVATE_ROOT, PUBLIC_ROOT, SOCIAL_NAMESPACE};
use crate::models::legacy_v0::{ParsedUri, Resource};
use crate::uri::strip_media_ext;

/// The dedup key of a stored object across epochs and roots, or a legacy media reference
/// that needs its v0 File object to complete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StableId {
    Key(String),
    NeedsDeref { tsid: String },
}

/// The v0 namespace, epoch 0: one path segment where a social epoch takes two.
const LEGACY_EPOCH: &str = "pubky.app";

/// Resource segments whose leaf carries the id. An empty or missing leaf is not a stored
/// object, so it has no key.
const ID_SEGMENTS: &[&str] = &["tags", "follows", "mutes", "bookmarks", "feeds"];

/// v0 resources that are a whole object on their own. They key only under `pubky.app`, so
/// the migrator can find them; in v1 they belong to the app, not to this library.
const LEAF_SEGMENTS: &[&str] = &["last_read", "settings"];

fn strip_json(leaf: &str) -> &str {
    leaf.strip_suffix(".json").unwrap_or(leaf)
}

/// `{segment}/{id}`, or `None` when the strip left no id behind, so `tags/.json` and
/// `files/.png` are not stored objects rather than a key with an empty id.
fn keyed(segment: &str, id: &str) -> Option<String> {
    if id.is_empty() {
        return None;
    }
    Some(format!("{segment}/{id}"))
}

/// From an owner-relative path (`pub/social/v1/posts/X/Y.json`, `pub/pubky.app/posts/X`,
/// `priv/social/v1/files/H.png`, ...) to the resource-relative key that every epoch spelling
/// of one object shares. `None` for a path that is not a social object under any epoch.
///
/// The key drops the root, so a private draft and its published copy are one object, and it
/// drops the whole post version leaf, label included, so every edit of a post is one row.
/// No id is validated here: the key comes from a path the ingest already accepted.
///
/// Only `social/v1` keys. A new epoch exists for a change that breaks these rules, a
/// re-pinned id function or a grammar break, so a later epoch adds its own rules here
/// instead of inheriting v1's.
///
/// A leaf is read on the terms of the epoch that wrote it. A post leaf is the version path,
/// so everything from its first segment on is dropped under every epoch. Any other leaf is
/// the id: under `pubky.app` a trailing extra segment is ignored, because the v0 parser
/// ignores it and keys the object anyway, and under a social epoch it makes the path no
/// object at all, because the v1 parser rejects it.
///
/// What collapses across epochs is the path grammar, not the id inside it. `posts`, `files`
/// and `blobs`, `follows`, `mutes` and `profile` carry the same id in both spellings of one
/// object, so the two paths key onto one row. `tags`, `bookmarks`
/// and `feeds` do not: a v0 tag id hashes a target uri that the migration itself respells
/// for social targets, a v0 bookmark id is a hash where the v1 leaf is a filename, and a
/// feed id is re-derived. One migrated tag, bookmark or feed therefore holds two keys, and
/// collapsing those is the indexer's own job, by normalized target or by its own rule.
pub fn stable_id(owner_relative_path: &str) -> Option<StableId> {
    let path = owner_relative_path
        .strip_prefix('/')
        .unwrap_or(owner_relative_path);

    let (root, after_root) = path.split_once('/')?;
    if root != PUBLIC_ROOT && root != PRIVATE_ROOT {
        return None;
    }

    let (namespace, tail) = after_root.split_once('/')?;
    let (legacy, rest) = if namespace == LEGACY_EPOCH {
        (true, tail)
    } else if namespace == SOCIAL_NAMESPACE {
        let (version, rest) = tail.split_once('/')?;
        if version != epoch_segment() {
            return None;
        }
        (false, rest)
    } else {
        return None;
    };

    // The resource segment and everything after it.
    let (segment, leaf) = match rest.split_once('/') {
        Some((s, l)) => (s, Some(l)),
        None => (rest, None),
    };
    let leaf = leaf.filter(|l| !l.is_empty());

    // A post leaf is a version path under every epoch, so it always trims to its first
    // segment. For every other resource the leaf is the id, and the two epochs read an
    // extra segment differently: the v0 parser matches `[resource, id, ..]` and ignores
    // whatever follows, the v1 parser rejects it. Each epoch gets its own answer, or a
    // read-then-key pass drops objects the epoch's own parser accepted.
    let leaf = match leaf {
        Some(l) if segment == "posts" || legacy => match l.split('/').next().unwrap_or(l) {
            "" => return None,
            first => Some(first),
        },
        Some(l) if l.contains('/') => return None,
        none_or_plain => none_or_plain,
    };

    let key = match (segment, leaf) {
        ("posts", Some(id)) => keyed("posts", id)?,
        ("files", Some(leaf)) => {
            if legacy {
                // The v0 metadata object names the bytes; only its `src` completes the key.
                return Some(StableId::NeedsDeref {
                    tsid: leaf.to_string(),
                });
            }
            keyed("files", strip_media_ext(leaf))?
        }
        // v0 kept the bytes under `blobs/` and their metadata under `files/`; the bytes are
        // the v1 media object, so a blob id keys straight onto it. Ingest never writes a
        // `blobs/` path under a social epoch, the segment rule simply does not ask.
        ("blobs", Some(leaf)) => keyed("files", leaf)?,
        (seg, Some(leaf))
            if ID_SEGMENTS.contains(&seg) || (legacy && LEAF_SEGMENTS.contains(&seg)) =>
        {
            keyed(seg, strip_json(leaf))?
        }
        ("profile.json", None) => "profile".to_string(),
        (seg, None) if legacy && LEAF_SEGMENTS.contains(&strip_json(seg)) => {
            strip_json(seg).to_string()
        }
        _ => return None,
    };
    Some(StableId::Key(key))
}

/// Completes a legacy `files/{tsid}` reference through the v0 File object's `src`
/// (`pubky://<pk>/pub/pubky.app/blobs/<hash>`) to `files/<hash>`. `None` when the src is not
/// a legacy blob reference; the caller then keys the reference verbatim, so a dangling or
/// foreign src still resolves for a reader.
///
/// The src is read with the frozen v0 parser, so a reference is completed on exactly the
/// terms v0 accepted it. The tsid does not reach the result: it is in the signature because
/// the two halves of one operation should read as a pair, and because a later policy may
/// need to know which reference it is completing.
pub fn resolve_deref(_tsid: &str, v0_file_src: &str) -> Option<String> {
    match ParsedUri::try_from(v0_file_src).ok()?.resource {
        Resource::Blob(hash) => Some(format!("files/{hash}")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(path: &str) -> Option<String> {
        match stable_id(path) {
            Some(StableId::Key(k)) => Some(k),
            _ => None,
        }
    }

    const HASH: &str = "8Z8CWH8NVYQY39ZEBFGKQWWEKG";

    #[test]
    fn the_contract_table() {
        let cases: &[(&str, &str)] = &[
            (
                "pub/social/v1/posts/0RDX5H0000000/0RDX5J0000002.json",
                "posts/0RDX5H0000000",
            ),
            (
                "pub/social/v1/posts/0RDX5H0000000/0RDX5J0000002-hello-world.json",
                "posts/0RDX5H0000000",
            ),
            ("pub/pubky.app/posts/0RDX5H0000000", "posts/0RDX5H0000000"),
            (
                "priv/social/v1/files/8Z8CWH8NVYQY39ZEBFGKQWWEKG.jpg",
                "files/8Z8CWH8NVYQY39ZEBFGKQWWEKG",
            ),
            (
                "pub/pubky.app/blobs/8Z8CWH8NVYQY39ZEBFGKQWWEKG",
                "files/8Z8CWH8NVYQY39ZEBFGKQWWEKG",
            ),
        ];
        for (path, expected) in cases {
            assert_eq!(key(path).as_deref(), Some(*expected), "{path}");
        }
        assert_eq!(
            stable_id("pub/pubky.app/files/0RDX5H0000000"),
            Some(StableId::NeedsDeref {
                tsid: "0RDX5H0000000".to_string()
            })
        );
    }

    #[test]
    fn the_key_is_root_and_epoch_independent() {
        let one = format!("posts/{}", "0RDX5H0000000");
        for path in [
            "pub/social/v1/posts/0RDX5H0000000",
            "priv/social/v1/posts/0RDX5H0000000",
            "/pub/social/v1/posts/0RDX5H0000000",
            "pub/pubky.app/posts/0RDX5H0000000",
        ] {
            assert_eq!(key(path).as_deref(), Some(one.as_str()), "{path}");
        }
    }

    #[test]
    fn a_blobs_path_under_a_social_epoch_still_keys_as_media() {
        // Ingest never produces this: v1 has no blobs/ resource. The segment rule is
        // epoch-blind on purpose, so the spelling keys rather than dropping on the floor.
        assert_eq!(
            key(&format!("pub/social/v1/blobs/{HASH}")).as_deref(),
            Some(format!("files/{HASH}").as_str())
        );
    }

    #[test]
    fn only_a_closed_set_extension_is_stripped() {
        // The strip set is case-sensitive, matching the parser, which reads `.JPG` as a
        // filename that happens to contain a dot.
        let cases: &[(&str, &str)] = &[
            (
                "8Z8CWH8NVYQY39ZEBFGKQWWEKG.jpg",
                "8Z8CWH8NVYQY39ZEBFGKQWWEKG",
            ),
            ("8Z8CWH8NVYQY39ZEBFGKQWWEKG", "8Z8CWH8NVYQY39ZEBFGKQWWEKG"),
            (
                "8Z8CWH8NVYQY39ZEBFGKQWWEKG.JPG",
                "8Z8CWH8NVYQY39ZEBFGKQWWEKG.JPG",
            ),
            (
                "8Z8CWH8NVYQY39ZEBFGKQWWEKG.tar.gz",
                "8Z8CWH8NVYQY39ZEBFGKQWWEKG.tar.gz",
            ),
            (
                "8Z8CWH8NVYQY39ZEBFGKQWWEKG.tar.zip",
                "8Z8CWH8NVYQY39ZEBFGKQWWEKG.tar",
            ),
        ];
        for (leaf, expected) in cases {
            assert_eq!(
                key(&format!("priv/social/v1/files/{leaf}")).as_deref(),
                Some(format!("files/{expected}").as_str()),
                "{leaf}"
            );
        }
    }

    #[test]
    fn the_remaining_segments_drop_one_trailing_json() {
        let cases: &[(&str, &str)] = &[
            ("pub/social/v1/profile.json", "profile"),
            ("pub/pubky.app/profile.json", "profile"),
            ("pub/social/v1/tags/ABC.json", "tags/ABC"),
            ("pub/pubky.app/tags/ABC", "tags/ABC"),
            ("pub/social/v1/follows/PK.json", "follows/PK"),
            ("priv/social/v1/mutes/PK.json", "mutes/PK"),
            ("priv/social/v1/bookmarks/ABC.json", "bookmarks/ABC"),
            ("priv/social/v1/feeds/ABC.json", "feeds/ABC"),
            ("pub/pubky.app/last_read", "last_read"),
            ("pub/pubky.app/settings.json", "settings"),
        ];
        for (path, expected) in cases {
            assert_eq!(key(path).as_deref(), Some(*expected), "{path}");
        }
    }

    #[test]
    fn a_post_keys_the_same_with_or_without_a_version_leaf() {
        let versionless = key("pub/social/v1/posts/0RDX5H0000000");
        for leaf in [
            "0RDX5J0000002.json",
            "0RDX5J0000002-hello-world.json",
            "0RDX5J0000002",
            "anything/at/all",
        ] {
            assert_eq!(
                key(&format!("pub/social/v1/posts/0RDX5H0000000/{leaf}")),
                versionless,
                "{leaf}"
            );
        }
    }

    const OWNER: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";

    #[test]
    fn a_deref_completes_only_through_a_legacy_blob_src() {
        let tsid = "0032SSN7Q4EVG";
        assert_eq!(
            resolve_deref(tsid, &format!("pubky://{OWNER}/pub/pubky.app/blobs/{HASH}")),
            Some(format!("files/{HASH}"))
        );
        for src in [
            // The owner-relative spelling is not a URI, and v0 never stored one.
            &format!("/pub/pubky.app/blobs/{HASH}"),
            &format!("pub/pubky.app/blobs/{HASH}"),
            // An off-network src: the reference keys verbatim instead.
            "https://example.com/photo.jpg",
            // A v0 File never points into a social epoch.
            &format!("pubky://{OWNER}/pub/social/v1/files/{HASH}.jpg"),
            // A v0 File pointing at another File rather than at bytes.
            &format!("pubky://{OWNER}/pub/pubky.app/files/{tsid}"),
            "not a url",
        ] {
            assert_eq!(resolve_deref(tsid, src), None, "{src}");
        }
    }

    #[test]
    fn a_path_that_is_not_a_social_object_has_no_key() {
        for path in [
            "",
            "/",
            "pub",
            "pub/",
            "pub/social/v1/",
            "www/social/v1/profile.json",           // not a root
            "pub/other.app/posts/0RDX5H0000000",    // not an epoch
            "pub/social/posts/0RDX5H0000000",       // no version segment
            "pub/social/vX/posts/0RDX5H0000000",    // not digits
            "pub/social/v/posts/0RDX5H0000000",     // no digits
            "pub/social/v0/posts/0RDX5H0000000",    // epoch 0 spells itself as pubky.app
            "pub/social/v01/posts/0RDX5H0000000",   // a leading zero is not an epoch
            "pub/social/v2/posts/0RDX5H0000000",    // a later epoch brings its own rules
            "priv/social/v10/posts/0RDX5H0000000",  // a later epoch brings its own rules
            "pub/social/v1/last_read.json",         // app-owned in v1
            "pub/social/v1/settings.json",          // app-owned in v1
            "pub/social/v1/settings/x.json",        // app-owned in v1
            "pub/social/v1/widgets/ABC",            // unknown segment
            "pub/social/v1/posts/",                 // missing leaf
            "pub/social/v1/tags/",                  // missing leaf
            "pub/social/v1/tags",                   // missing leaf
            "pub/pubky.app/files/",                 // missing leaf
            "pub/social/v1/profile",                // the leaf is profile.json
            "pubky://x/pub/social/v1/profile.json", // not owner-relative
        ] {
            assert_eq!(stable_id(path), None, "{path}");
        }
    }

    #[test]
    fn an_empty_id_is_not_an_object() {
        // A key with an empty id would collapse every such path onto one row.
        for path in [
            "pub/social/v1/posts//0RDX5J0000002.json",
            "pub/social/v1/posts//",
            "pub/social/v1/tags/.json",
            "pub/pubky.app/settings/.json",
            "pub/social/v1/files/.png",
            "pub/social/v1/files/.jpg",
        ] {
            assert_eq!(stable_id(path), None, "{path}");
        }
    }

    #[test]
    fn a_post_leaf_is_a_path_under_every_epoch() {
        for root in ["pub/social/v1", "pub/pubky.app"] {
            assert_eq!(
                key(&format!("{root}/posts/0RDX5H0000000/a/b/c")).as_deref(),
                Some("posts/0RDX5H0000000"),
                "{root}"
            );
        }
    }

    #[test]
    fn a_v0_leaf_ignores_an_extra_segment_the_way_the_v0_parser_does() {
        // The v0 parser matches [resource, id, ..] and keys on the id, so dropping these
        // would lose objects that v0 itself accepted.
        assert_eq!(
            stable_id(&format!("pub/pubky.app/files/{HASH}/extra")),
            Some(StableId::NeedsDeref {
                tsid: HASH.to_string()
            })
        );
        let cases: &[(&str, &str)] = &[
            ("pub/pubky.app/tags/ABC/x", "tags/ABC"),
            ("pub/pubky.app/tags/ABC/", "tags/ABC"),
            ("priv/pubky.app/bookmarks/ABC/x/y", "bookmarks/ABC"),
            ("pub/pubky.app/follows/PK/x", "follows/PK"),
            ("pub/pubky.app/blobs/ABC/x", "files/ABC"),
        ];
        for (path, expected) in cases {
            assert_eq!(key(path).as_deref(), Some(*expected), "{path}");
        }
        // An empty first segment is still no object.
        for path in [
            "pub/pubky.app/tags//x",
            "pub/pubky.app/files//extra",
            "pub/pubky.app/posts//x",
        ] {
            assert_eq!(stable_id(path), None, "{path}");
        }
    }

    #[test]
    fn a_social_epoch_leaf_may_not_be_a_path() {
        // The v1 parser rejects the extra segment, so nothing was ever stored there.
        for path in [
            &format!("pub/social/v1/tags/{HASH}/"),
            &format!("pub/social/v1/files/{HASH}.png/x"),
            &format!("pub/social/v1/blobs/{HASH}/x"),
            &format!("pub/social/v1/follows/{HASH}/x"),
            "pub/social/v1/profile.json/x",
            "pub/pubky.app/profile.json/x",
        ] {
            assert_eq!(stable_id(path), None, "{path}");
        }
    }
}
