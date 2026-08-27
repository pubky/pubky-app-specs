//! Serialized bytes of every v0 wire model, pinned as literals so a rename or refactor that
//! changes a serde attribute fails here before it reaches a homeserver. Captured from the 0.8.0
//! crate (commit 3eebe18), before the rename. `{PK}` stands in for the host key.
#![cfg(not(target_arch = "wasm32"))]

use pubky_social_specs::{
    ExtendedParsedUri, ParsedUri, PubkyId, PubkySocialBlob, PubkySocialBookmark,
    PubkySocialCollectionContent, PubkySocialCollectionLayout, PubkySocialFeed,
    PubkySocialFeedConfig, PubkySocialFeedLayout, PubkySocialFeedReach, PubkySocialFeedSort,
    PubkySocialFile, PubkySocialFollow, PubkySocialLastRead, PubkySocialMute, PubkySocialPost,
    PubkySocialPostEmbed, PubkySocialPostKind, PubkySocialTag, PubkySocialUser,
    PubkySocialUserLink, Resource, VALIDATION_LIMITS,
};
use serde::Serialize;

const PK: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";
const TS: i64 = 1_727_740_800_000_000;

fn json<T: Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap()
}

fn user() -> PubkySocialUser {
    PubkySocialUser::new(
        "Alice".into(),
        Some("bio".into()),
        Some(format!("pubky://{PK}/pub/pubky.app/files/0032SSN7Q4EVG")),
        Some(vec![PubkySocialUserLink::new(
            "site".into(),
            "https://example.com".into(),
        )]),
        Some("here".into()),
    )
}

fn post_full() -> PubkySocialPost {
    PubkySocialPost::new_with_lock(
        "hello".into(),
        PubkySocialPostKind::Short,
        Some(format!("pubky://{PK}/pub/pubky.app/posts/0032SSN7Q4EVG")),
        Some(PubkySocialPostEmbed {
            uri: format!("pubky://{PK}/pub/pubky.app/posts/0034A0X7NJ52G"),
            kind: PubkySocialPostKind::Short,
        }),
        Some(vec![format!(
            "pubky://{PK}/pub/pubky.app/files/0032SSN7Q4EVG"
        )]),
        Some(format!("pubky://{PK}/pub/app.locks/0032SSN7Q4EVG.json")),
    )
}

fn post_minimal() -> PubkySocialPost {
    PubkySocialPost::new("hello".into(), PubkySocialPostKind::Long, None, None, None)
}

fn tag() -> PubkySocialTag {
    let mut t = PubkySocialTag::new(
        format!("pubky://{PK}/pub/pubky.app/posts/0032SSN7Q4EVG"),
        "rust".into(),
    );
    t.created_at = TS;
    t
}

fn bookmark() -> PubkySocialBookmark {
    let mut b = PubkySocialBookmark::new(format!("pubky://{PK}/pub/pubky.app/posts/0032SSN7Q4EVG"));
    b.created_at = TS;
    b
}

fn follow() -> PubkySocialFollow {
    let mut f = PubkySocialFollow::new();
    f.created_at = TS;
    f
}

fn mute() -> PubkySocialMute {
    let mut m = PubkySocialMute::new();
    m.created_at = TS;
    m
}

fn feed_config() -> PubkySocialFeedConfig {
    PubkySocialFeedConfig {
        tags: Some(vec!["rust".into()]),
        domain_tags: Some(vec!["dev".into()]),
        reach: PubkySocialFeedReach::Wot,
        layout: PubkySocialFeedLayout::Columns,
        sort: PubkySocialFeedSort::Recent,
        content: Some(PubkySocialPostKind::Short),
    }
}

fn feed_with_icon() -> PubkySocialFeed {
    let mut f = PubkySocialFeed::new(feed_config(), "Rust".into(), "code".into());
    f.created_at = TS;
    f
}

fn feed_legacy() -> PubkySocialFeed {
    PubkySocialFeed {
        feed: PubkySocialFeedConfig {
            tags: None,
            domain_tags: None,
            reach: PubkySocialFeedReach::All,
            layout: PubkySocialFeedLayout::List,
            sort: PubkySocialFeedSort::Popularity,
            content: None,
        },
        name: "All".into(),
        icon: None,
        created_at: TS,
    }
}

fn file() -> PubkySocialFile {
    let mut f = PubkySocialFile::new(
        "cat.jpg".into(),
        format!("pubky://{PK}/pub/pubky.app/blobs/8Z8CWH8NVYQY39ZEBFGKQWWEKG"),
        "image/jpeg".into(),
        1234,
    );
    f.created_at = TS;
    f
}

fn blob() -> PubkySocialBlob {
    PubkySocialBlob::new(vec![1, 2])
}

fn last_read() -> PubkySocialLastRead {
    let mut l = PubkySocialLastRead::new();
    l.timestamp = TS;
    l
}

fn collection_with_layout() -> PubkySocialCollectionContent {
    PubkySocialCollectionContent {
        name: "Photos".into(),
        description: Some("mine".into()),
        items: vec![format!("pubky://{PK}/pub/pubky.app/posts/0032SSN7Q4EVG")],
        cover_image: Some(format!("pubky://{PK}/pub/pubky.app/files/0032SSN7Q4EVG")),
        layout: Some(PubkySocialCollectionLayout::Visual),
    }
}

fn collection_legacy() -> PubkySocialCollectionContent {
    PubkySocialCollectionContent {
        name: "Photos".into(),
        description: None,
        items: vec![],
        cover_image: None,
        layout: None,
    }
}

/// Captured from the 0.8.0 crate (commit 3eebe18), before the rename. `PK` stands in for the host key.
fn parsed_uri() -> ParsedUri {
    ParsedUri {
        user_id: PubkyId::try_from(PK).unwrap(),
        resource: Resource::Post("0032SSN7Q4EVG".into()),
    }
}

fn extended_app() -> ExtendedParsedUri {
    ExtendedParsedUri::PubkyApp {
        user_id: PubkyId::try_from(PK).unwrap(),
        resource: Resource::User,
    }
}

fn extended_universal_tag() -> ExtendedParsedUri {
    ExtendedParsedUri::UniversalTag {
        user_id: PubkyId::try_from(PK).unwrap(),
        app: "mapky.app".into(),
        resource: Resource::Tag("8Z8CWH8NVYQY39ZEBFGKQWWEKG".into()),
    }
}

/// The parse types are not stored objects, but they derive serde, so their names are pinned too.
#[rustfmt::skip]
fn pinned() -> Vec<(&'static str, String, &'static str)> {
    vec![
        (
            "user",
            json(&user()),
            r#"{"name":"Alice","bio":"bio","image":"pubky://{PK}/pub/pubky.app/files/0032SSN7Q4EVG","links":[{"title":"site","url":"https://example.com/"}],"status":"here"}"#,
        ),
        (
            "post_full",
            json(&post_full()),
            r#"{"content":"hello","kind":"short","parent":"pubky://{PK}/pub/pubky.app/posts/0032SSN7Q4EVG","embed":{"kind":"short","uri":"pubky://{PK}/pub/pubky.app/posts/0034A0X7NJ52G"},"attachments":["pubky://{PK}/pub/pubky.app/files/0032SSN7Q4EVG"],"lock":"pubky://{PK}/pub/app.locks/0032SSN7Q4EVG.json"}"#,
        ),
        (
            "post_minimal",
            json(&post_minimal()),
            r#"{"content":"hello","kind":"long","parent":null,"embed":null,"attachments":null}"#,
        ),
        (
            "tag",
            json(&tag()),
            r#"{"uri":"pubky://{PK}/pub/pubky.app/posts/0032SSN7Q4EVG","label":"rust","created_at":1727740800000000}"#,
        ),
        (
            "bookmark",
            json(&bookmark()),
            r#"{"uri":"pubky://{PK}/pub/pubky.app/posts/0032SSN7Q4EVG","created_at":1727740800000000}"#,
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
            r#"{"name":"cat.jpg","created_at":1727740800000000,"src":"pubky://{PK}/pub/pubky.app/blobs/8Z8CWH8NVYQY39ZEBFGKQWWEKG","content_type":"image/jpeg","size":1234}"#,
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
            r#"{"name":"Photos","description":"mine","items":["pubky://{PK}/pub/pubky.app/posts/0032SSN7Q4EVG"],"cover_image":"pubky://{PK}/pub/pubky.app/files/0032SSN7Q4EVG","layout":"visual"}"#,
        ),
        (
            "collection_legacy",
            json(&collection_legacy()),
            r#"{"name":"Photos","items":[]}"#,
        ),
        (
            "parsed_uri",
            json(&parsed_uri()),
            r#"{"user_id":"{PK}","resource":{"Post":"0032SSN7Q4EVG"}}"#,
        ),
        (
            "extended_app",
            json(&extended_app()),
            r#"{"PubkyApp":{"user_id":"{PK}","resource":"User"}}"#,
        ),
        (
            "extended_universal_tag",
            json(&extended_universal_tag()),
            r#"{"UniversalTag":{"user_id":"{PK}","app":"mapky.app","resource":{"Tag":"8Z8CWH8NVYQY39ZEBFGKQWWEKG"}}}"#,
        ),
    ]
}

#[test]
fn wire_bytes_are_unchanged() {
    for (name, actual, expected) in pinned() {
        assert_eq!(actual, expected.replace("{PK}", PK), "{name}");
    }
}

fn pk() -> PubkyId {
    PubkyId::try_from(PK).unwrap()
}

/// Every variant of every serde enum, so a rename that touches a variant name fails here.
#[rustfmt::skip]
fn pinned_variants() -> Vec<(String, &'static str)> {
    use PubkySocialCollectionLayout as C;
    use PubkySocialFeedLayout as L;
    use PubkySocialFeedReach as R;
    use PubkySocialFeedSort as S;
    use PubkySocialPostKind as K;
    let h = "8Z8CWH8NVYQY39ZEBFGKQWWEKG".to_string();
    vec![
        (json(&Resource::User), r#""User""#),
        (json(&Resource::Post("0032SSN7Q4EVG".into())), r#"{"Post":"0032SSN7Q4EVG"}"#),
        (json(&Resource::Follow(pk())), r#"{"Follow":"{PK}"}"#),
        (json(&Resource::Mute(pk())), r#"{"Mute":"{PK}"}"#),
        (json(&Resource::Bookmark(h.clone())), r#"{"Bookmark":"8Z8CWH8NVYQY39ZEBFGKQWWEKG"}"#),
        (json(&Resource::Tag(h.clone())), r#"{"Tag":"8Z8CWH8NVYQY39ZEBFGKQWWEKG"}"#),
        (json(&Resource::File("0032SSN7Q4EVG".into())), r#"{"File":"0032SSN7Q4EVG"}"#),
        (json(&Resource::Blob(h.clone())), r#"{"Blob":"8Z8CWH8NVYQY39ZEBFGKQWWEKG"}"#),
        (json(&Resource::Feed(h)), r#"{"Feed":"8Z8CWH8NVYQY39ZEBFGKQWWEKG"}"#),
        (json(&Resource::LastRead), r#""LastRead""#),
        (json(&Resource::Unknown), r#""Unknown""#),
        (json(&K::Short), r#""short""#), (json(&K::Long), r#""long""#), (json(&K::Image), r#""image""#),
        (json(&K::Video), r#""video""#), (json(&K::Link), r#""link""#), (json(&K::File), r#""file""#),
        (json(&K::Collection), r#""collection""#), (json(&K::Unknown), r#""unknown""#),
        (json(&R::Following), r#""following""#), (json(&R::Followers), r#""followers""#),
        (json(&R::Friends), r#""friends""#), (json(&R::All), r#""all""#), (json(&R::Wot), r#""wot""#),
        (json(&R::Me), r#""me""#),
        (json(&L::Columns), r#""columns""#), (json(&L::Wide), r#""wide""#), (json(&L::Visual), r#""visual""#),
        (json(&L::List), r#""list""#),
        (json(&S::Recent), r#""recent""#), (json(&S::Popularity), r#""popularity""#),
        (json(&C::Grid), r#""grid""#), (json(&C::List), r#""list""#), (json(&C::Visual), r#""visual""#),
        (json(&C::Unknown), r#""unknown""#),
    ]
}

#[test]
fn every_enum_variant_serializes_as_before() {
    for (actual, expected) in pinned_variants() {
        assert_eq!(actual, expected.replace("{PK}", PK));
    }
}

/// The limits table ships to npm as `validationLimits.json`, so its keys are wire too. This is
/// the 1.0 table: renamed, added and removed rows are deliberate and consumers adopt them with 1.0.
#[test]
fn validation_limits_wire_keys_are_pinned() {
    assert_eq!(
        json(&VALIDATION_LIMITS),
        r#"{"maxFileSizeBytes":104857600,"tagLabelMinLength":1,"tagLabelMaxLength":20,"tagInvalidChars":[",",":"," ","\t","\n","\r"],"userNameMinLength":3,"userNameMaxLength":50,"userBioMaxLength":160,"imageUrlMaxLength":300,"userLinksMaxCount":5,"userLinkTitleMaxLength":100,"userLinkUrlMaxLength":300,"userStatusMaxLength":50,"postNoteContentMaxLength":2000,"articleTitleMaxLength":100,"articleBodyMaxLength":50000,"articleContentMaxLength":52000,"postAttachmentsMaxCount":10,"attachmentAltMaxLength":1000,"attachmentNameMaxLength":255,"referenceUriMaxLength":1024,"postAllowedAttachmentProtocols":["pubky","http","https"],"collectionContentMaxLength":40000,"collectionNameMinLength":1,"collectionNameMaxLength":100,"collectionDescriptionMaxLength":500,"collectionItemsMaxCount":100,"feedTagsMaxCount":5,"feedNameMaxLength":100,"feedIconMaxLength":50,"bookmarkTargetUriMaxBytes":187}"#
    );
}
