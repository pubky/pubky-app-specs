use crate::{
    common::*,
    traits::{HasIdPath, HasPath},
    PubkyAppPost, PubkyAppUser,
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
