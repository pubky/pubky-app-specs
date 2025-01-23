#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use js_sys::Array;
use pubky_app_specs::{PubkyAppBuilder, PubkyAppUserLink};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_create_follow() {
    let specs = PubkyAppBuilder::new("test_pubky_id".to_string());

    let result = specs
        .create_follow("followee_123".to_string())
        .expect("create_follow should not fail");
    let meta = result.meta();
    let follow = result.follow();

    // Now we can call the Rust getter methods directly:
    assert_eq!(meta.path(), "/pub/pubky.app/follows/followee_123");
    assert_eq!(
        meta.url(),
        "pubky://test_pubky_id/pub/pubky.app/follows/followee_123"
    );
    assert_eq!(meta.id(), "");
    assert!(follow.created_at > 0);
}

#[wasm_bindgen_test]
fn test_create_user_rust_api() {
    let specs = PubkyAppBuilder::new("test_pubky_id".to_string());

    // Prepare links as a JS-compatible array
    let links = Array::new();
    links.push(
        &to_value(&PubkyAppUserLink {
            title: "GitHub".to_string(),
            url: "https://github.com/alice".to_string(),
        })
        .unwrap(),
    );
    links.push(
        &to_value(&PubkyAppUserLink {
            title: "Website".to_string(),
            url: "https://alice.dev".to_string(),
        })
        .unwrap(),
    );

    // Call `create_user` with test data
    let result = specs
        .create_user(
            "Alice".to_string(),
            Some("Maximalist".to_string()),
            Some("https://example.com/image.png".to_string()),
            JsValue::from(links),
            Some("Exploring the decentralized web.".to_string()),
        )
        .expect("create_user should not fail");

    // Extract meta and user objects
    let meta = result.meta();
    let user = result.user();

    // Validate the meta object
    assert_eq!(meta.path(), "/pub/pubky.app/profile.json");
    assert_eq!(
        meta.url(),
        "pubky://test_pubky_id/pub/pubky.app/profile.json"
    );
    assert_eq!(meta.id(), "");

    // Validate the user object
    assert_eq!(user.name(), "Alice");
    assert_eq!(user.bio().as_deref(), Some("Maximalist"));
    assert_eq!(
        user.image().as_deref(),
        Some("https://example.com/image.png")
    );
    assert_eq!(
        user.status().as_deref(),
        Some("Exploring the decentralized web.")
    );

    // Validate user links
    let user_links = user.links().expect("User should have links");
    assert_eq!(user_links.len(), 2);

    let first_link = user_links.get(0).expect("First link should exist");
    assert_eq!(first_link.title, "GitHub");
    assert_eq!(first_link.url, "https://github.com/alice");

    let second_link = user_links.get(1).expect("Second link should exist");
    assert_eq!(second_link.title, "Website");
    assert_eq!(second_link.url, "https://alice.dev/");
}

#[wasm_bindgen_test]
fn test_create_user_with_minimal_data() {
    let specs = PubkyAppBuilder::new("test_pubky_id".to_string());

    // Call `create_user` with minimal data
    let result = specs
        .create_user(
            "Bob".to_string(),
            None,
            None,
            JsValue::NULL, // No links
            None,
        )
        .expect("create_user should not fail");

    // Extract meta and user objects
    let meta = result.meta();
    let user = result.user();

    // Validate the meta object
    assert_eq!(meta.path(), "/pub/pubky.app/profile.json");
    assert_eq!(
        meta.url(),
        "pubky://test_pubky_id/pub/pubky.app/profile.json"
    );
    assert_eq!(meta.id(), "");

    // Validate the user object
    assert_eq!(user.name(), "Bob");
    assert_eq!(user.bio(), None);
    assert_eq!(user.image(), None);
    assert!(user.links().is_none());
    assert_eq!(user.status(), None);
}
