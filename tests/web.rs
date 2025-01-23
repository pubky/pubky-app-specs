#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

use js_sys::{Object, Reflect};
use pubky_app_specs::{
    create_pubky_app_follow, create_pubky_app_user, traits::Validatable, PubkyAppUser,
};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_create_pubky_app_follow() {
    let pubky_id = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";

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

#[wasm_bindgen_test]
fn test_create_pubky_app_user_ffi() {
    let name = String::from("alice");
    let bio = Some(String::from("Alice is a test user."));
    let image = None;
    let links = JsValue::NULL;
    let status = Some(String::from("testing"));

    let result = create_pubky_app_user(name, bio, image, links, status)
        .expect("create_pubky_app_user should not fail");

    let result_obj = js_sys::Object::try_from(&result).expect("should be an object");

    let id_val =
        js_sys::Reflect::get(&result_obj, &JsValue::from_str("id")).expect("no `id` field");
    let path_val =
        js_sys::Reflect::get(&result_obj, &JsValue::from_str("path")).expect("no `path` field");
    // Check the `json` field explicitly
    let json_val = Reflect::get(&result_obj, &JsValue::from_str("json")).expect("no `json` field");

    // Attempt to convert `json_val` to an Object
    let json_obj = Object::try_from(&json_val).expect("json field should be an object");

    let id_str = id_val.as_string().expect("id must be a string");
    let path_str = path_val.as_string().expect("path must be a string");

    assert_eq!(id_str, "", "Expected ID to be empty");
    assert_eq!(path_str, "/pub/pubky.app/profile.json", "Path is incorrect");

    // name should be "alice" now:
    let username_val =
        Reflect::get(&json_obj, &JsValue::from_str("name")).expect("no `name` in json");
    let username_str = username_val.as_string().unwrap();
    assert_eq!(username_str, "alice", "username mismatch");

    let bio_val = Reflect::get(&json_obj, &JsValue::from_str("bio")).expect("no `bio` in json");
    let bio_str = bio_val.as_string().unwrap();
    assert_eq!(bio_str, "Alice is a test user.", "bio mismatch");
}

#[wasm_bindgen_test]
fn test_create_pubky_app_user_oop() {
    // Instead of calling `create_pubky_app_user(...)`, we now
    // directly construct and validate a `PubkyAppUser`.

    let user = PubkyAppUser::new(
        "alice".to_string(),
        Some("Alice is a test user.".to_string()),
        None, // image
        None, // links
        Some("testing".to_string()),
    );

    // Perform validation (no ID-based checks for profile.json)
    user.validate("").expect("User validation should succeed");

    // `get_data()` returns a `JsValue` that contains { id, path, json }
    let data_value = user.get_data().expect("Should return data");

    // Convert that JsValue into a JS object so we can pick out fields
    let data_obj = Object::try_from(&data_value).expect("expected a JS object");

    let id_val = Reflect::get(&data_obj, &JsValue::from_str("id")).expect("no `id` field");
    let path_val = Reflect::get(&data_obj, &JsValue::from_str("path")).expect("no `path` field");
    let json_val = Reflect::get(&data_obj, &JsValue::from_str("json")).expect("no `json` field");

    // Convert to Rust-friendly types

    assert_eq!(id_val.as_string(), None);

    let path_str = path_val.as_string().expect("path must be a string");
    assert_eq!(path_str, "/pub/pubky.app/profile.json", "Path is incorrect");

    // Inside `json`, we should see `{"name":"alice","bio":"Alice is a test user.",...}`
    let json_obj = Object::try_from(&json_val).expect("json field should be an object");

    let name_val = Reflect::get(&json_obj, &JsValue::from_str("name")).expect("no `name` in json");
    let name_str = name_val.as_string().unwrap_or_default();
    assert_eq!(name_str, "alice", "name mismatch");

    let bio_val = Reflect::get(&json_obj, &JsValue::from_str("bio")).expect("no `bio` in json");
    let bio_str = bio_val.as_string().unwrap_or_default();
    assert_eq!(bio_str, "Alice is a test user.", "bio mismatch");

    web_sys::console::log_1(&json_obj);

    let status_val =
        Reflect::get(&json_obj, &JsValue::from_str("status")).expect("no `status` in json");
    let status_str = status_val.as_string().unwrap_or_default();
    assert_eq!(status_str, "testing", "status mismatch");
}
