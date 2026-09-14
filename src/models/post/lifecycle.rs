//! Pure planners for the post lifecycle across the two roots: publish a private draft,
//! unpublish a public post, delete a post everywhere. No I/O: each planner takes the listings
//! the caller already fetched and returns ordered operations for the caller to execute. Paths
//! are owner-relative (`/pub/social/v1/...`), the form a homeserver LIST returns.

use super::content::article::PubkySocialArticleContent;
use super::content::collection::PubkySocialCollectionContent;
use super::{PubkySocialPost, PubkySocialPostKind};
use crate::constants::{social_path, PROTOCOL};
use crate::models::file::PubkySocialFile;
use crate::traits::{HasIdPath, Root, TimestampId, Validatable, ValidationCtx};
use crate::types::PubkyId;
use crate::uri::parse_version_leaf;

/// Publish: media copies first, then the post PUT. Skip-if-exists on a copy is the caller's,
/// since existence proves completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishPlan {
    /// `(private path, public path)` pairs, in reference order, deduplicated.
    pub media_copies: Vec<(String, String)>,
    /// The chosen version with its private media references respelled under `pub`.
    pub rewritten_post_json: String,
    /// `/pub/social/v1/posts/{id}/{editId}.json`.
    pub dest_path: String,
}

/// Unpublish: copy-backs first, then deletes, each list in order. Public media is deliberately
/// absent: removing it needs a whole-tree referencer check only the caller can run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnpublishPlan {
    /// `(public path, private path)` pairs, oldest first.
    pub copy_backs: Vec<(String, String)>,
    /// Legacy paths in the given order, then every public version, oldest first.
    pub deletes: Vec<String>,
}

/// Delete: legacy first, then every version oldest first with `pub` before `priv`, so the
/// post keeps resolving to its newest surviving version until the last DELETE. Media GC runs
/// only after that, and expanding each candidate to its every-epoch spelling is the caller's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletePlan {
    pub deletes: Vec<String>,
    /// Same-owner media referenced by any parsed version, under both roots, sorted.
    pub media_gc_candidates: Vec<String>,
}

fn owner_prefix(owner: &PubkyId) -> String {
    [PROTOCOL, owner.as_ref()].concat()
}

fn media_prefix(owner: &PubkyId, root: Root) -> String {
    let leaf = PubkySocialFile::PATH_SEGMENT;
    [owner_prefix(owner).as_str(), &social_path(root, leaf)].concat()
}

fn is_priv_rooted(uri: &str) -> bool {
    uri.strip_prefix(PROTOCOL)
        .and_then(|rest| rest.split_once('/'))
        .is_some_and(|(_, path)| path == "priv" || path.starts_with("priv/"))
}

/// The cover of either envelope, when it parses. An unparsable envelope is validation's error.
fn cover_of(post: &PubkySocialPost) -> Option<String> {
    match post.kind {
        PubkySocialPostKind::Article => {
            serde_json::from_str::<PubkySocialArticleContent>(&post.content)
                .ok()
                .and_then(|e| e.cover_image)
        }
        PubkySocialPostKind::Collection => {
            serde_json::from_str::<PubkySocialCollectionContent>(&post.content)
                .ok()
                .and_then(|e| e.cover_image)
        }
        _ => None,
    }
}

/// Media positions in reference order: attachments, then the cover.
fn media_refs(post: &PubkySocialPost) -> Vec<String> {
    let mut refs: Vec<String> = post.attachments.iter().map(|a| a.uri.clone()).collect();
    refs.extend(cover_of(post));
    refs
}

/// Non-media reference positions: parent, embed, lock, collection items.
fn other_refs(post: &PubkySocialPost) -> Vec<String> {
    let mut refs: Vec<String> = [&post.parent, &post.embed, &post.lock]
        .into_iter()
        .flatten()
        .cloned()
        .collect();
    if matches!(post.kind, PubkySocialPostKind::Collection) {
        if let Ok(e) = serde_json::from_str::<PubkySocialCollectionContent>(&post.content) {
            refs.extend(e.items);
        }
    }
    refs
}

/// The owner's private media a version references, first-encountered order, deduplicated.
/// Any other private reference is the root rule stated as a publish error. Kept in the shape
/// the media-closure enumerator will export once media collapses to one object.
fn private_media_refs(post: &PubkySocialPost, owner: &PubkyId) -> Result<Vec<String>, String> {
    let own_private = media_prefix(owner, Root::Priv);
    let mut media = Vec::new();
    for uri in media_refs(post) {
        if uri.starts_with(&own_private) {
            if !media.contains(&uri) {
                media.push(uri);
            }
        } else if is_priv_rooted(&uri) {
            return Err(format!(
                "cannot publish: private media is not the author's own: {uri}"
            ));
        }
    }
    if let Some(uri) = other_refs(post).into_iter().find(|u| is_priv_rooted(u)) {
        return Err(format!(
            "cannot publish: a public post cannot reference a private object: {uri}"
        ));
    }
    Ok(media)
}

/// `pubky://{owner}/priv/social/v1/files/X` -> the same under `pub`.
fn to_public(uri: &str, owner: &PubkyId) -> String {
    let private = media_prefix(owner, Root::Priv);
    match uri.strip_prefix(&private) {
        Some(rest) => [media_prefix(owner, Root::Pub).as_str(), rest].concat(),
        None => uri.to_string(),
    }
}

/// `pubky://{owner}{path}` -> `{path}`.
fn to_path(uri: &str, owner: &PubkyId) -> String {
    uri.strip_prefix(&owner_prefix(owner))
        .unwrap_or(uri)
        .to_string()
}

/// Publish one chosen version of a private post. Rewrites private media references to their
/// public spelling in the reference positions only, never over the content text, and
/// re-validates the result as a public object with the author in scope.
pub fn plan_publish(
    post_id: &str,
    chosen_edit_id: &str,
    chosen_version: &PubkySocialPost,
    owner: &PubkyId,
) -> Result<PublishPlan, String> {
    chosen_version.validate_id(post_id)?;
    crate::common::validate_timestamp_id_format(chosen_edit_id)?;
    let media = private_media_refs(chosen_version, owner)?;
    let media_copies = media
        .iter()
        .map(|uri| (to_path(uri, owner), to_path(&to_public(uri, owner), owner)))
        .collect();

    let mut post = chosen_version.clone();
    for attachment in &mut post.attachments {
        attachment.uri = to_public(&attachment.uri, owner);
    }
    // The cover lives inside the envelope: parse, respell, re-serialize; only when it changes
    if let Some(cover) = cover_of(&post) {
        let public = to_public(&cover, owner);
        if public != cover {
            post.content = match post.kind {
                PubkySocialPostKind::Article => {
                    let mut e: PubkySocialArticleContent =
                        serde_json::from_str(&post.content).map_err(|e| e.to_string())?;
                    e.cover_image = Some(public);
                    serde_json::to_string(&e).map_err(|e| e.to_string())?
                }
                _ => {
                    let mut e: PubkySocialCollectionContent =
                        serde_json::from_str(&post.content).map_err(|e| e.to_string())?;
                    e.cover_image = Some(public);
                    serde_json::to_string(&e).map_err(|e| e.to_string())?
                }
            };
        }
    }
    let ctx = ValidationCtx { root: Root::Pub };
    post.validate(Some(post_id), &ctx)?;
    post.check_references(&ctx, Some(owner))?;
    Ok(PublishPlan {
        media_copies,
        rewritten_post_json: serde_json::to_string(&post).map_err(|e| e.to_string())?,
        dest_path: PubkySocialPost::create_path_in(Root::Pub, post_id, chosen_edit_id, None),
    })
}

/// The editId of a version path, parsed by the path grammar's own rule.
fn edit_id_of(path: &str) -> Result<String, String> {
    let leaf = path.rsplit('/').next().unwrap_or(path);
    parse_version_leaf(leaf)
        .map(|(v, _)| v)
        .ok_or_else(|| format!("not a post version path: {path}"))
}

/// Same version leaf under the other root.
fn under(root: Root, path: &str) -> String {
    let after_root = path
        .trim_start_matches('/')
        .split_once('/')
        .map(|(_, rest)| rest)
        .unwrap_or("");
    ["/", root.segment(), "/", after_root].concat()
}

/// Versions sorted oldest first. Bytewise on the editId, never decoded: fixed-width
/// single-case ids make bytewise order chronological.
fn sorted_by_edit_id<T: Clone>(
    items: &[T],
    path: impl Fn(&T) -> &str,
) -> Result<Vec<(String, T)>, String> {
    let mut keyed = items
        .iter()
        .map(|t| Ok((edit_id_of(path(t))?, t.clone())))
        .collect::<Result<Vec<_>, String>>()?;
    keyed.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    Ok(keyed)
}

/// Unpublish: every public version newer than the private head is copied back, oldest
/// first; without a private tree the newest public version seeds it. Then the legacy paths
/// and every public version are deleted.
pub fn plan_unpublish(
    post_id: &str,
    public_v1_paths: &[String],
    legacy_public_paths: &[String],
    private_head_path: Option<&str>,
) -> Result<UnpublishPlan, String> {
    crate::common::validate_timestamp_id_format(post_id)?;
    let public = sorted_by_edit_id(public_v1_paths, |p| p.as_str())?;
    let head = private_head_path.map(edit_id_of).transpose()?;
    if public.is_empty() && head.is_none() {
        return Err(format!("nothing to unpublish for post {post_id}"));
    }
    let copy_backs = match &head {
        Some(head) => public
            .iter()
            .filter(|(e, _)| e.as_bytes() > head.as_bytes())
            .map(|(_, p)| (p.clone(), under(Root::Priv, p)))
            .collect(),
        None => public
            .last()
            .map(|(_, p)| vec![(p.clone(), under(Root::Priv, p))])
            .unwrap_or_default(),
    };
    let mut deletes = legacy_public_paths.to_vec();
    deletes.extend(public.into_iter().map(|(_, p)| p));
    Ok(UnpublishPlan {
        copy_backs,
        deletes,
    })
}

/// Delete everywhere. `parsed_versions` are the versions the caller could read; one it could
/// not contributes no GC candidates, a documented residual, since the caller's own index is
/// the real GC source.
pub fn plan_delete(
    post_id: &str,
    legacy_paths: &[String],
    v1_copies: &[(Root, String)],
    parsed_versions: &[PubkySocialPost],
    owner: &PubkyId,
) -> Result<DeletePlan, String> {
    crate::common::validate_timestamp_id_format(post_id)?;
    let mut copies = sorted_by_edit_id(v1_copies, |(_, p)| p.as_str())?;
    // stable sort above keeps input order at equal editId; pin pub before priv explicitly
    copies.sort_by(|a, b| {
        a.0.as_bytes()
            .cmp(b.0.as_bytes())
            .then_with(|| (a.1 .0 == Root::Priv).cmp(&(b.1 .0 == Root::Priv)))
    });
    let mut deletes = legacy_paths.to_vec();
    deletes.extend(copies.into_iter().map(|(_, (_, p))| p));

    let public = media_prefix(owner, Root::Pub);
    let private = media_prefix(owner, Root::Priv);
    let mut media_gc_candidates: Vec<String> = parsed_versions
        .iter()
        .flat_map(media_refs)
        .filter(|u| u.starts_with(&public) || u.starts_with(&private))
        .map(|u| to_path(&u, owner))
        .collect();
    media_gc_candidates.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    media_gc_candidates.dedup();
    Ok(DeletePlan {
        deletes,
        media_gc_candidates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PubkySocialAttachment;

    const PK: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";
    const OTHER: &str = "8pinxxgqs41n4aididenw5apqp1urfmzdztr8jt4abrkdn435ewo";
    const TS: &str = "0032SSN7Q4EVG";
    const E2: &str = "0032SSN7Q4EW0";
    const E3: &str = "0032SSN7Q4EWG";

    fn owner() -> PubkyId {
        PubkyId::try_from(PK).unwrap()
    }

    fn priv_file(n: &str) -> String {
        format!("pubky://{PK}/priv/social/v1/files/{n}")
    }

    fn pub_file(n: &str) -> String {
        format!("pubky://{PK}/pub/social/v1/files/{n}")
    }

    fn att(uri: &str) -> PubkySocialAttachment {
        PubkySocialAttachment::new(uri.into(), None, None)
    }

    fn image(atts: Vec<PubkySocialAttachment>) -> PubkySocialPost {
        PubkySocialPost::new("pic".into(), PubkySocialPostKind::Image, None, None, atts)
    }

    fn post_id() -> String {
        PubkySocialPost::default().create_id()
    }

    #[test]
    fn publish_copies_own_private_media_and_respells_only_those() {
        let a = priv_file("0034A0X7NJ52G");
        let b = priv_file("0034A0X7NJ52H");
        let web = "https://x.com/c.png";
        let mut draft = image(vec![att(&a), att(web), att(&b), att(&a)]);
        // prose that mentions a private URI must survive byte for byte
        draft.content = format!("see {a}");
        let id = post_id();
        let plan = plan_publish(&id, &id, &draft, &owner()).unwrap();
        assert_eq!(
            plan.media_copies,
            vec![
                (
                    "/priv/social/v1/files/0034A0X7NJ52G".to_string(),
                    "/pub/social/v1/files/0034A0X7NJ52G".to_string()
                ),
                (
                    "/priv/social/v1/files/0034A0X7NJ52H".to_string(),
                    "/pub/social/v1/files/0034A0X7NJ52H".to_string()
                ),
            ]
        );
        let out: PubkySocialPost = serde_json::from_str(&plan.rewritten_post_json).unwrap();
        let uris: Vec<&str> = out.attachments.iter().map(|a| a.uri.as_str()).collect();
        assert_eq!(
            uris,
            [
                pub_file("0034A0X7NJ52G").as_str(),
                web,
                pub_file("0034A0X7NJ52H").as_str(),
                pub_file("0034A0X7NJ52G").as_str()
            ]
        );
        assert_eq!(out.content, format!("see {a}"));
        assert_eq!(
            plan.dest_path,
            format!("/pub/social/v1/posts/{id}/{id}.json")
        );
    }

    #[test]
    fn publish_rewrites_the_article_cover_inside_the_envelope() {
        let cover = priv_file("0034A0X7NJ52G");
        let article = PubkySocialPost::new_article(
            "t".into(),
            "b".into(),
            Some(cover.clone()),
            None,
            None,
            vec![],
            None,
        );
        let id = post_id();
        let plan = plan_publish(&id, &id, &article, &owner()).unwrap();
        assert_eq!(plan.media_copies.len(), 1);
        let out: PubkySocialPost = serde_json::from_str(&plan.rewritten_post_json).unwrap();
        let e: PubkySocialArticleContent = serde_json::from_str(&out.content).unwrap();
        assert_eq!(
            e.cover_image.as_deref(),
            Some(pub_file("0034A0X7NJ52G").as_str())
        );
        assert_eq!(e.title, "t");
        // an already-public cover leaves the content bytes alone
        let public = PubkySocialPost::new_article(
            "t".into(),
            "b".into(),
            Some(pub_file("0034A0X7NJ52G")),
            None,
            None,
            vec![],
            None,
        );
        let plan = plan_publish(&id, &id, &public, &owner()).unwrap();
        let out: PubkySocialPost = serde_json::from_str(&plan.rewritten_post_json).unwrap();
        assert_eq!(out.content, public.content);
        assert!(plan.media_copies.is_empty());
    }

    #[test]
    fn publish_refuses_private_references_it_cannot_carry() {
        let id = post_id();
        let mut reply = PubkySocialPost::new(
            "re".into(),
            PubkySocialPostKind::Note,
            Some(format!("pubky://{PK}/priv/social/v1/posts/{TS}")),
            None,
            vec![],
        );
        let e = plan_publish(&id, &id, &reply, &owner()).unwrap_err();
        assert!(e.contains("private object"), "{e}");
        reply.parent = None;
        let foreign = image(vec![att(&format!(
            "pubky://{OTHER}/priv/social/v1/files/0034A0X7NJ52G"
        ))]);
        let e = plan_publish(&id, &id, &foreign, &owner()).unwrap_err();
        assert!(e.contains("author's own"), "{e}");
        assert!(plan_publish("not-an-id", &id, &reply, &owner()).is_err());
        assert!(plan_publish(&id, "not-an-id", &reply, &owner()).is_err());
    }

    fn pub_v(e: &str) -> String {
        format!("/pub/social/v1/posts/{TS}/{e}.json")
    }

    fn priv_v(e: &str) -> String {
        format!("/priv/social/v1/posts/{TS}/{e}.json")
    }

    #[test]
    fn unpublish_copies_back_versions_newer_than_the_private_head() {
        let legacy = "/pub/pubky.app/posts/0032SSN7Q4EVG".to_string();
        let public = vec![pub_v(E3), pub_v(TS), pub_v(E2)];
        let plan = plan_unpublish(
            TS,
            &public,
            std::slice::from_ref(&legacy),
            Some(&priv_v(E2)),
        )
        .unwrap();
        assert_eq!(plan.copy_backs, vec![(pub_v(E3), priv_v(E3))]);
        assert_eq!(plan.deletes, vec![legacy, pub_v(TS), pub_v(E2), pub_v(E3)]);
        let plan = plan_unpublish(TS, &public, &[], Some(&priv_v(E3))).unwrap();
        assert!(plan.copy_backs.is_empty());
        // no private tree: the newest public version seeds it
        let plan = plan_unpublish(TS, &public, &[], None).unwrap();
        assert_eq!(plan.copy_backs, vec![(pub_v(E3), priv_v(E3))]);
        assert!(plan_unpublish(TS, &[], &[], None).is_err());
        // a slugged leaf keeps its slug on the way back
        let slugged = format!("/pub/social/v1/posts/{TS}/{E3}-hello.json");
        let plan = plan_unpublish(TS, std::slice::from_ref(&slugged), &[], None).unwrap();
        assert_eq!(
            plan.copy_backs,
            vec![(
                slugged,
                format!("/priv/social/v1/posts/{TS}/{E3}-hello.json")
            )]
        );
        assert!(plan_unpublish(TS, &["/pub/social/v1/posts/x/y.json".into()], &[], None).is_err());
    }

    #[test]
    fn delete_orders_versions_oldest_first_pub_before_priv() {
        let legacy = "/pub/pubky.app/posts/0032SSN7Q4EVG".to_string();
        let copies = vec![
            (Root::Priv, priv_v(E3)),
            (Root::Priv, priv_v(TS)),
            (Root::Pub, pub_v(E2)),
            (Root::Pub, pub_v(TS)),
        ];
        let plan = plan_delete(TS, std::slice::from_ref(&legacy), &copies, &[], &owner()).unwrap();
        assert_eq!(
            plan.deletes,
            vec![legacy, pub_v(TS), priv_v(TS), pub_v(E2), priv_v(E3)]
        );
        assert!(plan.media_gc_candidates.is_empty());
    }

    #[test]
    fn delete_collects_same_owner_media_under_both_roots() {
        let v1 = image(vec![
            att(&priv_file("0034A0X7NJ52H")),
            att("https://x.com/a.png"),
        ]);
        let v2 = image(vec![
            att(&pub_file("0034A0X7NJ52G")),
            att(&pub_file("0034A0X7NJ52H")),
            att(&format!(
                "pubky://{OTHER}/pub/social/v1/files/0034A0X7NJ52G"
            )),
        ]);
        let plan = plan_delete(TS, &[], &[], &[v1, v2], &owner()).unwrap();
        assert_eq!(
            plan.media_gc_candidates,
            vec![
                "/priv/social/v1/files/0034A0X7NJ52H".to_string(),
                "/pub/social/v1/files/0034A0X7NJ52G".to_string(),
                "/pub/social/v1/files/0034A0X7NJ52H".to_string(),
            ]
        );
    }
}
