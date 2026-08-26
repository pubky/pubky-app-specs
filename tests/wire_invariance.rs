//! Serialized bytes of every v0 wire model, pinned as literals so a rename or refactor that
//! changes a serde attribute fails here before it reaches a homeserver.
#![cfg(not(target_arch = "wasm32"))]

use pubky_app_specs::{
    PubkyAppBlob, PubkyAppBookmark, PubkyAppCollectionContent, PubkyAppCollectionLayout,
    PubkyAppFeed, PubkyAppFeedConfig, PubkyAppFeedLayout, PubkyAppFeedReach, PubkyAppFeedSort,
    PubkyAppFile, PubkyAppFollow, PubkyAppLastRead, PubkyAppMute, PubkyAppPost, PubkyAppPostEmbed,
    PubkyAppPostKind, PubkyAppTag, PubkyAppUser, PubkyAppUserLink,
};
use serde::Serialize;

const PK: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";
const TS: i64 = 1_727_740_800_000_000;

fn json<T: Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap()
}

fn user() -> PubkyAppUser {
    PubkyAppUser::new(
        "Alice".into(),
        Some("bio".into()),
        Some(format!("pubky://{PK}/pub/pubky.app/files/0032SSN7Q4EVG")),
        Some(vec![PubkyAppUserLink::new(
            "site".into(),
            "https://example.com".into(),
        )]),
        Some("here".into()),
    )
}

fn post_full() -> PubkyAppPost {
    PubkyAppPost::new_with_lock(
        "hello".into(),
        PubkyAppPostKind::Short,
        Some(format!("pubky://{PK}/pub/pubky.app/posts/0032SSN7Q4EVG")),
        Some(PubkyAppPostEmbed {
            uri: format!("pubky://{PK}/pub/pubky.app/posts/0034A0X7NJ52G"),
            kind: PubkyAppPostKind::Short,
        }),
        Some(vec![format!(
            "pubky://{PK}/pub/pubky.app/files/0032SSN7Q4EVG"
        )]),
        Some(format!("pubky://{PK}/pub/app.locks/0032SSN7Q4EVG.json")),
    )
}

fn post_minimal() -> PubkyAppPost {
    PubkyAppPost::new("hello".into(), PubkyAppPostKind::Long, None, None, None)
}

fn tag() -> PubkyAppTag {
    let mut t = PubkyAppTag::new(
        format!("pubky://{PK}/pub/pubky.app/posts/0032SSN7Q4EVG"),
        "rust".into(),
    );
    t.created_at = TS;
    t
}

fn bookmark() -> PubkyAppBookmark {
    let mut b = PubkyAppBookmark::new(format!("pubky://{PK}/pub/pubky.app/posts/0032SSN7Q4EVG"));
    b.created_at = TS;
    b
}

fn follow() -> PubkyAppFollow {
    let mut f = PubkyAppFollow::new();
    f.created_at = TS;
    f
}

fn mute() -> PubkyAppMute {
    let mut m = PubkyAppMute::new();
    m.created_at = TS;
    m
}

fn feed_config() -> PubkyAppFeedConfig {
    PubkyAppFeedConfig {
        tags: Some(vec!["rust".into()]),
        domain_tags: Some(vec!["dev".into()]),
        reach: PubkyAppFeedReach::Wot,
        layout: PubkyAppFeedLayout::Columns,
        sort: PubkyAppFeedSort::Recent,
        content: Some(PubkyAppPostKind::Short),
    }
}

fn feed_with_icon() -> PubkyAppFeed {
    let mut f = PubkyAppFeed::new(feed_config(), "Rust".into(), "code".into());
    f.created_at = TS;
    f
}

fn feed_legacy() -> PubkyAppFeed {
    PubkyAppFeed {
        feed: PubkyAppFeedConfig {
            tags: None,
            domain_tags: None,
            reach: PubkyAppFeedReach::All,
            layout: PubkyAppFeedLayout::List,
            sort: PubkyAppFeedSort::Popularity,
            content: None,
        },
        name: "All".into(),
        icon: None,
        created_at: TS,
    }
}

fn file() -> PubkyAppFile {
    let mut f = PubkyAppFile::new(
        "cat.jpg".into(),
        format!("pubky://{PK}/pub/pubky.app/blobs/8Z8CWH8NVYQY39ZEBFGKQWWEKG"),
        "image/jpeg".into(),
        1234,
    );
    f.created_at = TS;
    f
}

fn blob() -> PubkyAppBlob {
    PubkyAppBlob::new(vec![1, 2])
}

fn last_read() -> PubkyAppLastRead {
    let mut l = PubkyAppLastRead::new();
    l.timestamp = TS;
    l
}

fn collection_with_layout() -> PubkyAppCollectionContent {
    PubkyAppCollectionContent {
        name: "Photos".into(),
        description: Some("mine".into()),
        items: vec![format!("pubky://{PK}/pub/pubky.app/posts/0032SSN7Q4EVG")],
        cover_image: Some(format!("pubky://{PK}/pub/pubky.app/files/0032SSN7Q4EVG")),
        layout: Some(PubkyAppCollectionLayout::Visual),
    }
}

fn collection_legacy() -> PubkyAppCollectionContent {
    PubkyAppCollectionContent {
        name: "Photos".into(),
        description: None,
        items: vec![],
        cover_image: None,
        layout: None,
    }
}

/// Captured from pubky-app-specs 0.8.0 (commit 3eebe18). `PK` stands in for the host key.
fn pinned() -> Vec<(&'static str, String, &'static str)> {
    vec![
        (
            "user",
            json(&user()),
            r#"{"name":"Alice","bio":"bio","image":"pubky://PK/pub/pubky.app/files/0032SSN7Q4EVG","links":[{"title":"site","url":"https://example.com/"}],"status":"here"}"#,
        ),
        (
            "post_full",
            json(&post_full()),
            r#"{"content":"hello","kind":"short","parent":"pubky://PK/pub/pubky.app/posts/0032SSN7Q4EVG","embed":{"kind":"short","uri":"pubky://PK/pub/pubky.app/posts/0034A0X7NJ52G"},"attachments":["pubky://PK/pub/pubky.app/files/0032SSN7Q4EVG"],"lock":"pubky://PK/pub/app.locks/0032SSN7Q4EVG.json"}"#,
        ),
        (
            "post_minimal",
            json(&post_minimal()),
            r#"{"content":"hello","kind":"long","parent":null,"embed":null,"attachments":null}"#,
        ),
        (
            "tag",
            json(&tag()),
            r#"{"uri":"pubky://PK/pub/pubky.app/posts/0032SSN7Q4EVG","label":"rust","created_at":1727740800000000}"#,
        ),
        (
            "bookmark",
            json(&bookmark()),
            r#"{"uri":"pubky://PK/pub/pubky.app/posts/0032SSN7Q4EVG","created_at":1727740800000000}"#,
        ),
        (
            "follow",
            json(&follow()),
            r#"{"created_at":1727740800000000}"#,
        ),
        ("mute", json(&mute()), r#"{"created_at":1727740800000000}"#),
        (
            "feed_with_icon",
            json(&feed_with_icon()),
            r#"{"feed":{"tags":["rust"],"domain_tags":["dev"],"reach":"wot","layout":"columns","sort":"recent","content":"short"},"name":"Rust","icon":"code","created_at":1727740800000000}"#,
        ),
        (
            "feed_legacy",
            json(&feed_legacy()),
            r#"{"feed":{"tags":null,"reach":"all","layout":"list","sort":"popularity","content":null},"name":"All","created_at":1727740800000000}"#,
        ),
        (
            "file",
            json(&file()),
            r#"{"name":"cat.jpg","created_at":1727740800000000,"src":"pubky://PK/pub/pubky.app/blobs/8Z8CWH8NVYQY39ZEBFGKQWWEKG","content_type":"image/jpeg","size":1234}"#,
        ),
        ("blob", json(&blob()), r#"[1,2]"#),
        (
            "last_read",
            json(&last_read()),
            r#"{"timestamp":1727740800000000}"#,
        ),
        (
            "collection_with_layout",
            json(&collection_with_layout()),
            r#"{"name":"Photos","description":"mine","items":["pubky://PK/pub/pubky.app/posts/0032SSN7Q4EVG"],"cover_image":"pubky://PK/pub/pubky.app/files/0032SSN7Q4EVG","layout":"visual"}"#,
        ),
        (
            "collection_legacy",
            json(&collection_legacy()),
            r#"{"name":"Photos","items":[]}"#,
        ),
    ]
}

#[test]
fn wire_bytes_are_unchanged() {
    for (name, actual, expected) in pinned() {
        assert_eq!(actual, expected.replace("PK", PK), "{name}");
    }
}
