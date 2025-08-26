use crate::{
    common::*,
    traits::{HasIdPath, HasPath},
    PubkyAppBlob, PubkyAppBookmark, PubkyAppFile, PubkyAppFollow, PubkyAppMute, PubkyAppPost,
    PubkyAppTag, PubkyAppUser,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = baseUriBuilder))]
pub fn base_uri_builder(user_id: String) -> String {
    format!("{}{}{}{}", PROTOCOL, user_id, PUBLIC_PATH, APP_PATH)
}

/// Builds an User Path of the form "/pub/pubky.app/profile.json"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = userPathBuilder))]
pub fn user_path_builder() -> String {
    PubkyAppUser::create_path()
}

/// Builds an User URI of the form "pubky://<user_pubky_id>/pub/pubky.app/profile.json"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = userUriBuilder))]
pub fn user_uri_builder(user_id: String) -> String {
    let user_path = user_path_builder();
    [PROTOCOL, &user_id, &user_path].concat()
}

/// Builds a Post Path of the form "/pub/pubky.app/posts/<post_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = postPathBuilder))]
pub fn post_path_builder(id: &str) -> String {
    PubkyAppPost::create_path(id)
}

/// Builds a Post URI of the form "pubky://<author_id>/pub/pubky.app/posts/<post_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = postUriBuilder))]
pub fn post_uri_builder(author_id: String, post_id: String) -> String {
    let post_path = post_path_builder(&post_id);
    [PROTOCOL, &author_id, &post_path].concat()
}

/// Builds a Follow Path of the form "/pub/pubky.app/follows/<follow_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = followPathBuilder))]
pub fn follow_path_builder(id: &str) -> String {
    PubkyAppFollow::create_path(id)
}

/// Builds a Follow URI of the form "pubky://<author_id>/pub/pubky.app/follows/<follow_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = followUriBuilder))]
pub fn follow_uri_builder(author_id: String, follow_id: String) -> String {
    let follow_path = follow_path_builder(&follow_id);
    [PROTOCOL, &author_id, &follow_path].concat()
}

/// Builds a Mute Path of the form "/pub/pubky.app/mutes/<mute_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = mutePathBuilder))]
pub fn mute_path_builder(id: &str) -> String {
    PubkyAppMute::create_path(id)
}

/// Builds a Mute URI of the form "pubky://<author_id>/pub/pubky.app/mutes/<mute_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = muteUriBuilder))]
pub fn mute_uri_builder(author_id: String, mute_id: String) -> String {
    let mute_path = mute_path_builder(&mute_id);
    [PROTOCOL, &author_id, &mute_path].concat()
}

/// Builds a Bookmark Path of the form "/pub/pubky.app/bookmarks/<bookmark_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = bookmarkPathBuilder))]
pub fn bookmark_path_builder(id: &str) -> String {
    PubkyAppBookmark::create_path(id)
}

/// Builds a Bookmark URI of the form "pubky://<author_id>/pub/pubky.app/bookmarks/<bookmark_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = bookmarkUriBuilder))]
pub fn bookmark_uri_builder(author_id: String, bookmark_id: String) -> String {
    let bookmark_path = bookmark_path_builder(&bookmark_id);
    [PROTOCOL, &author_id, &bookmark_path].concat()
}

/// Builds a Tag Path of the form "/pub/pubky.app/tags/<tag_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = tagPathBuilder))]
pub fn tag_path_builder(id: &str) -> String {
    PubkyAppTag::create_path(id)
}

/// Builds a Tag URI of the form "pubky://<author_id>/pub/pubky.app/tags/<tag_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = tagUriBuilder))]
pub fn tag_uri_builder(author_id: String, tag_id: String) -> String {
    let tag_path = tag_path_builder(&tag_id);
    [PROTOCOL, &author_id, &tag_path].concat()
}

/// Builds a File Path of the form "/pub/pubky.app/files/<file_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = filePathBuilder))]
pub fn file_path_builder(id: &str) -> String {
    PubkyAppFile::create_path(id)
}

/// Builds a File URI of the form "pubky://<author_id>/pub/pubky.app/files/<file_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = fileUriBuilder))]
pub fn file_uri_builder(author_id: String, file_id: String) -> String {
    let file_path = file_path_builder(&file_id);
    [PROTOCOL, &author_id, &file_path].concat()
}

/// Builds a Blob Path of the form "/pub/pubky.app/blobs/<blob_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = blobPathBuilder))]
pub fn blob_path_builder(id: &str) -> String {
    PubkyAppBlob::create_path(id)
}

/// Builds a Blob URI of the form "pubky://<author_id>/pub/pubky.app/blobs/<blob_id>"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = blobUriBuilder))]
pub fn blob_uri_builder(author_id: String, blob_id: String) -> String {
    let blob_path = blob_path_builder(&blob_id);
    [PROTOCOL, &author_id, &blob_path].concat()
}
