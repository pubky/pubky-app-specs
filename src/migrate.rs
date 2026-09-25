//! The v0 to v1 transforms: one owner's `pub/pubky.app/` objects in, the `social/v1` objects
//! to write out.
//!
//! Every transform is a pure function over bytes. A v0 object is read as a JSON value, never
//! through the derived v0 structs, with three rules: a duplicate key keeps its last value, an
//! integer field must be an integer within ±(2^53-1), and invalid UTF-8 or a lone surrogate is
//! fatal. The output is built with the v1 builders, which trim and fold, and every output is
//! read back through [`PubkySocialObject::from_uri`] before it is returned. That read is the
//! one skip point: an object either migrates whole or is skipped with a [`Skip`] category.
//!
//! What the run learned from listing the v0 tree lives in a [`MigrationCtx`]: the owner, and
//! from every v0 File object the name and blob it names and the blob's extension. Feed it all
//! the File objects first; posts, profiles, tags and blobs read it.
//!
//! References are rewritten one way everywhere. A `pubky://<pk>/pub/pubky.app/...` URI that
//! the frozen v0 parser classifies as a profile, post, follow or tag moves to
//! `pubky://<pk>/pub/social/v1/...` with the host unchanged, other users' included. The
//! owner's v0 `files/{tsid}` and `blobs/{hash}` references instead resolve to the migrated
//! `files/{hash}.{ext}`, the former through the File object, and stay as they were when the
//! run cannot resolve them, since the legacy URI keeps resolving. A feed, mute or bookmark
//! reference stays as written too: those migrate under the private root, so no public v1
//! spelling of them resolves. A path the v0 parser calls unknown, and `http` and `https`
//! targets, are not rewritten. Every value then takes its canonical spelling, which is what
//! the v1 reader requires and what ids are derived from.
//!
//! Foreign members of a v0 object are discarded and not reported: the v0 read models have no
//! catch-all, so a run cannot count them.

use crate::canonicalize::{
    canonicalize_external_uri, canonicalize_pubky_uri, canonicalize_web_uri, checked,
    AllowedSchemes,
};
use crate::common::{frozen_trim, is_frozen_whitespace, trimmed_or_none, validate_safe_json_int};
use crate::constants::{PROTOCOL, PUBLIC_ROOT, SOCIAL_NAMESPACE};
use crate::limits::VALIDATION_LIMITS;
use crate::mime::{essence, mime_to_ext};
use crate::models::legacy_v0;
use crate::traits::{HasIdPath, HasPath, HashId, Root, Validatable, PUB_CTX};
use crate::{
    bookmark_filename, epoch_segment, resolve_deref, sanitize_tag_label, PubkyId,
    PubkySocialAttachment, PubkySocialBookmark, PubkySocialCollectionItem,
    PubkySocialCollectionLayout, PubkySocialFeed, PubkySocialFeedConfig, PubkySocialFile,
    PubkySocialFollow, PubkySocialMute, PubkySocialObject, PubkySocialPost, PubkySocialPostKind,
    PubkySocialTag, PubkySocialUser, PubkySocialUserLink,
};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

/// Why an object did not migrate. Categories, not messages, so a run can count them.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Skip {
    /// The bytes are not one JSON object: a syntax error, invalid UTF-8, a lone surrogate.
    Malformed,
    /// A field the transform reads is missing or has the wrong JSON type.
    Shape,
    /// An integer field is not an integer within ±(2^53-1).
    UnsafeInteger,
    /// A v0 deletion marker: a post whose content or a profile whose name is `[DELETED]`.
    /// v1 has no sentinel, so it must not become a live object.
    Tombstone,
    /// An article whose title is empty once trimmed and truncated.
    EmptyTitle,
    /// A post kind v1 does not know.
    UnknownPostKind,
    /// A feed filtering on a content kind v1 does not know. Its id cannot be derived, and
    /// dropping the filter would write a different feed.
    UnknownFeedContent,
    /// The output is over the v1 size cap of its type.
    Oversize,
    /// The output fails the v1 reader.
    Invalid,
    /// The path is not a v0 object with a v1 counterpart, or it is another owner's.
    NotMigrated,
}

impl Skip {
    /// Every category, in declaration order, for a report that counts them.
    pub const ALL: &[Skip] = &[
        Skip::Malformed,
        Skip::Shape,
        Skip::UnsafeInteger,
        Skip::Tombstone,
        Skip::EmptyTitle,
        Skip::UnknownPostKind,
        Skip::UnknownFeedContent,
        Skip::Oversize,
        Skip::Invalid,
        Skip::NotMigrated,
    ];

    /// The snake_case category name.
    pub fn as_str(&self) -> &'static str {
        match self {
            Skip::Malformed => "malformed",
            Skip::Shape => "shape",
            Skip::UnsafeInteger => "unsafe_integer",
            Skip::Tombstone => "tombstone",
            Skip::EmptyTitle => "empty_title",
            Skip::UnknownPostKind => "unknown_post_kind",
            Skip::UnknownFeedContent => "unknown_feed_content",
            Skip::Oversize => "oversize",
            Skip::Invalid => "invalid",
            Skip::NotMigrated => "not_migrated",
        }
    }
}

impl fmt::Display for Skip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A category is also an error, so a run can carry it through `?` and report it.
impl std::error::Error for Skip {}

/// A value the v1 rules refuse, dropped so the object around it still migrates.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dropped {
    /// The profile image failed the image gate: a scheme other than pubky or web, a
    /// non-canonical spelling, or over the cap.
    ProfileImage,
    /// The profile link at this index of the v0 list failed the web gate.
    ProfileLink { index: usize },
}

impl fmt::Display for Dropped {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Dropped::ProfileImage => f.write_str("profile_image"),
            Dropped::ProfileLink { index } => write!(f, "profile_link[{index}]"),
        }
    }
}

/// What one v0 object becomes: the owner-relative paths and bytes to write, and what was
/// dropped on the way.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Migrated {
    pub writes: Vec<(String, Vec<u8>)>,
    pub dropped: Vec<Dropped>,
}

impl Migrated {
    fn one(write: (String, Vec<u8>)) -> Self {
        Self {
            writes: vec![write],
            dropped: vec![],
        }
    }
}

/// A v0 File object as the run keeps it.
#[derive(Debug, Clone)]
struct V0File {
    name: String,
    /// The blob it names, only when that blob is the owner's own: another tree's blob
    /// migrates under that tree's extension table, which this run cannot see.
    hash: Option<String>,
}

/// What one run knows about the owner's v0 tree.
#[derive(Debug, Clone)]
pub struct MigrationCtx {
    owner: PubkyId,
    files: BTreeMap<String, V0File>,
    /// blob hash -> (lowest File id naming it, its extension, whether the type is an image)
    exts: BTreeMap<String, (String, String, bool)>,
}

/// The v0 deletion marker, shared by the post content and the profile name.
const TOMBSTONE: &str = "[DELETED]";

impl MigrationCtx {
    pub fn new(owner: PubkyId) -> Self {
        Self {
            owner,
            files: BTreeMap::new(),
            exts: BTreeMap::new(),
        }
    }

    pub fn owner(&self) -> &PubkyId {
        &self.owner
    }

    /// Reads the v0 File object stored at `files/{tsid}`. It has no v1 counterpart: its name
    /// moves into the attachments that reference it, its blob becomes the reference target,
    /// and its content type picks the blob's extension.
    pub fn read_v0_file(&mut self, tsid: &str, v0_bytes: &[u8]) -> Result<(), Skip> {
        let object = read_object(v0_bytes)?;
        let name = str_field(&object, "name")?;
        let src = str_field(&object, "src")?;
        let content_type = str_field(&object, "content_type")?;

        let own = legacy_v0::ParsedUri::try_from(src).is_ok_and(|p| p.user_id == self.owner);
        let hash = own
            .then(|| resolve_deref(tsid, src))
            .flatten()
            .and_then(|key| key.strip_prefix("files/").map(str::to_string));

        if let Some(hash) = &hash {
            // One extension per blob: the File with the bytewise-lowest id names it, so the
            // answer does not depend on the order the run met the Files in; a File read again
            // replaces its own earlier reading, as it does in `files`
            let ext = mime_to_ext(content_type);
            let image = essence(content_type).is_some_and(|e| e.starts_with("image/"));
            let takes_over =
                |(held, _, _): &(String, String, bool)| tsid.as_bytes() <= held.as_bytes();
            if self.exts.get(hash).is_none_or(takes_over) {
                self.exts
                    .insert(hash.clone(), (tsid.to_string(), ext, image));
            }
        }
        self.files.insert(
            tsid.to_string(),
            V0File {
                name: name.to_string(),
                hash,
            },
        );
        Ok(())
    }

    /// One v0 object by its owner-relative path, for a run that walks the tree: a File
    /// object is read into the run and writes nothing, anything else goes through
    /// [`transform`]. Walk `files/` first, so the objects that reference them find them.
    pub fn migrate(&mut self, v0_path: &str, v0_bytes: &[u8]) -> Result<Migrated, Skip> {
        match classify(&self.owner, v0_path)? {
            legacy_v0::Resource::File(tsid) => self
                .read_v0_file(&tsid, v0_bytes)
                .map(|()| Migrated::default()),
            resource => transform_resource(resource, v0_bytes, self),
        }
    }

    /// The extension a blob migrates under; `bin` for a blob no File names.
    fn ext_of(&self, hash: &str) -> &str {
        self.exts
            .get(hash)
            .map_or("bin", |(_, ext, _)| ext.as_str())
    }

    /// Rewrites one reference, with the name of the v0 File it went through, if any. The
    /// input is trimmed first: v0's own writer trimmed, and a migrator owes canonical spelling.
    fn rewrite(&self, uri: &str) -> Rewritten {
        use legacy_v0::Resource;
        let uri = frozen_trim(uri);
        if !uri.starts_with("pubky") {
            let canonical = if uri.starts_with("http://") || uri.starts_with("https://") {
                canonicalize_web_uri(uri)
            } else {
                canonicalize_external_uri(uri)
            };
            return Rewritten::plain(canonical.unwrap_or_else(|_| uri.to_string()));
        }
        // A spelling nothing can canonicalize stays as written, for the reader to refuse
        let Ok(canonical) = canonicalize_pubky_uri(uri) else {
            return Rewritten::plain(uri.to_string());
        };
        let rest = &canonical[PROTOCOL.len()..];
        let Some((host, path)) = rest.split_once('/') else {
            return Rewritten::plain(canonical);
        };
        let Some(tail) = path.strip_prefix(LEGACY_PREFIX) else {
            return Rewritten::plain(canonical);
        };
        let Ok(parsed) = legacy_v0::ParsedUri::try_from(canonical.as_str()) else {
            return Rewritten::plain(canonical);
        };
        let own = host == self.owner.as_ref();
        match parsed.resource {
            // Another tree's media migrates under that tree's extension table, which this
            // run cannot see, so only the owner's references resolve
            Resource::File(tsid) => {
                let file = own.then(|| self.files.get(&tsid)).flatten();
                match file.and_then(|f| f.hash.as_ref().map(|h| (f, h))) {
                    Some((file, hash)) => self.media(hash, Some(file.name.clone())),
                    None => Rewritten::plain(canonical),
                }
            }
            Resource::Blob(hash) if own => self.media(&hash, None),
            // Feeds, mutes and bookmarks migrate under the private root, a feed under a
            // re-derived id, so no public v1 spelling of them resolves; the legacy one does
            Resource::Blob(_)
            | Resource::Feed(_)
            | Resource::Mute(_)
            | Resource::Bookmark(_)
            | Resource::LastRead
            | Resource::Unknown => Rewritten::plain(canonical),
            Resource::User | Resource::Post(_) | Resource::Follow(_) | Resource::Tag(_) => {
                Rewritten::plain(format!(
                    "{PROTOCOL}{host}/{PUBLIC_ROOT}/{SOCIAL_NAMESPACE}/{}/{tail}",
                    epoch_segment()
                ))
            }
        }
    }

    fn media(&self, hash: &str, name: Option<String>) -> Rewritten {
        let path = PubkySocialFile::create_path(&format!("{hash}.{}", self.ext_of(hash)));
        Rewritten {
            uri: [PROTOCOL, self.owner.as_ref(), &path].concat(),
            name,
            image: self.exts.get(hash).is_some_and(|(_, _, image)| *image),
        }
    }

    /// Serializes, checks the size cap, and reads the result back through the v1 reader.
    fn emit<T: Validatable>(&self, path: &str, object: &T) -> Result<(String, Vec<u8>), Skip> {
        let bytes = serde_json::to_vec(object).map_err(|_| Skip::Invalid)?;
        if bytes.len() > T::MAX_BYTES {
            return Err(Skip::Oversize);
        }
        self.read_back(path, bytes)
    }

    fn read_back(&self, path: &str, bytes: Vec<u8>) -> Result<(String, Vec<u8>), Skip> {
        let uri = [PROTOCOL, self.owner.as_ref(), path].concat();
        PubkySocialObject::from_uri(&uri, &bytes).map_err(|_| Skip::Invalid)?;
        Ok((path.trim_start_matches('/').to_string(), bytes))
    }
}

/// The legacy namespace under the public root, where every v0 object lived.
const LEGACY_PREFIX: &str = "pub/pubky.app/";

struct Rewritten {
    uri: String,
    name: Option<String>,
    /// Resolved to the owner's media, and that media is an image type.
    image: bool,
}

impl Rewritten {
    fn plain(uri: String) -> Self {
        Self {
            uri,
            name: None,
            image: false,
        }
    }
}

// ---- reading v0 bytes ----

fn read_object(bytes: &[u8]) -> Result<Map<String, Value>, Skip> {
    match serde_json::from_slice(bytes) {
        Ok(Value::Object(object)) => Ok(object),
        _ => Err(Skip::Malformed),
    }
}

fn str_field<'a>(object: &'a Map<String, Value>, key: &str) -> Result<&'a str, Skip> {
    object.get(key).and_then(Value::as_str).ok_or(Skip::Shape)
}

/// Absent and `null` are both no value.
fn opt_str<'a>(object: &'a Map<String, Value>, key: &str) -> Result<Option<&'a str>, Skip> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(_) => Err(Skip::Shape),
    }
}

/// Absent and `null` are both an empty list.
fn opt_array<'a>(object: &'a Map<String, Value>, key: &str) -> Result<&'a [Value], Skip> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(&[]),
        Some(Value::Array(items)) => Ok(items),
        Some(_) => Err(Skip::Shape),
    }
}

fn opt_str_list<'a>(
    object: &'a Map<String, Value>,
    key: &str,
) -> Result<Option<Vec<&'a str>>, Skip> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(items)) => items
            .iter()
            .map(|v| v.as_str().ok_or(Skip::Shape))
            .collect::<Result<_, _>>()
            .map(Some),
        Some(_) => Err(Skip::Shape),
    }
}

fn int_field(object: &Map<String, Value>, key: &str) -> Result<i64, Skip> {
    match object.get(key) {
        Some(Value::Number(n)) => n
            .as_i64()
            .filter(|&v| validate_safe_json_int(v).is_ok())
            .ok_or(Skip::UnsafeInteger),
        _ => Err(Skip::Shape),
    }
}

/// A wire enum from its string; a value the enum does not know lands in its `Unknown`.
fn enum_field<T: DeserializeOwned>(object: &Map<String, Value>, key: &str) -> Result<T, Skip> {
    let wire = Value::String(str_field(object, key)?.to_string());
    serde_json::from_value(wire).map_err(|_| Skip::Shape)
}

/// v0 spelled two kinds differently; every other kind keeps its wire name.
fn v1_kind(v0_kind: &str) -> PubkySocialPostKind {
    let name = match v0_kind {
        "short" => "note",
        "long" => "article",
        other => other,
    };
    PubkySocialPostKind::from_str(name).unwrap_or(PubkySocialPostKind::Unknown)
}

// ---- the transforms ----

/// The v0 profile. Display text takes the builder trim. An image or a link url the v1 gate
/// refuses is dropped and reported, and the profile still migrates. A `[DELETED]` name is
/// the v0 deletion marker and skips.
pub fn transform_user(v0_bytes: &[u8], ctx: &MigrationCtx) -> Result<Migrated, Skip> {
    let object = read_object(v0_bytes)?;
    let name = str_field(&object, "name")?;
    if name == TOMBSTONE {
        return Err(Skip::Tombstone);
    }
    let bio = opt_str(&object, "bio")?;
    let status = opt_str(&object, "status")?;
    let mut dropped = vec![];

    let image = opt_str(&object, "image")?.and_then(|raw| {
        let uri = ctx.rewrite(raw).uri;
        let max = VALIDATION_LIMITS.image_url_max_length;
        match checked(
            "image",
            &uri,
            AllowedSchemes::PubkyHttpHttps,
            max,
            &PUB_CTX,
            None,
        ) {
            Ok(()) => Some(uri),
            Err(_) => {
                dropped.push(Dropped::ProfileImage);
                None
            }
        }
    });

    let links = match object.get("links") {
        None | Some(Value::Null) => None,
        Some(Value::Array(items)) => {
            let mut links = vec![];
            for (index, item) in items.iter().enumerate() {
                let link = item.as_object().ok_or(Skip::Shape)?;
                let title = str_field(link, "title")?;
                let url = ctx.rewrite(str_field(link, "url")?).uri;
                let max = VALIDATION_LIMITS.user_link_url_max_length;
                match checked("url", &url, AllowedSchemes::HttpHttps, max, &PUB_CTX, None) {
                    Ok(()) => links.push(PubkySocialUserLink::new(title.to_string(), url)),
                    Err(_) => dropped.push(Dropped::ProfileLink { index }),
                }
            }
            Some(links)
        }
        Some(_) => return Err(Skip::Shape),
    };

    let user = PubkySocialUser::new(
        name.to_string(),
        bio.map(str::to_string),
        image,
        links,
        status.map(str::to_string),
    );
    let write = ctx.emit(&PubkySocialUser::create_path(), &user)?;
    Ok(Migrated {
        writes: vec![write],
        dropped,
    })
}

/// The v0 post `posts/{id}`, written as its first version `posts/{id}/{id}.json`.
pub fn transform_post(id: &str, v0_bytes: &[u8], ctx: &MigrationCtx) -> Result<Migrated, Skip> {
    let object = read_object(v0_bytes)?;
    let content = str_field(&object, "content")?;
    if content == TOMBSTONE {
        return Err(Skip::Tombstone);
    }
    let kind = v1_kind(str_field(&object, "kind")?);
    if !kind.is_known() {
        return Err(Skip::UnknownPostKind);
    }
    let rewrite = |uri: &str| ctx.rewrite(uri).uri;
    let parent = opt_str(&object, "parent")?.map(rewrite);
    // The embed kind is derivable from its target, so only the uri moves
    let embed = match object.get("embed") {
        None | Some(Value::Null) => None,
        Some(Value::Object(embed)) => Some(rewrite(str_field(embed, "uri")?)),
        Some(_) => return Err(Skip::Shape),
    };
    let lock = opt_str(&object, "lock")?.map(rewrite);
    let references = opt_array(&object, "attachments")?
        .iter()
        .map(|v| v.as_str().map(|uri| ctx.rewrite(uri)).ok_or(Skip::Shape))
        .collect::<Result<Vec<_>, _>>()?;

    let post = match kind {
        PubkySocialPostKind::Article => {
            let (title, body, cover) = article_text(content);
            let title = article_title(&title).ok_or(Skip::EmptyTitle)?;
            let mut references = references;
            // An envelope cover wins. Otherwise the first attachment was the cover by
            // convention, but only an image can be one: it moves into the envelope without
            // its file name, and anything else stays an attachment
            let cover = match cover {
                Some(uri) => Some(rewrite(&uri)),
                None if references.first().is_some_and(|r| r.image) => {
                    Some(references.remove(0).uri)
                }
                None => None,
            };
            PubkySocialPost::new_article(
                title,
                body,
                cover,
                parent,
                embed,
                references.into_iter().map(attachment).collect(),
                lock,
            )
        }
        PubkySocialPostKind::Collection => {
            let mut post = collection(content, ctx)?;
            // Carried so the reader refuses what a collection may not have, rather than
            // the transform silently dropping it
            post.parent = parent;
            post.embed = embed;
            post.attachments = references.into_iter().map(attachment).collect();
            post.lock = lock;
            post
        }
        kind => PubkySocialPost::new_with_lock(
            content.to_string(),
            kind,
            parent,
            embed,
            references.into_iter().map(attachment).collect(),
            lock,
        ),
    };
    let path = PubkySocialPost::create_path_in(Root::Pub, id, id, None);
    Ok(Migrated::one(ctx.emit(&path, &post)?))
}

/// An attachment keeps the v0 File name it was dereferenced through; a blank one is absent.
fn attachment(reference: Rewritten) -> PubkySocialAttachment {
    let name = reference.name.and_then(trimmed_or_none);
    PubkySocialAttachment::new(reference.uri, None, name)
}

/// A v0 long post's title, body and cover: the JSON envelope's when the content is one, else
/// the first line with text in it, the whole content, and no cover.
fn article_text(content: &str) -> (String, String, Option<String>) {
    if let Ok(Value::Object(envelope)) = serde_json::from_str::<Value>(content) {
        if let (Some(Value::String(title)), Some(Value::String(body))) =
            (envelope.get("title"), envelope.get("body"))
        {
            let cover = envelope
                .get("cover_image")
                .and_then(Value::as_str)
                .map(str::to_string);
            return (title.clone(), body.clone(), cover);
        }
    }
    let title = content
        .split('\n')
        .find(|line| !frozen_trim(line).is_empty())
        .unwrap_or("");
    (title.to_string(), content.to_string(), None)
}

/// Trim, truncate to the title cap in code points, trim the end again; `None` when nothing
/// is left.
fn article_title(raw: &str) -> Option<String> {
    let truncated: String = frozen_trim(raw)
        .chars()
        .take(VALIDATION_LIMITS.article_title_max_length)
        .collect();
    let title = truncated.trim_end_matches(is_frozen_whitespace);
    (!title.is_empty()).then(|| title.to_string())
}

/// A v0 collection envelope: string items become item objects, a blank description becomes
/// absent through the builder, the rest copies.
fn collection(content: &str, ctx: &MigrationCtx) -> Result<PubkySocialPost, Skip> {
    let Ok(Value::Object(envelope)) = serde_json::from_str::<Value>(content) else {
        return Err(Skip::Shape);
    };
    let name = str_field(&envelope, "name")?;
    let description = opt_str(&envelope, "description")?;
    let items = opt_array(&envelope, "items")?
        .iter()
        .map(|v| {
            v.as_str()
                .map(|uri| PubkySocialCollectionItem::new(ctx.rewrite(uri).uri, None))
                .ok_or(Skip::Shape)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let cover = opt_str(&envelope, "cover_image")?.map(|uri| ctx.rewrite(uri).uri);
    let layout: Option<PubkySocialCollectionLayout> = match envelope.get("layout") {
        None | Some(Value::Null) => None,
        Some(_) => Some(enum_field(&envelope, "layout")?),
    };
    Ok(PubkySocialPost::new_collection(
        name.to_string(),
        description.map(str::to_string),
        items,
        cover,
        layout,
    ))
}

/// A v0 tag. The target is rewritten, the label folded, and the id re-derived from both.
pub fn transform_tag(v0_bytes: &[u8], ctx: &MigrationCtx) -> Result<Migrated, Skip> {
    let object = read_object(v0_bytes)?;
    let tag = PubkySocialTag {
        uri: ctx.rewrite(str_field(&object, "uri")?).uri,
        label: sanitize_tag_label(str_field(&object, "label")?),
        created_at: int_field(&object, "created_at")?,
        extra: Default::default(),
    };
    let path = PubkySocialTag::create_path(&tag.create_id());
    Ok(Migrated::one(ctx.emit(&path, &tag)?))
}

/// A v0 follow of `followee`.
pub fn transform_follow(
    followee: &str,
    v0_bytes: &[u8],
    ctx: &MigrationCtx,
) -> Result<Migrated, Skip> {
    let object = read_object(v0_bytes)?;
    let follow = PubkySocialFollow {
        created_at: int_field(&object, "created_at")?,
        extra: Default::default(),
    };
    let path = PubkySocialFollow::create_path(followee);
    Ok(Migrated::one(ctx.emit(&path, &follow)?))
}

/// A v0 mute of `mutee`, now under the private root.
pub fn transform_mute(mutee: &str, v0_bytes: &[u8], ctx: &MigrationCtx) -> Result<Migrated, Skip> {
    let object = read_object(v0_bytes)?;
    let mute = PubkySocialMute {
        created_at: int_field(&object, "created_at")?,
        extra: Default::default(),
    };
    let path = PubkySocialMute::create_path(mutee);
    Ok(Migrated::one(ctx.emit(&path, &mute)?))
}

/// A v0 bookmark, now under the private root with the rewritten target in its filename. A
/// target too long for the filename takes the overflow form and rides in the content.
pub fn transform_bookmark(v0_bytes: &[u8], ctx: &MigrationCtx) -> Result<Migrated, Skip> {
    let object = read_object(v0_bytes)?;
    let target = ctx.rewrite(str_field(&object, "uri")?).uri;
    let created_at = int_field(&object, "created_at")?;
    let filename = bookmark_filename(&target).map_err(|_| Skip::Invalid)?;
    let bookmark = PubkySocialBookmark {
        created_at,
        target: filename.starts_with('~').then_some(target),
        extra: Default::default(),
    };
    let path = PubkySocialBookmark::create_path(&filename);
    Ok(Migrated::one(ctx.emit(&path, &bookmark)?))
}

/// A v0 feed, now private, under an id re-derived from its config. Name and icon copy with
/// the builder trim and fold. A blank tag label is dropped, as the v0 reader dropped it; a
/// list left empty becomes no filter, which is the one spelling the v1 builder accepts for
/// it (v0 kept the empty list). The two renamed post kinds are renamed here too.
pub fn transform_feed(v0_bytes: &[u8], ctx: &MigrationCtx) -> Result<Migrated, Skip> {
    let object = read_object(v0_bytes)?;
    let config = object
        .get("feed")
        .and_then(Value::as_object)
        .ok_or(Skip::Shape)?;
    let filter = |key: &str| -> Result<Option<Vec<String>>, Skip> {
        let labels: Vec<String> = opt_str_list(config, key)?
            .unwrap_or_default()
            .into_iter()
            .filter(|label| !frozen_trim(label).is_empty())
            .map(str::to_string)
            .collect();
        Ok((!labels.is_empty()).then_some(labels))
    };
    let content = match opt_str(config, "content")? {
        None => None,
        Some(kind) => match v1_kind(kind) {
            PubkySocialPostKind::Unknown => return Err(Skip::UnknownFeedContent),
            kind => Some(kind),
        },
    };
    let config = PubkySocialFeedConfig::new(
        filter("tags")?,
        filter("domain_tags")?,
        enum_field(config, "reach")?,
        enum_field(config, "layout")?,
        enum_field(config, "sort")?,
        content,
    )
    .map_err(|_| Skip::Invalid)?;
    let feed = PubkySocialFeed {
        feed: config,
        name: frozen_trim(str_field(&object, "name")?).to_string(),
        icon: opt_str(&object, "icon")?.map(|icon| crate::ascii_fold(frozen_trim(icon))),
        created_at: int_field(&object, "created_at")?,
        extra: Default::default(),
    };
    let id = feed.derive_id().map_err(|_| Skip::Invalid)?;
    let path = PubkySocialFeed::create_path_in(Root::Priv, &id);
    Ok(Migrated::one(ctx.emit(&path, &feed)?))
}

/// A v0 blob `blobs/{hash}`: the same bytes at `files/{hash}.{ext}`, the extension from the
/// run's table.
pub fn transform_blob(hash: &str, bytes: &[u8], ctx: &MigrationCtx) -> Result<Migrated, Skip> {
    if bytes.len() > VALIDATION_LIMITS.max_file_size_bytes {
        return Err(Skip::Oversize);
    }
    let path = PubkySocialFile::create_path(&format!("{hash}.{}", ctx.ext_of(hash)));
    Ok(Migrated::one(ctx.read_back(&path, bytes.to_vec())?))
}

/// Any v0 object by its owner-relative path (`pub/pubky.app/...`), classified by the v0
/// parser. A v0 File has no v1 counterpart and writes nothing; read it into the context with
/// [`MigrationCtx::read_v0_file`] before anything that references it, or walk the tree with
/// [`MigrationCtx::migrate`], which does both.
pub fn transform(v0_path: &str, v0_bytes: &[u8], ctx: &MigrationCtx) -> Result<Migrated, Skip> {
    transform_resource(classify(&ctx.owner, v0_path)?, v0_bytes, ctx)
}

/// What the v0 parser calls a path of the owner's tree, given as the owner-relative path or
/// the full `pubky://` URL a LIST returns. A path it does not know has no v1 counterpart, and
/// another owner's tree is not this run's to migrate.
fn classify(owner: &PubkyId, v0_path: &str) -> Result<legacy_v0::Resource, Skip> {
    let uri = if v0_path.starts_with(PROTOCOL) {
        v0_path.to_string()
    } else {
        [
            PROTOCOL,
            owner.as_ref(),
            "/",
            v0_path.trim_start_matches('/'),
        ]
        .concat()
    };
    let parsed = legacy_v0::ParsedUri::try_from(uri.as_str()).map_err(|_| Skip::NotMigrated)?;
    if parsed.user_id != *owner {
        return Err(Skip::NotMigrated);
    }
    Ok(parsed.resource)
}

fn transform_resource(
    resource: legacy_v0::Resource,
    v0_bytes: &[u8],
    ctx: &MigrationCtx,
) -> Result<Migrated, Skip> {
    use legacy_v0::Resource;
    match resource {
        Resource::User => transform_user(v0_bytes, ctx),
        Resource::Post(id) => transform_post(&id, v0_bytes, ctx),
        Resource::Follow(pk) => transform_follow(pk.as_ref(), v0_bytes, ctx),
        Resource::Mute(pk) => transform_mute(pk.as_ref(), v0_bytes, ctx),
        Resource::Bookmark(_) => transform_bookmark(v0_bytes, ctx),
        Resource::Tag(_) => transform_tag(v0_bytes, ctx),
        Resource::Feed(_) => transform_feed(v0_bytes, ctx),
        Resource::Blob(hash) => transform_blob(&hash, v0_bytes, ctx),
        Resource::File(_) => Ok(Migrated::default()),
        Resource::LastRead | Resource::Unknown => Err(Skip::NotMigrated),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";
    const TS: &str = "0032SSN7Q4EVG";
    const TS2: &str = "0034A0X7NJ52G";
    const HASH: &str = "AKSZ57W2RFKHV1EHK007FQQ8TW";

    fn ctx() -> MigrationCtx {
        MigrationCtx::new(PubkyId::try_from(OWNER).unwrap())
    }

    fn file(content_type: &str) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "name": "photo",
            "created_at": 1727740800000000i64,
            "src": format!("pubky://{OWNER}/pub/pubky.app/blobs/{HASH}"),
            "content_type": content_type,
            "size": 20,
        }))
        .unwrap()
    }

    #[test]
    fn all_lists_every_category_once_in_order() {
        // Wildcard-free, so a new category fails to compile here until ALL carries it
        let position = |skip: &Skip| match skip {
            Skip::Malformed => 0,
            Skip::Shape => 1,
            Skip::UnsafeInteger => 2,
            Skip::Tombstone => 3,
            Skip::EmptyTitle => 4,
            Skip::UnknownPostKind => 5,
            Skip::UnknownFeedContent => 6,
            Skip::Oversize => 7,
            Skip::Invalid => 8,
            Skip::NotMigrated => 9,
        };
        assert_eq!(Skip::ALL.len(), 10);
        for (index, skip) in Skip::ALL.iter().enumerate() {
            assert_eq!(position(skip), index, "{skip}");
        }
    }

    #[test]
    fn a_walk_reads_files_into_the_run_and_transforms_the_rest() {
        let mut ctx = ctx();
        let file_path = format!("pub/pubky.app/files/{TS}");
        assert_eq!(
            ctx.migrate(&file_path, &file("image/png")),
            Ok(Migrated::default())
        );
        assert_eq!(ctx.migrate(&file_path, b"{\"name\":1}"), Err(Skip::Shape));
        let tag = serde_json::json!({
            "uri": format!("pubky://{OWNER}/pub/pubky.app/files/{TS}"),
            "label": "pic",
            "created_at": 1727740800000000i64,
        });
        let migrated = ctx
            .migrate(
                "pub/pubky.app/tags/0034A0X7NJ536",
                tag.to_string().as_bytes(),
            )
            .unwrap();
        let (_, bytes) = &migrated.writes[0];
        let written: serde_json::Value = serde_json::from_slice(bytes).unwrap();
        assert_eq!(
            written["uri"],
            format!("pubky://{OWNER}/pub/social/v1/files/{HASH}.png")
        );
        assert_eq!(
            ctx.migrate("pub/pubky.app/widgets/x", b"{}"),
            Err(Skip::NotMigrated)
        );
    }

    #[test]
    fn a_full_url_of_the_owner_classifies_like_its_path_and_another_owners_does_not() {
        let ctx = ctx();
        let bytes = br#"{"created_at":1727740800000000}"#;
        let path = format!("pub/pubky.app/follows/{OWNER}");
        let from_path = transform(&path, bytes, &ctx).unwrap();
        let from_url = transform(&format!("pubky://{OWNER}/{path}"), bytes, &ctx).unwrap();
        assert_eq!(from_url, from_path);
        let other = "pxnu33x7jtpx9ar1ytsi4yxbp6a5o36gwhffs8zoxmbuptici1jy";
        assert_eq!(
            transform(&format!("pubky://{other}/{path}"), bytes, &ctx),
            Err(Skip::NotMigrated)
        );
    }

    #[test]
    fn invalid_utf8_and_lone_surrogates_are_fatal() {
        let ctx = ctx();
        for bytes in [
            b"{\"created_at\":1727740800000000,\"x\":\"\xff\"}".as_slice(),
            br#"{"created_at":1727740800000000,"x":"\ud800"}"#.as_slice(),
            br#"{"created_at":1727740800000000,"x":"\udc00\ud800"}"#.as_slice(),
            b"[1]".as_slice(),
            b"".as_slice(),
        ] {
            assert_eq!(
                transform_follow(OWNER, bytes, &ctx),
                Err(Skip::Malformed),
                "{bytes:?}"
            );
        }
    }

    #[test]
    fn an_integer_field_is_a_safe_integer() {
        let ctx = ctx();
        for (raw, verdict) in [
            ("9007199254740991", Ok(())),
            ("-9007199254740991", Ok(())),
            ("9007199254740992", Err(Skip::UnsafeInteger)),
            ("-9007199254740992", Err(Skip::UnsafeInteger)),
            ("1.5", Err(Skip::UnsafeInteger)),
            ("1e3", Err(Skip::UnsafeInteger)),
            ("\"1\"", Err(Skip::Shape)),
        ] {
            let bytes = format!(r#"{{"created_at":{raw}}}"#);
            let got = transform_follow(OWNER, bytes.as_bytes(), &ctx).map(|_| ());
            assert_eq!(got, verdict, "{raw}");
        }
    }

    #[test]
    fn the_lowest_file_id_names_the_extension_in_any_order() {
        for order in [[TS, TS2], [TS2, TS]] {
            let mut ctx = ctx();
            for tsid in order {
                let content_type = if tsid == TS {
                    "image/png"
                } else {
                    "image/jpeg"
                };
                ctx.read_v0_file(tsid, &file(content_type)).unwrap();
            }
            assert_eq!(ctx.ext_of(HASH), "png", "{order:?}");
        }
        assert_eq!(ctx().ext_of(HASH), "bin");
    }

    #[test]
    fn a_file_read_twice_takes_its_last_reading_everywhere() {
        let mut ctx = ctx();
        ctx.read_v0_file(TS, &file("image/png")).unwrap();
        let again = serde_json::to_vec(&serde_json::json!({
            "name": "second",
            "created_at": 1727740800000000i64,
            "src": format!("pubky://{OWNER}/pub/pubky.app/blobs/{HASH}"),
            "content_type": "application/pdf",
            "size": 20,
        }))
        .unwrap();
        ctx.read_v0_file(TS, &again).unwrap();
        assert_eq!(ctx.ext_of(HASH), "pdf");
        let rewritten = ctx.rewrite(&format!("pubky://{OWNER}/pub/pubky.app/files/{TS}"));
        assert_eq!(rewritten.name.as_deref(), Some("second"));
        assert!(!rewritten.image);
    }

    #[test]
    fn a_file_that_does_not_read_leaves_its_references_as_written() {
        let mut ctx = ctx();
        assert_eq!(ctx.read_v0_file(TS, b"{\"name\":1}"), Err(Skip::Shape));
        let legacy = format!("pubky://{OWNER}/pub/pubky.app/files/{TS}");
        assert_eq!(ctx.rewrite(&legacy).uri, legacy);
        ctx.read_v0_file(TS, &file("image/png")).unwrap();
        assert_eq!(
            ctx.rewrite(&legacy).uri,
            format!("pubky://{OWNER}/pub/social/v1/files/{HASH}.png")
        );
    }

    #[test]
    fn only_the_legacy_prefix_is_rewritten() {
        let ctx = ctx();
        for (from, to) in [
            (
                format!("pubky{OWNER}/pub/pubky.app/posts/{TS}"),
                format!("pubky://{OWNER}/pub/social/v1/posts/{TS}"),
            ),
            (
                format!("pubky://{OWNER}/pub/locks.app/x"),
                format!("pubky://{OWNER}/pub/locks.app/x"),
            ),
            (
                format!("pubky://{OWNER}/pub/pubky.appx/y"),
                format!("pubky://{OWNER}/pub/pubky.appx/y"),
            ),
            (format!("pubky://{OWNER}"), format!("pubky://{OWNER}")),
            // Private in v1, under a re-derived id for a feed: the legacy spelling is the
            // one that resolves
            (
                format!("pubky://{OWNER}/pub/pubky.app/feeds/{TS}"),
                format!("pubky://{OWNER}/pub/pubky.app/feeds/{TS}"),
            ),
            (
                format!("pubky://{OWNER}/pub/pubky.app/mutes/{OWNER}"),
                format!("pubky://{OWNER}/pub/pubky.app/mutes/{OWNER}"),
            ),
            (
                format!("pubky://{OWNER}/pub/pubky.app/bookmarks/{HASH}"),
                format!("pubky://{OWNER}/pub/pubky.app/bookmarks/{HASH}"),
            ),
            // No v1 counterpart: the frozen parser calls these last_read and unknown
            (
                format!("pubky://{OWNER}/pub/pubky.app/last_read"),
                format!("pubky://{OWNER}/pub/pubky.app/last_read"),
            ),
            (
                format!("pubky://{OWNER}/pub/pubky.app/widgets/{TS}"),
                format!("pubky://{OWNER}/pub/pubky.app/widgets/{TS}"),
            ),
            ("https://x.com \n".into(), "https://x.com".into()),
            (" https://x.com".into(), "https://x.com".into()),
            ("\u{3000}NOSTR:abc ".into(), "nostr:abc".into()),
            ("not a uri".into(), "not a uri".into()),
        ] {
            assert_eq!(ctx.rewrite(&from).uri, to, "{from}");
        }
    }

    #[test]
    fn the_owners_blob_reference_becomes_the_media_spelling() {
        let mut ctx = ctx();
        ctx.read_v0_file(TS, &file("image/png")).unwrap();
        let own = format!("pubky://{OWNER}/pub/pubky.app/blobs/{HASH}");
        let rewritten = ctx.rewrite(&own);
        assert_eq!(
            rewritten.uri,
            format!("pubky://{OWNER}/pub/social/v1/files/{HASH}.png")
        );
        assert!(rewritten.image && rewritten.name.is_none());
        // An orphan blob still resolves, as bin
        let orphan = "8Z8CWH8NVYQY39ZEBFGKQWWEKG";
        assert_eq!(
            ctx.rewrite(&format!("pubky://{OWNER}/pub/pubky.app/blobs/{orphan}"))
                .uri,
            format!("pubky://{OWNER}/pub/social/v1/files/{orphan}.bin")
        );
        let other = "pxnu33x7jtpx9ar1ytsi4yxbp6a5o36gwhffs8zoxmbuptici1jy";
        let theirs = format!("pubky://{other}/pub/pubky.app/blobs/{HASH}");
        assert_eq!(ctx.rewrite(&theirs).uri, theirs);
    }

    #[test]
    fn a_blob_over_the_size_cap_skips_as_oversize() {
        let bytes = vec![0u8; VALIDATION_LIMITS.max_file_size_bytes + 1];
        assert_eq!(transform_blob(HASH, &bytes, &ctx()), Err(Skip::Oversize));
        // At the cap the size passes; the read-back then refuses the hash, not the size
        assert_ne!(
            transform_blob(HASH, &bytes[1..], &ctx()),
            Err(Skip::Oversize)
        );
    }

    #[test]
    fn only_an_image_deref_is_promoted_to_the_cover() {
        let mut ctx = ctx();
        ctx.read_v0_file(TS, &file("application/pdf")).unwrap();
        assert!(
            !ctx.rewrite(&format!("pubky://{OWNER}/pub/pubky.app/files/{TS}"))
                .image
        );
        ctx.read_v0_file(TS2, &file("image/jpeg; charset=x"))
            .unwrap();
        // TS is lower and names the blob pdf, so TS2's jpeg deref is not an image either
        assert!(
            !ctx.rewrite(&format!("pubky://{OWNER}/pub/pubky.app/files/{TS2}"))
                .image
        );
    }

    #[test]
    fn the_title_pipeline_counts_code_points_and_retrims() {
        let title = format!("{}\u{3000}\u{3000}tail", "é".repeat(98));
        assert_eq!(article_title(&title), Some("é".repeat(98)));
        assert_eq!(article_title("\u{3000} x \u{3000}"), Some("x".into()));
        assert_eq!(article_title(" \t "), None);
        // U+200B is not whitespace, so it is a title
        assert_eq!(article_title("\u{200B}"), Some("\u{200B}".into()));
    }

    #[test]
    fn a_post_over_the_size_cap_skips_as_oversize() {
        let content = "x".repeat(VALIDATION_LIMITS.post_max_bytes);
        let bytes = serde_json::to_vec(&serde_json::json!({
            "content": content, "kind": "short", "parent": null, "embed": null,
        }))
        .unwrap();
        assert_eq!(transform_post(TS, &bytes, &ctx()), Err(Skip::Oversize));
    }

    #[test]
    fn a_collection_keeps_what_it_may_not_have_so_the_reader_refuses_it() {
        let content = serde_json::json!({"name": "list", "items": []}).to_string();
        let bytes = serde_json::to_vec(&serde_json::json!({
            "content": content, "kind": "collection", "parent": null, "embed": null,
            "attachments": ["https://example.com/a.png"],
        }))
        .unwrap();
        assert_eq!(transform_post(TS, &bytes, &ctx()), Err(Skip::Invalid));
    }
}
