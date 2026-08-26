//! The forward-compatibility contract: unknown enum values and unknown fields never break a
//! reader. `PubkySocialBlob` is not covered: its wire form is raw bytes, not a JSON object.
#![cfg(not(target_arch = "wasm32"))]

use pubky_social_specs::{
    traits::Validatable, PubkySocialBookmark, PubkySocialCollectionContent,
    PubkySocialCollectionLayout, PubkySocialFeed, PubkySocialFeedConfig, PubkySocialFeedLayout,
    PubkySocialFeedReach, PubkySocialFeedSort, PubkySocialFile, PubkySocialFollow,
    PubkySocialLastRead, PubkySocialMute, PubkySocialPost, PubkySocialPostEmbed,
    PubkySocialPostKind, PubkySocialTag, PubkySocialUser, PubkySocialUserLink,
};
use serde::de::DeserializeOwned;

const PK: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";

fn config(reach: &str, layout: &str, sort: &str, content: &str) -> String {
    format!(
        r#"{{"tags":null,"reach":"{reach}","layout":"{layout}","sort":"{sort}","content":{content}}}"#
    )
}

#[test]
fn unknown_feed_enum_values_deserialize_to_unknown() {
    let c: PubkySocialFeedConfig =
        serde_json::from_str(&config("galaxy", "columns", "recent", "null")).unwrap();
    assert_eq!(c.reach, PubkySocialFeedReach::Unknown);
    let c: PubkySocialFeedConfig =
        serde_json::from_str(&config("all", "spiral", "recent", "null")).unwrap();
    assert_eq!(c.layout, PubkySocialFeedLayout::Unknown);
    let c: PubkySocialFeedConfig =
        serde_json::from_str(&config("all", "columns", "random", "null")).unwrap();
    assert_eq!(c.sort, PubkySocialFeedSort::Unknown);
}

#[test]
fn unknown_primary_feed_enum_fails_validation_with_a_clear_message() {
    for (json, field) in [
        (config("galaxy", "columns", "recent", "null"), "reach"),
        (config("all", "spiral", "recent", "null"), "layout"),
        (config("all", "columns", "random", "null"), "sort"),
    ] {
        let c: PubkySocialFeedConfig = serde_json::from_str(&json).unwrap();
        let err = c.validate(None).unwrap_err();
        assert!(err.contains(field) && err.contains("unknown"), "{err}");
    }
    let feed = format!(
        r#"{{"feed":{},"name":"x","created_at":1727740800000000}}"#,
        config("galaxy", "columns", "recent", "null")
    );
    let f: PubkySocialFeed = serde_json::from_str(&feed).unwrap();
    assert!(f.validate(None).is_err());
}

#[test]
fn unknown_secondary_enum_degrades_instead_of_rejecting() {
    let c: PubkySocialFeedConfig =
        serde_json::from_str(&config("all", "columns", "recent", r#""totally-new-kind""#)).unwrap();
    assert_eq!(c.content, Some(PubkySocialPostKind::Unknown));
    assert_eq!(c.validate(None), Ok(()));

    let c: PubkySocialCollectionContent =
        serde_json::from_str(r#"{"name":"X","layout":"spiral"}"#).unwrap();
    assert_eq!(c.layout, Some(PubkySocialCollectionLayout::Unknown));
}

fn round_trips<T: DeserializeOwned + serde::Serialize>(wire: &[&str]) {
    for w in wire {
        let quoted = format!("\"{w}\"");
        let v: T = serde_json::from_str(&quoted).unwrap();
        assert_eq!(serde_json::to_string(&v).unwrap(), quoted);
    }
}

#[test]
fn known_wire_strings_round_trip_and_unknown_serializes_as_unknown() {
    round_trips::<PubkySocialFeedReach>(&["following", "followers", "friends", "all", "wot", "me"]);
    round_trips::<PubkySocialFeedLayout>(&["columns", "wide", "visual", "list"]);
    round_trips::<PubkySocialFeedSort>(&["recent", "popularity"]);
    round_trips::<PubkySocialPostKind>(&[
        "short",
        "long",
        "image",
        "video",
        "link",
        "file",
        "collection",
    ]);
    round_trips::<PubkySocialCollectionLayout>(&["grid", "list", "visual"]);
    assert_eq!(
        serde_json::to_string(&PubkySocialFeedReach::Unknown).unwrap(),
        "\"unknown\""
    );
    assert_eq!(
        serde_json::to_string(&PubkySocialFeedLayout::Unknown).unwrap(),
        "\"unknown\""
    );
    assert_eq!(
        serde_json::to_string(&PubkySocialFeedSort::Unknown).unwrap(),
        "\"unknown\""
    );
    assert_eq!(
        serde_json::to_string(&PubkySocialPostKind::Unknown).unwrap(),
        "\"unknown\""
    );
    assert_eq!(
        serde_json::to_string(&PubkySocialCollectionLayout::Unknown).unwrap(),
        "\"unknown\""
    );
}

#[test]
fn is_known_is_false_only_for_unknown() {
    use PubkySocialFeedLayout as L;
    use PubkySocialFeedReach as R;
    use PubkySocialFeedSort as S;
    for r in [
        R::Following,
        R::Followers,
        R::Friends,
        R::All,
        R::Wot,
        R::Me,
    ] {
        assert!(r.is_known());
    }
    assert!(!R::Unknown.is_known());
    for l in [L::Columns, L::Wide, L::Visual, L::List] {
        assert!(l.is_known());
    }
    assert!(!L::Unknown.is_known());
    for s in [S::Recent, S::Popularity] {
        assert!(s.is_known());
    }
    assert!(!S::Unknown.is_known());
    use PubkySocialCollectionLayout as C;
    for c in [C::Grid, C::List, C::Visual] {
        assert!(c.is_known());
    }
    assert!(!C::Unknown.is_known());
}

fn with_unknown_field(json: &str) -> String {
    let mut v: serde_json::Value = serde_json::from_str(json).unwrap();
    v.as_object_mut()
        .unwrap()
        .insert("__future_field".into(), serde_json::json!({"x": 1}));
    serde_json::to_string(&v).unwrap()
}

fn reads_with_unknown_field<T: DeserializeOwned>(json: &str) {
    serde_json::from_str::<T>(&with_unknown_field(json)).unwrap();
}

#[test]
fn every_json_wire_type_ignores_unknown_fields() {
    let post_uri = format!("pubky://{PK}/pub/pubky.app/posts/0032SSN7Q4EVG");
    reads_with_unknown_field::<PubkySocialUser>(r#"{"name":"Alice"}"#);
    reads_with_unknown_field::<PubkySocialUserLink>(
        r#"{"title":"site","url":"https://example.com/"}"#,
    );
    reads_with_unknown_field::<PubkySocialPost>(
        r#"{"content":"hello","kind":"short","parent":null,"embed":null,"attachments":null}"#,
    );
    reads_with_unknown_field::<PubkySocialPostEmbed>(&format!(
        r#"{{"kind":"short","uri":"{post_uri}"}}"#
    ));
    reads_with_unknown_field::<PubkySocialCollectionContent>(r#"{"name":"Photos","items":[]}"#);
    reads_with_unknown_field::<PubkySocialTag>(&format!(
        r#"{{"uri":"{post_uri}","label":"rust","created_at":1727740800000000}}"#
    ));
    reads_with_unknown_field::<PubkySocialBookmark>(&format!(
        r#"{{"uri":"{post_uri}","created_at":1727740800000000}}"#
    ));
    reads_with_unknown_field::<PubkySocialFollow>(r#"{"created_at":1727740800000000}"#);
    reads_with_unknown_field::<PubkySocialMute>(r#"{"created_at":1727740800000000}"#);
    reads_with_unknown_field::<PubkySocialFeedConfig>(&config("all", "list", "popularity", "null"));
    reads_with_unknown_field::<PubkySocialFeed>(&format!(
        r#"{{"feed":{},"name":"All","created_at":1727740800000000}}"#,
        config("all", "list", "popularity", "null")
    ));
    reads_with_unknown_field::<PubkySocialFile>(&format!(
        r#"{{"name":"cat.jpg","created_at":1727740800000000,"src":"pubky://{PK}/pub/pubky.app/blobs/8Z8CWH8NVYQY39ZEBFGKQWWEKG","content_type":"image/jpeg","size":1234}}"#
    ));
    reads_with_unknown_field::<PubkySocialLastRead>(r#"{"timestamp":1727740800000000}"#);
}
