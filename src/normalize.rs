//! The one normalization every consumer shares, so an indexer, a migrator and a client
//! cannot disagree about which stored paths are the same object.

use crate::constants::{PRIVATE_ROOT, PUBLIC_ROOT, SOCIAL_NAMESPACE};
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

/// Resource names that are a whole object on their own. They also accept a leaf, so an
/// epoch that grows one keys without a change here.
const LEAF_SEGMENTS: &[&str] = &["last_read", "settings"];

/// `true` for `v` followed by at least one digit and nothing else.
fn is_epoch_segment(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() >= 2 && b[0] == b'v' && b[1..].iter().all(u8::is_ascii_digit)
}

fn strip_json(leaf: &str) -> &str {
    leaf.strip_suffix(".json").unwrap_or(leaf)
}

/// From an owner-relative path (`pub/social/v1/posts/X/Y.json`, `pub/pubky.app/posts/X`,
/// `priv/social/v1/files/H.png`, ...) to the resource-relative key that every epoch spelling
/// of one object shares. `None` for a path that is not a social object under any epoch.
///
/// The key drops the root, so a private draft and its published copy are one object, and it
/// drops the whole post version leaf, label included, so every edit of a post is one row.
/// No id is validated here: the key comes from a path the ingest already accepted. Pure
/// string work, no parse, so a later epoch keys its leaves without touching this.
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
        if !is_epoch_segment(version) {
            return None;
        }
        (false, rest)
    } else {
        return None;
    };

    // The resource segment and everything after it. A post leaf keeps its own slash.
    let (segment, leaf) = match rest.split_once('/') {
        Some((s, l)) => (s, Some(l)),
        None => (rest, None),
    };
    let leaf = leaf.filter(|l| !l.is_empty());

    let key = match (segment, leaf) {
        ("posts", Some(leaf)) => {
            let id = leaf.split('/').next().unwrap_or(leaf);
            format!("posts/{id}")
        }
        ("files", Some(leaf)) => {
            if legacy {
                // The v0 metadata object names the bytes; only its `src` completes the key.
                return Some(StableId::NeedsDeref {
                    tsid: leaf.to_string(),
                });
            }
            format!("files/{}", strip_media_ext(leaf))
        }
        // v0 kept the bytes under `blobs/` and their metadata under `files/`; the bytes are
        // the v1 media object, so a blob id keys straight onto it. Ingest never writes a
        // `blobs/` path under a social epoch, the segment rule simply does not ask.
        ("blobs", Some(leaf)) => format!("files/{leaf}"),
        (seg, Some(leaf)) if ID_SEGMENTS.contains(&seg) || LEAF_SEGMENTS.contains(&seg) => {
            format!("{seg}/{}", strip_json(leaf))
        }
        ("profile.json", None) => "profile".to_string(),
        (seg, None) if LEAF_SEGMENTS.contains(&strip_json(seg)) => strip_json(seg).to_string(),
        _ => return None,
    };
    Some(StableId::Key(key))
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
            "pub/social/v7/posts/0RDX5H0000000",
            "priv/social/v7/posts/0RDX5H0000000",
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
            ("pub/social/v1/last_read.json", "last_read"),
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
}
