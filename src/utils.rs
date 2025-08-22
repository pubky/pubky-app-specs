use crate::{common::*, traits::HasPath, PubkyAppUser};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = baseUriBuilder))]
pub fn base_uri_builder(user_id: String) -> String {
    format!("{}{}{}{}", PROTOCOL, user_id, PUBLIC_PATH, APP_PATH)
}

/// Builds an User URI of the form "pubky://<user_pubky_id>/pub/pubky.app/profile.json"
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = userUriBuilder))]
pub fn user_uri_builder(user_id: String) -> String {
    format!("{}{}{}", PROTOCOL, user_id, PubkyAppUser::create_path())
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = postUriBuilder))]
pub fn post_uri_builder(author_id: String, post_id: String) -> String {
    format!(
        "{}{}{}{}posts/{}",
        PROTOCOL, author_id, PUBLIC_PATH, APP_PATH, post_id
    )
}
