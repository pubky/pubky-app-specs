use crate::{
    constants::{social_path, PROTOCOL},
    traits::Root,
    traits::{HasIdPath, HasPath},
    PubkySocialBlob, PubkySocialBookmark, PubkySocialFeed, PubkySocialFile, PubkySocialFollow,
    PubkySocialMute, PubkySocialPost, PubkySocialTag, PubkySocialUser,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// The public v1 LIST prefix: "pubky://<user_id>/pub/social/v1/".
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = baseUriBuilder))]
pub fn base_uri_builder(user_id: String) -> String {
    let prefix = social_path(Root::Pub, "");
    [PROTOCOL, &user_id, &prefix].concat()
}

/// Builds an User URI of the form "pubky://<user_pubky_id>/pub/social/v1/profile.json"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = userUriBuilder))]
pub fn user_uri_builder(user_id: String) -> String {
    let user_path = PubkySocialUser::create_path();
    [PROTOCOL, &user_id, &user_path].concat()
}

/// Builds the versionless post REFERENCE, "pubky://<author_id>/pub/social/v1/posts/<post_id>":
/// the spelling every reference field uses. The storage path of a version comes from
/// `PubkySocialPost::create_path`.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = postUriBuilder))]
pub fn post_uri_builder(author_id: String, post_id: String) -> String {
    let leaf = format!("{}{post_id}", PubkySocialPost::PATH_SEGMENT);
    let post_path = social_path(Root::Pub, &leaf);
    [PROTOCOL, &author_id, &post_path].concat()
}

/// Builds a Follow URI of the form "pubky://<author_id>/pub/social/v1/follows/<follow_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = followUriBuilder))]
pub fn follow_uri_builder(author_id: String, follow_id: String) -> String {
    let follow_path = PubkySocialFollow::create_path(&follow_id);
    [PROTOCOL, &author_id, &follow_path].concat()
}

/// Builds a Mute URI of the form "pubky://<author_id>/pub/social/v1/mutes/<mute_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = muteUriBuilder))]
pub fn mute_uri_builder(author_id: String, mute_id: String) -> String {
    let mute_path = PubkySocialMute::create_path(&mute_id);
    [PROTOCOL, &author_id, &mute_path].concat()
}

/// Builds a Bookmark URI of the form "pubky://<author_id>/pub/social/v1/bookmarks/<bookmark_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = bookmarkUriBuilder))]
pub fn bookmark_uri_builder(author_id: String, bookmark_id: String) -> String {
    let bookmark_path = PubkySocialBookmark::create_path(&bookmark_id);
    [PROTOCOL, &author_id, &bookmark_path].concat()
}

/// Builds a Tag URI of the form "pubky://<author_id>/pub/social/v1/tags/<tag_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = tagUriBuilder))]
pub fn tag_uri_builder(author_id: String, tag_id: String) -> String {
    let tag_path = PubkySocialTag::create_path(&tag_id);
    [PROTOCOL, &author_id, &tag_path].concat()
}

/// Builds a File URI of the form "pubky://<author_id>/pub/social/v1/files/<file_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = fileUriBuilder))]
pub fn file_uri_builder(author_id: String, file_id: String) -> String {
    let file_path = PubkySocialFile::create_path(&file_id);
    [PROTOCOL, &author_id, &file_path].concat()
}

/// Builds a Blob URI of the form "pubky://<author_id>/pub/social/v1/blobs/<blob_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = blobUriBuilder))]
pub fn blob_uri_builder(author_id: String, blob_id: String) -> String {
    let blob_path = PubkySocialBlob::create_path(&blob_id);
    [PROTOCOL, &author_id, &blob_path].concat()
}

/// Builds a Feed URI of the form "pubky://<author_id>/pub/social/v1/feeds/<feed_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = feedUriBuilder))]
pub fn feed_uri_builder(author_id: String, feed_id: String) -> String {
    let feed_path = PubkySocialFeed::create_path(&feed_id);
    [PROTOCOL, &author_id, &feed_path].concat()
}
