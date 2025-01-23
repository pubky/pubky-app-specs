#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

use pubky_app_specs::PubkyAppSpecs;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_create_follow_rust_api() {
    let specs = PubkyAppSpecs::new("test_pubky_id".to_string());

    let result = specs
        .create_follow("followee_123".to_string())
        .expect("create_follow should not fail");

    // Now we can call the Rust getter methods directly:
    assert_eq!(result.path(), "/pub/pubky.app/follows/followee_123");
    assert_eq!(
        result.url(),
        "pubky://test_pubky_id/pub/pubky.app/follows/followee_123"
    );
    assert_eq!(result.id(), "");
    // And you could do more checks on `result.json()`, etc.
}
