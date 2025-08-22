#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use js_sys::Array;
use pubky_app_specs::{
    parse_uri, post_uri_builder, user_path_builder, user_uri_builder, PubkyAppPost,
    PubkyAppPostKind, PubkyAppUserLink, PubkySpecsBuilder,
};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_create_follow() {
    let specs =
        PubkySpecsBuilder::new("operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".to_string())
            .expect("Valid pubky ID");

    let result = specs
        .create_follow("operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".to_string())
        .expect("create_follow should not fail");
    let meta = result.meta();
    let follow = result.follow();

    // Now we can call the Rust getter methods directly:
    assert_eq!(
        meta.path(),
        "/pub/pubky.app/follows/operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo"
    );
    assert_eq!(
        meta.url(),
        "pubky://operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/pub/pubky.app/follows/operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo"
    );
    assert_eq!(
        meta.id(),
        "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo"
    );
    assert!(follow.created_at > 0);
}

#[wasm_bindgen_test]
fn test_create_user_rust_api() {
    let specs =
        PubkySpecsBuilder::new("operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".to_string())
            .expect("Valid pubky ID");

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
    assert_eq!(meta.path(), user_path_builder());
    assert_eq!(
        meta.url(),
        user_uri_builder("operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".into())
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
    let specs =
        PubkySpecsBuilder::new("operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".to_string())
            .expect("Invalid specsBuilder");

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
    assert_eq!(meta.path(), user_path_builder());
    assert_eq!(
        meta.url(),
        user_uri_builder("operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".into())
    );
    assert_eq!(meta.id(), "");

    // Validate the user object
    assert_eq!(user.name(), "Bob");
    assert_eq!(user.bio(), None);
    assert_eq!(user.image(), None);
    assert!(user.links().is_none());
    assert_eq!(user.status(), None);
}

#[wasm_bindgen_test]
fn test_post_from_json() {
    // A JSON string representing a post.
    let post_json = r#"
    {
        "content": "Hello from JSON!",
        "kind": "long",
        "parent": null,
        "embed": null,
        "attachments": null
    }
    "#;
    // Convert the JSON string into a JsValue.
    let js_value = js_sys::JSON::parse(post_json).expect("Failed to parse JSON string");
    // Use the new factory method to create a WASM PubkyAppPost.
    let post = PubkyAppPost::from_json(&js_value).expect("Post should deserialize successfully");

    assert_eq!(post.content, "Hello from JSON!");
    assert_eq!(post.kind, PubkyAppPostKind::Long);
    assert_eq!(post.embed, None);
    assert_eq!(post.attachments, None);
}

#[wasm_bindgen_test]
fn test_parse_uri() {
    // A valid URI for a post resource.
    let uri = post_uri_builder(
        "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo".into(),
        "0032SSN7Q4EVG".into(),
    );

    // Call the wasm-exposed parse_uri function.
    let parsed = parse_uri(&uri).expect("Expected valid URI parsing");

    // Verify the user ID is correctly parsed.
    assert_eq!(
        parsed.user_id(),
        "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo",
        "The user ID should match the host in the URI"
    );

    // Verify that the resource string indicates a post resource.
    assert!(
        parsed.resource().contains("posts"),
        "The resource field should indicate a posts resource"
    );

    // Verify that the resource ID is correctly extracted.
    assert_eq!(
        parsed.resource_id().unwrap(),
        "0032SSN7Q4EVG",
        "The resource_id should match the post id provided in the URI"
    );
}
