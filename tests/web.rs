#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

use js_sys::{Object, Reflect};
use pubky_app_specs::PubkyAppSpecs;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_create_follow() {
    // Instantiate our WASM-bound struct with some pubky_id
    let specs = PubkyAppSpecs::new("test_pubky_id".to_string());

    // Call the method under test, providing a followee pubky_id
    let result_value = specs
        .create_follow("followee_123".to_string())
        .expect("create_follow should not fail");

    // Convert the returned JsValue into a JS object so we can inspect fields
    let result_obj = Object::try_from(&result_value).expect("expected a JS object");

    // The returned object includes { id, path, url, json }
    // We'll check "path" and "url" for correctness
    let path_val =
        Reflect::get(&result_obj, &JsValue::from_str("path")).expect("no path field in result");
    let path_str = path_val.as_string().expect("path must be a string");

    let url_val =
        Reflect::get(&result_obj, &JsValue::from_str("url")).expect("no url field in result");
    let url_str = url_val.as_string().expect("url must be a string");

    // Since this is a follow, the path should be "/pub/pubky.app/follows/<followee_id>"
    assert_eq!(path_str, "/pub/pubky.app/follows/followee_123");

    assert_eq!(
        url_str,
        "pubky://test_pubky_id/pub/pubky.app/follows/followee_123"
    );
}
