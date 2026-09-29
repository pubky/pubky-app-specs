#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use js_sys::{Reflect, JSON};
use pubky_social_specs::traits::HasIdPath;
use pubky_social_specs::wasm::{
    create_follow, create_mute, create_tag, create_user, parse_uri, read_object, validate,
};
use pubky_social_specs::{follow_uri_builder, post_uri_builder, PubkySocialFollow};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const PK: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";

fn get(value: &JsValue, path: &str) -> JsValue {
    path.split('.').fold(value.clone(), |v, key| {
        Reflect::get(&v, &JsValue::from_str(key)).unwrap()
    })
}

fn string(value: &JsValue, path: &str) -> String {
    get(value, path).as_string().unwrap()
}

#[wasm_bindgen_test]
fn follow_meta_names_the_followee() {
    let made = create_follow(PK, PK).unwrap();
    assert_eq!(string(&made, "meta.id"), PK);
    assert_eq!(
        string(&made, "meta.path"),
        PubkySocialFollow::create_path(PK)
    );
    assert_eq!(
        string(&made, "meta.url"),
        follow_uri_builder(PK.into(), PK.into())
    );
    assert!(get(&made, "object.created_at").as_f64().unwrap() > 0.0);
}

#[wasm_bindgen_test]
fn mute_lives_under_the_private_root() {
    let made = create_mute(PK, PK).unwrap();
    assert!(string(&made, "meta.url").contains("/priv/social/v1/mutes/"));
}

#[wasm_bindgen_test]
fn user_input_is_a_plain_object() {
    let input =
        JSON::parse(r#"{"name":"  Alice  ","links":[{"title":"x","url":"https://a.dev"}]}"#)
            .unwrap();
    let made = create_user(PK, input).unwrap();
    assert_eq!(string(&made, "object.name"), "Alice");
    assert_eq!(string(&made, "meta.id"), "");
    assert_eq!(string(&made, "meta.path"), "/pub/social/v1/profile.json");
    // An unknown input member is a typo, not an extension
    let typo = JSON::parse(r#"{"name":"Alice","bios":"x"}"#).unwrap();
    assert!(create_user(PK, typo).is_err());
}

#[wasm_bindgen_test]
fn parse_uri_tags_the_resource() {
    let parsed = parse_uri(&post_uri_builder(PK.into(), "0032SSN7Q4EVG".into())).unwrap();
    assert_eq!(string(&parsed, "userId"), PK);
    assert_eq!(string(&parsed, "visibility"), "public");
    assert_eq!(string(&parsed, "resource.kind"), "post");
    assert_eq!(string(&parsed, "resource.id"), "0032SSN7Q4EVG");
    assert_eq!(
        string(&parsed, "path"),
        "/pub/social/v1/posts/0032SSN7Q4EVG"
    );
}

#[cfg(feature = "migrator")]
#[wasm_bindgen_test]
fn a_migrated_write_is_what_read_object_gives_with_its_meta() {
    use pubky_social_specs::wasm::{create_migration, migrate};
    let mut run = create_migration(PK).unwrap();
    let path = format!("pub/pubky.app/follows/{PK}");
    let result = migrate(&mut run, &path, br#"{"created_at":1727740800000000}"#).unwrap();
    let write = js_sys::Array::from(&get(&result, "writes")).get(0);
    assert_eq!(string(&write, "kind"), "follow");
    assert_eq!(string(&write, "meta.id"), PK);
    assert_eq!(
        string(&write, "meta.url"),
        follow_uri_builder(PK.into(), PK.into())
    );
    assert_eq!(
        get(&write, "object.created_at").as_f64(),
        Some(1727740800000000.0)
    );
    assert_eq!(js_sys::Array::from(&get(&result, "dropped")).length(), 0);
    let skipped = migrate(&mut run, &path, b"not json").unwrap();
    assert_eq!(string(&skipped, "skip"), "malformed");
}

#[wasm_bindgen_test]
fn a_built_object_reads_back_and_validates() {
    let uri = format!("pubky://{PK}/pub/social/v1/profile.json");
    let made = create_tag(PK, uri, "x".into()).unwrap();
    let url = string(&made, "meta.url");
    let stored = String::from(JSON::stringify(&get(&made, "object")).unwrap());
    let read = read_object(&url, stored.into_bytes()).unwrap();
    assert_eq!(string(&read, "kind"), "tag");
    validate(&url, get(&read, "object")).unwrap();
}
