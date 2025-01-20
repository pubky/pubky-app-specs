// tests/wasm_follow_test.rs

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

use js_sys::Object;
use pubky_app_specs::create_pubky_app_follow;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_create_pubky_app_follow() {
    let pubky_id = "fake_pubky_id";

    // Call the wasm-exported function
    let result = create_pubky_app_follow(pubky_id.into()).unwrap();

    // The returned `JsValue` is the JSON object { json, id, path }
    // We can convert it to a JS object so we can extract fields
    let result_obj = Object::try_from(&result).expect("expected a JS object");

    // For example, check the `path` field
    let path_val =
        js_sys::Reflect::get(&result_obj, &JsValue::from_str("path")).expect("no path field");
    let path_str = path_val.as_string().expect("path must be a string");

    assert_eq!(
        path_str,
        format!("/pub/pubky.app/follows/{}", pubky_id),
        "create_pubky_app_follow did not produce expected path"
    );
}
