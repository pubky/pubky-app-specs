use crate::{
    common::timestamp,
    limits::VALIDATION_LIMITS,
    models::tag::{sanitize_tag_label, validate_tag_label},
    traits::{HasIdPath, HashId, Validatable},
    PubkySocialPostKind, APP_PATH, PUBLIC_PATH,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[cfg(target_arch = "wasm32")]
use crate::traits::Json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Enum representing the reach of the feed.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum PubkySocialFeedReach {
    Following,
    Followers,
    Friends,
    All,
    Wot,
    Me,
    #[serde(other)]
    Unknown,
}

impl PubkySocialFeedReach {
    /// `false` only for the `Unknown` catch-all a newer writer's value lands in.
    pub fn is_known(&self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

/// Enum representing the layout of the feed.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum PubkySocialFeedLayout {
    Columns,
    Wide,
    Visual,
    List,
    #[serde(other)]
    Unknown,
}

impl PubkySocialFeedLayout {
    /// `false` only for the `Unknown` catch-all a newer writer's value lands in.
    pub fn is_known(&self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

/// Enum representing the sort order of the feed.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum PubkySocialFeedSort {
    Recent,
    Popularity,
    #[serde(other)]
    Unknown,
}

impl PubkySocialFeedSort {
    /// `false` only for the `Unknown` catch-all a newer writer's value lands in.
    pub fn is_known(&self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

/// Configuration object for the feed.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkySocialFeedConfig {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub domain_tags: Option<Vec<String>>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub reach: PubkySocialFeedReach,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub layout: PubkySocialFeedLayout,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub sort: PubkySocialFeedSort,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub content: Option<PubkySocialPostKind>,
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkySocialFeedConfig {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = fromJson))]
    pub fn from_json(js_value: &JsValue) -> Result<Self, String> {
        Self::import_json(js_value)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = toJson))]
    pub fn to_json(&self) -> Result<JsValue, String> {
        self.export_json()
    }

    /// Getter for `tags`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn tags(&self) -> Option<Vec<String>> {
        self.tags.clone()
    }

    /// Getter for `domain_tags`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn domain_tags(&self) -> Option<Vec<String>> {
        self.domain_tags.clone()
    }

    /// Getter for `name`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn reach(&self) -> PubkySocialFeedReach {
        self.reach.clone()
    }

    /// Getter for `layout`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn layout(&self) -> PubkySocialFeedLayout {
        self.layout.clone()
    }

    /// Getter for `sort`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn sort(&self) -> PubkySocialFeedSort {
        self.sort.clone()
    }

    /// Getter for `content`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn content(&self) -> Option<PubkySocialPostKind> {
        self.content.clone()
    }
}

fn sanitize_tag_list(tags: Option<Vec<String>>) -> Option<Vec<String>> {
    tags.map(|tags| {
        tags.into_iter()
            .map(|tag| sanitize_tag_label(&tag))
            .filter(|tag| !tag.is_empty())
            .collect()
    })
}

fn sanitize_feed_icon(icon: Option<String>) -> Option<String> {
    icon.map(|icon| icon.trim().to_lowercase())
}

/// Only the shape of the name is validated, not whether the icon exists: the
/// icon set is curated by the client.
///
/// `None` is accepted for feeds created before the field existed; new feeds
/// always carry one, since [`PubkySocialFeed::new`] requires it.
fn validate_feed_icon(icon: &Option<String>) -> Result<(), String> {
    let Some(icon) = icon else {
        return Ok(());
    };

    if icon.trim().is_empty() {
        return Err("Validation Error: Feed icon cannot be empty".into());
    }

    let icon_len = icon.chars().count();
    if icon_len > VALIDATION_LIMITS.feed_icon_max_length {
        return Err(format!(
            "Validation Error: Feed icon '{}' exceeds maximum length of {} characters",
            icon, VALIDATION_LIMITS.feed_icon_max_length
        ));
    }

    if let Some(c) = icon
        .chars()
        .find(|c| !c.is_ascii_lowercase() && !c.is_ascii_digit() && *c != '-')
    {
        return Err(format!(
            "Validation Error: Feed icon '{}' contains invalid character: {}",
            icon, c
        ));
    }

    Ok(())
}

fn validate_tag_list(tags: &Option<Vec<String>>, field_name: &str) -> Result<(), String> {
    if let Some(tags) = tags {
        if tags.len() > VALIDATION_LIMITS.feed_tags_max_count {
            return Err(format!(
                "Validation Error: Feed config cannot have more than {} {}",
                VALIDATION_LIMITS.feed_tags_max_count, field_name
            ));
        }

        for tag in tags {
            validate_tag_label(tag)?;
        }
    }

    Ok(())
}

impl Validatable for PubkySocialFeedConfig {
    fn sanitize(self) -> Self {
        let tags = sanitize_tag_list(self.tags);
        let domain_tags = sanitize_tag_list(self.domain_tags);

        PubkySocialFeedConfig {
            tags,
            domain_tags,
            ..self
        }
    }

    fn validate(&self, _id: Option<&str>) -> Result<(), String> {
        // reach, layout and sort define the feed, so an unknown value rejects it.
        // An unknown content filter only degrades to "no filter", so it passes.
        if !self.reach.is_known() {
            return Err("Validation Error: feed reach is unknown".into());
        }
        if !self.layout.is_known() {
            return Err("Validation Error: feed layout is unknown".into());
        }
        if !self.sort.is_known() {
            return Err("Validation Error: feed sort is unknown".into());
        }
        validate_tag_list(&self.tags, "tags")?;
        validate_tag_list(&self.domain_tags, "domain_tags")?;

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl Json for PubkySocialFeedConfig {}

/// Represents a feed configuration.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkySocialFeed {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub feed: PubkySocialFeedConfig,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub name: String,
    /// Lucide icon name, e.g. `"bitcoin"`. Required on new feeds, but optional
    /// on the wire: feeds created before this field existed have none, and
    /// clients render their default icon for those. Not part of the `feed_id`,
    /// so the icon can change without recreating the feed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub icon: Option<String>,
    pub created_at: i64,
}

impl PubkySocialFeed {
    /// Creates a new `PubkySocialFeed` instance and sanitizes it.
    pub fn new(feed: PubkySocialFeedConfig, name: String, icon: String) -> Self {
        let created_at = timestamp();
        Self {
            feed,
            name,
            icon: Some(icon),
            created_at,
        }
        .sanitize()
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkySocialFeed {
    /// Serialize to JSON for WASM.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = fromJson))]
    pub fn from_json(js_value: &JsValue) -> Result<Self, String> {
        Self::import_json(js_value)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = toJson))]
    pub fn to_json(&self) -> Result<JsValue, String> {
        self.export_json()
    }

    /// Getter for `feed`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn feed(&self) -> PubkySocialFeedConfig {
        self.feed.clone()
    }

    /// Getter for `name`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Getter for `icon`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn icon(&self) -> Option<String> {
        self.icon.clone()
    }
}

#[cfg(target_arch = "wasm32")]
impl Json for PubkySocialFeed {}

impl HashId for PubkySocialFeed {
    /// Generates an ID based on the serialized `feed` object.
    fn get_id_data(&self) -> String {
        serde_json::to_string(&self.feed).unwrap_or_default()
    }
}

impl HasIdPath for PubkySocialFeed {
    const PATH_SEGMENT: &'static str = "feeds/";

    fn create_path(id: &str) -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT, id].concat()
    }
}

impl Validatable for PubkySocialFeed {
    fn validate(&self, id: Option<&str>) -> Result<(), String> {
        // Content first, so an unrecognized value is reported as such and not as an id mismatch
        self.feed.validate(None)?;

        if self.name.trim().is_empty() {
            return Err("Validation Error: Feed name cannot be empty".into());
        }

        validate_feed_icon(&self.icon)?;

        if let Some(id) = id {
            self.validate_id(id)?;
        }

        Ok(())
    }

    fn sanitize(self) -> Self {
        PubkySocialFeed {
            feed: self.feed.sanitize(),
            name: self.name.trim().to_string(),
            icon: sanitize_feed_icon(self.icon),
            ..self
        }
    }
}

impl FromStr for PubkySocialFeedReach {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "following" => Ok(PubkySocialFeedReach::Following),
            "followers" => Ok(PubkySocialFeedReach::Followers),
            "friends" => Ok(PubkySocialFeedReach::Friends),
            "all" => Ok(PubkySocialFeedReach::All),
            "wot" => Ok(PubkySocialFeedReach::Wot),
            "me" => Ok(PubkySocialFeedReach::Me),
            _ => Err(format!("Invalid feed reach: {}", s)),
        }
    }
}

impl FromStr for PubkySocialFeedLayout {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "columns" => Ok(PubkySocialFeedLayout::Columns),
            "wide" => Ok(PubkySocialFeedLayout::Wide),
            "visual" => Ok(PubkySocialFeedLayout::Visual),
            "list" => Ok(PubkySocialFeedLayout::List),
            _ => Err(format!("Invalid feed layout: {}", s)),
        }
    }
}

impl FromStr for PubkySocialFeedSort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "recent" => Ok(PubkySocialFeedSort::Recent),
            "popularity" => Ok(PubkySocialFeedSort::Popularity),
            _ => Err(format!("Invalid feed sort: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{limits::VALIDATION_LIMITS, traits::Validatable};

    fn feed_config(
        tags: Option<Vec<String>>,
        domain_tags: Option<Vec<String>>,
        reach: PubkySocialFeedReach,
        layout: PubkySocialFeedLayout,
        sort: PubkySocialFeedSort,
        content: Option<PubkySocialPostKind>,
    ) -> PubkySocialFeedConfig {
        PubkySocialFeedConfig {
            tags,
            domain_tags,
            reach,
            layout,
            sort,
            content,
        }
    }

    #[test]
    fn test_new() {
        let feed = PubkySocialFeed::new(
            feed_config(
                Some(vec!["bitcoin".to_string(), "rust".to_string()]),
                None,
                PubkySocialFeedReach::Following,
                PubkySocialFeedLayout::Columns,
                PubkySocialFeedSort::Recent,
                Some(PubkySocialPostKind::Image),
            ),
            "Rust Bitcoiners".to_string(),
            "bitcoin".to_string(),
        );

        let feed_config = PubkySocialFeedConfig {
            tags: Some(vec!["bitcoin".to_string(), "rust".to_string()]),
            domain_tags: None,
            reach: PubkySocialFeedReach::Following,
            layout: PubkySocialFeedLayout::Columns,
            sort: PubkySocialFeedSort::Recent,
            content: Some(PubkySocialPostKind::Image),
        };
        assert_eq!(feed.feed, feed_config);
        assert_eq!(feed.name, "Rust Bitcoiners");
        assert_eq!(feed.icon, Some("bitcoin".to_string()));
        // Check that created_at is recent
        let now = timestamp();
        assert!(feed.created_at <= now && feed.created_at >= now - 1_000_000);
    }

    #[test]
    fn test_create_id() {
        let feed = PubkySocialFeed::new(
            feed_config(
                Some(vec!["bitcoin".to_string(), "rust".to_string()]),
                None,
                PubkySocialFeedReach::Following,
                PubkySocialFeedLayout::Columns,
                PubkySocialFeedSort::Recent,
                None,
            ),
            "Rust Bitcoiners".to_string(),
            "bitcoin".to_string(),
        );

        let feed_id = feed.create_id();
        println!("Feed ID: {}", feed_id);
        // The ID should not be empty
        assert!(!feed_id.is_empty());
    }

    #[test]
    fn test_validate() {
        let feed = PubkySocialFeed::new(
            feed_config(
                Some(vec!["bitcoin".to_string(), "rust".to_string()]),
                None,
                PubkySocialFeedReach::Following,
                PubkySocialFeedLayout::Columns,
                PubkySocialFeedSort::Recent,
                None,
            ),
            "Rust Bitcoiners".to_string(),
            "bitcoin".to_string(),
        );
        let feed_id = feed.create_id();

        let result = feed.validate(Some(&feed_id));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_id() {
        let feed = PubkySocialFeed::new(
            feed_config(
                Some(vec!["bitcoin".to_string(), "rust".to_string()]),
                None,
                PubkySocialFeedReach::Following,
                PubkySocialFeedLayout::Columns,
                PubkySocialFeedSort::Recent,
                None,
            ),
            "Rust Bitcoiners".to_string(),
            "bitcoin".to_string(),
        );
        let invalid_id = "INVALIDID";
        let result = feed.validate(Some(invalid_id));
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize() {
        let feed = PubkySocialFeed::new(
            feed_config(
                Some(vec!["  BiTcoin  ".to_string(), " RUST   ".to_string()]),
                None,
                PubkySocialFeedReach::Following,
                PubkySocialFeedLayout::Columns,
                PubkySocialFeedSort::Recent,
                None,
            ),
            "  Rust Bitcoiners".to_string(),
            "  BitCoin  ".to_string(),
        );
        assert_eq!(feed.name, "Rust Bitcoiners");
        assert_eq!(feed.icon, Some("bitcoin".to_string()));
        assert_eq!(
            feed.feed.tags,
            Some(vec!["bitcoin".to_string(), "rust".to_string()])
        );
    }

    #[test]
    fn test_try_from_valid() {
        let feed_json = r#"
        {
            "feed": {
                "tags": ["bitcoin", "rust"],
                "reach": "following",
                "layout": "columns",
                "sort": "recent",
                "content": "video"
            },
            "name": "My Feed",
            "created_at": 1700000000
        }
        "#;

        let feed: PubkySocialFeed = serde_json::from_str(feed_json).unwrap();
        let feed_id = feed.create_id();

        let blob = feed_json.as_bytes();
        let feed_parsed = <PubkySocialFeed as Validatable>::try_from(blob, &feed_id).unwrap();

        assert_eq!(feed_parsed.name, "My Feed");
        assert_eq!(
            feed_parsed.feed.tags,
            Some(vec!["bitcoin".to_string(), "rust".to_string()])
        );
        assert_eq!(feed_parsed.feed.domain_tags, None);
    }

    #[test]
    fn test_domain_tags_json_roundtrip() {
        let feed_json = r#"
        {
            "feed": {
                "tags": ["rust"],
                "domain_tags": ["synonym"],
                "reach": "wot",
                "layout": "columns",
                "sort": "recent"
            },
            "name": "WoT Feed",
            "created_at": 1700000000
        }
        "#;

        let feed: PubkySocialFeed = serde_json::from_str(feed_json).unwrap();
        assert_eq!(feed.feed.reach, PubkySocialFeedReach::Wot);
        assert_eq!(feed.feed.domain_tags, Some(vec!["synonym".to_string()]));
    }

    #[test]
    fn test_sanitize_domain_tags() {
        let feed = PubkySocialFeed::new(
            feed_config(
                None,
                Some(vec!["  Synonym  ".to_string(), "  ".to_string()]),
                PubkySocialFeedReach::Wot,
                PubkySocialFeedLayout::Columns,
                PubkySocialFeedSort::Recent,
                None,
            ),
            "Test Feed".to_string(),
            "rss".to_string(),
        );

        assert_eq!(feed.feed.domain_tags, Some(vec!["synonym".to_string()]));
    }

    #[test]
    fn test_validate_too_many_domain_tags() {
        let feed = PubkySocialFeed::new(
            feed_config(
                None,
                Some(vec![
                    "tag1".to_string(),
                    "tag2".to_string(),
                    "tag3".to_string(),
                    "tag4".to_string(),
                    "tag5".to_string(),
                    "tag6".to_string(),
                ]),
                PubkySocialFeedReach::Me,
                PubkySocialFeedLayout::Columns,
                PubkySocialFeedSort::Recent,
                None,
            ),
            "Test Feed".to_string(),
            "rss".to_string(),
        );
        let feed_id = feed.create_id();

        let result = feed.validate(Some(&feed_id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("domain_tags"));
    }

    #[test]
    fn test_validate_domain_tag_with_invalid_char() {
        let feed = PubkySocialFeed::new(
            feed_config(
                None,
                Some(vec!["synonym,to".to_string()]),
                PubkySocialFeedReach::Wot,
                PubkySocialFeedLayout::Columns,
                PubkySocialFeedSort::Recent,
                None,
            ),
            "Test Feed".to_string(),
            "rss".to_string(),
        );
        let feed_id = feed.create_id();

        let result = feed.validate(Some(&feed_id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid character"));
    }

    #[test]
    fn test_validate_too_many_tags() {
        let feed = PubkySocialFeed::new(
            feed_config(
                Some(vec![
                    "tag1".to_string(),
                    "tag2".to_string(),
                    "tag3".to_string(),
                    "tag4".to_string(),
                    "tag5".to_string(),
                    "tag6".to_string(), // This exceeds feed_tags_max_count
                ]),
                None,
                PubkySocialFeedReach::Following,
                PubkySocialFeedLayout::Columns,
                PubkySocialFeedSort::Recent,
                None,
            ),
            "Test Feed".to_string(),
            "rss".to_string(),
        );
        let feed_id = feed.create_id();

        let result = feed.validate(Some(&feed_id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains(&format!(
            "more than {} tags",
            VALIDATION_LIMITS.feed_tags_max_count
        )));
    }

    #[test]
    fn test_validate_tag_too_long() {
        let feed = PubkySocialFeed::new(
            feed_config(
                Some(vec!["a".repeat(VALIDATION_LIMITS.tag_label_max_length + 1)]),
                None,
                PubkySocialFeedReach::Following,
                PubkySocialFeedLayout::Columns,
                PubkySocialFeedSort::Recent,
                None,
            ),
            "Test Feed".to_string(),
            "rss".to_string(),
        );
        let feed_id = feed.create_id();

        let result = feed.validate(Some(&feed_id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds maximum length"));
    }

    #[test]
    fn test_validate_tag_with_whitespace() {
        let feed = PubkySocialFeed::new(
            feed_config(
                Some(vec!["bit coin".to_string()]),
                None,
                PubkySocialFeedReach::Following,
                PubkySocialFeedLayout::Columns,
                PubkySocialFeedSort::Recent,
                None,
            ),
            "Test Feed".to_string(),
            "rss".to_string(),
        );
        let feed_id = feed.create_id();

        let result = feed.validate(Some(&feed_id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("whitespace"));
    }

    #[test]
    fn test_validate_tag_with_invalid_char() {
        let feed = PubkySocialFeed::new(
            feed_config(
                Some(vec!["bitcoin,rust".to_string()]),
                None,
                PubkySocialFeedReach::Following,
                PubkySocialFeedLayout::Columns,
                PubkySocialFeedSort::Recent,
                None,
            ),
            "Test Feed".to_string(),
            "rss".to_string(),
        );
        let feed_id = feed.create_id();

        let result = feed.validate(Some(&feed_id));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid character"));
    }

    #[test]
    fn test_validate_max_tags() {
        let feed = PubkySocialFeed::new(
            feed_config(
                Some(vec![
                    "tag1".to_string(),
                    "tag2".to_string(),
                    "tag3".to_string(),
                    "tag4".to_string(),
                    "tag5".to_string(),
                ]),
                None,
                PubkySocialFeedReach::Following,
                PubkySocialFeedLayout::Columns,
                PubkySocialFeedSort::Recent,
                None,
            ),
            "Test Feed".to_string(),
            "rss".to_string(),
        );
        let feed_id = feed.create_id();

        let result = feed.validate(Some(&feed_id));
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_filters_empty_tags() {
        let feed = PubkySocialFeed::new(
            feed_config(
                Some(vec![
                    "bitcoin".to_string(),
                    "  ".to_string(), // Empty after trim
                    "rust".to_string(),
                ]),
                None,
                PubkySocialFeedReach::Following,
                PubkySocialFeedLayout::Columns,
                PubkySocialFeedSort::Recent,
                None,
            ),
            "Test Feed".to_string(),
            "rss".to_string(),
        );

        assert_eq!(
            feed.feed.tags,
            Some(vec!["bitcoin".to_string(), "rust".to_string()])
        );
    }

    #[test]
    fn test_validate_tag_errors() {
        // Test multiple tag validation errors in one test
        let invalid_cases = vec![
            (
                "a".repeat(VALIDATION_LIMITS.tag_label_max_length + 1),
                "exceeds maximum length",
            ),
            ("bit coin".to_string(), "whitespace"),
            ("bitcoin,rust".to_string(), "invalid character"),
        ];

        for (invalid_tag, expected_error) in invalid_cases {
            let feed = PubkySocialFeed::new(
                feed_config(
                    Some(vec![invalid_tag.clone()]),
                    None,
                    PubkySocialFeedReach::Following,
                    PubkySocialFeedLayout::Columns,
                    PubkySocialFeedSort::Recent,
                    None,
                ),
                "Test Feed".to_string(),
                "rss".to_string(),
            );
            let feed_id = feed.create_id();

            let result = feed.validate(Some(&feed_id));
            assert!(result.is_err(), "Should reject tag: {}", invalid_tag);
            assert!(
                result.unwrap_err().contains(expected_error),
                "Expected error containing '{}' for tag: {}",
                expected_error,
                invalid_tag
            );
        }
    }

    #[test]
    fn test_icon_json_roundtrip() {
        let feed_json = r#"
        {
            "feed": {
                "tags": ["rust"],
                "reach": "all",
                "layout": "columns",
                "sort": "recent"
            },
            "name": "Rust",
            "icon": "code-2",
            "created_at": 1700000000
        }
        "#;

        let feed: PubkySocialFeed = serde_json::from_str(feed_json).unwrap();
        let feed_id = feed.create_id();

        let feed_parsed =
            <PubkySocialFeed as Validatable>::try_from(feed_json.as_bytes(), &feed_id).unwrap();
        assert_eq!(feed_parsed.icon, Some("code-2".to_string()));

        let serialized = serde_json::to_value(&feed_parsed).unwrap();
        assert_eq!(serialized["icon"], "code-2");
    }

    #[test]
    fn test_feed_without_icon_stays_valid() {
        // Feeds stored before `icon` existed carry no icon and must keep
        // parsing, validating and serializing without one.
        let feed_json = r#"
        {
            "feed": {
                "tags": ["rust"],
                "reach": "all",
                "layout": "columns",
                "sort": "recent"
            },
            "name": "Legacy Feed",
            "created_at": 1700000000
        }
        "#;

        let feed: PubkySocialFeed = serde_json::from_str(feed_json).unwrap();
        let feed_id = feed.create_id();

        let feed_parsed =
            <PubkySocialFeed as Validatable>::try_from(feed_json.as_bytes(), &feed_id).unwrap();
        assert_eq!(feed_parsed.icon, None);

        let serialized = serde_json::to_value(&feed_parsed).unwrap();
        assert!(serialized.get("icon").is_none());
    }

    #[test]
    fn test_feed_with_null_icon_stays_valid() {
        let feed_json = r#"
        {
            "feed": {
                "tags": ["rust"],
                "reach": "all",
                "layout": "columns",
                "sort": "recent"
            },
            "name": "Legacy Feed",
            "icon": null,
            "created_at": 1700000000
        }
        "#;

        let feed: PubkySocialFeed = serde_json::from_str(feed_json).unwrap();
        let feed_id = feed.create_id();
        let feed_parsed =
            <PubkySocialFeed as Validatable>::try_from(feed_json.as_bytes(), &feed_id).unwrap();

        assert_eq!(feed_parsed.icon, None);
        let serialized = serde_json::to_value(&feed_parsed).unwrap();
        assert!(serialized.get("icon").is_none());
    }

    #[test]
    fn test_sanitize_icon_lowercases() {
        let feed = PubkySocialFeed::new(
            feed_config(
                None,
                None,
                PubkySocialFeedReach::All,
                PubkySocialFeedLayout::Columns,
                PubkySocialFeedSort::Recent,
                None,
            ),
            "Mixed Case Icon".to_string(),
            "  Code-2  ".to_string(),
        );

        assert_eq!(feed.icon, Some("code-2".to_string()));
        let feed_id = feed.create_id();
        assert!(feed.validate(Some(&feed_id)).is_ok());
    }

    #[test]
    fn test_validate_icon_errors() {
        let invalid_cases = vec![
            ("   ".to_string(), "cannot be empty"),
            (
                "a".repeat(VALIDATION_LIMITS.feed_icon_max_length + 1),
                "exceeds maximum length",
            ),
            ("bit coin".to_string(), "invalid character"),
            ("bitcoin,rust".to_string(), "invalid character"),
            ("bitcoin_rust".to_string(), "invalid character"),
        ];

        for (invalid_icon, expected_error) in invalid_cases {
            let feed = PubkySocialFeed::new(
                feed_config(
                    None,
                    None,
                    PubkySocialFeedReach::All,
                    PubkySocialFeedLayout::Columns,
                    PubkySocialFeedSort::Recent,
                    None,
                ),
                "Test Feed".to_string(),
                invalid_icon.clone(),
            );
            let feed_id = feed.create_id();

            let result = feed.validate(Some(&feed_id));
            assert!(result.is_err(), "Should reject icon: {}", invalid_icon);
            assert!(
                result.unwrap_err().contains(expected_error),
                "Expected error containing '{}' for icon: {}",
                expected_error,
                invalid_icon
            );
        }
    }

    #[test]
    fn test_validate_icon_at_max_length() {
        let feed = PubkySocialFeed::new(
            feed_config(
                None,
                None,
                PubkySocialFeedReach::All,
                PubkySocialFeedLayout::Columns,
                PubkySocialFeedSort::Recent,
                None,
            ),
            "Test Feed".to_string(),
            "a".repeat(VALIDATION_LIMITS.feed_icon_max_length),
        );
        let feed_id = feed.create_id();

        assert!(feed.validate(Some(&feed_id)).is_ok());
    }

    #[test]
    fn test_validate_accepts_unknown_icon_name() {
        // Only the shape is validated; clients fall back to their default icon
        // for names outside their set.
        let feed = PubkySocialFeed::new(
            feed_config(
                None,
                None,
                PubkySocialFeedReach::All,
                PubkySocialFeedLayout::Columns,
                PubkySocialFeedSort::Recent,
                None,
            ),
            "Test Feed".to_string(),
            "no-such-icon-42".to_string(),
        );
        let feed_id = feed.create_id();

        assert!(feed.validate(Some(&feed_id)).is_ok());
        assert_eq!(feed.icon, Some("no-such-icon-42".to_string()));
    }

    #[test]
    fn test_icon_does_not_change_feed_id() {
        let make_feed = |icon: &str| {
            PubkySocialFeed::new(
                feed_config(
                    Some(vec!["rust".to_string()]),
                    None,
                    PubkySocialFeedReach::All,
                    PubkySocialFeedLayout::Columns,
                    PubkySocialFeedSort::Recent,
                    None,
                ),
                "Rust".to_string(),
                icon.to_string(),
            )
        };

        assert_eq!(
            make_feed("bitcoin").create_id(),
            make_feed("rss").create_id()
        );
    }

    #[test]
    fn test_feed_reach_from_str() {
        // Valid cases
        assert_eq!(
            "following".parse::<PubkySocialFeedReach>().unwrap(),
            PubkySocialFeedReach::Following
        );
        assert_eq!(
            "followers".parse::<PubkySocialFeedReach>().unwrap(),
            PubkySocialFeedReach::Followers
        );
        assert_eq!(
            "friends".parse::<PubkySocialFeedReach>().unwrap(),
            PubkySocialFeedReach::Friends
        );
        assert_eq!(
            "all".parse::<PubkySocialFeedReach>().unwrap(),
            PubkySocialFeedReach::All
        );
        assert_eq!(
            "wot".parse::<PubkySocialFeedReach>().unwrap(),
            PubkySocialFeedReach::Wot
        );
        assert_eq!(
            "me".parse::<PubkySocialFeedReach>().unwrap(),
            PubkySocialFeedReach::Me
        );

        // Invalid case
        assert!("invalid".parse::<PubkySocialFeedReach>().is_err());
    }

    #[test]
    fn test_feed_layout_from_str() {
        // Valid cases
        assert_eq!(
            "columns".parse::<PubkySocialFeedLayout>().unwrap(),
            PubkySocialFeedLayout::Columns
        );
        assert_eq!(
            "wide".parse::<PubkySocialFeedLayout>().unwrap(),
            PubkySocialFeedLayout::Wide
        );
        assert_eq!(
            "visual".parse::<PubkySocialFeedLayout>().unwrap(),
            PubkySocialFeedLayout::Visual
        );
        assert_eq!(
            "list".parse::<PubkySocialFeedLayout>().unwrap(),
            PubkySocialFeedLayout::List
        );

        // Invalid case
        assert!("invalid".parse::<PubkySocialFeedLayout>().is_err());
    }

    #[test]
    fn test_feed_sort_from_str() {
        // Valid cases
        assert_eq!(
            "recent".parse::<PubkySocialFeedSort>().unwrap(),
            PubkySocialFeedSort::Recent
        );
        assert_eq!(
            "popularity".parse::<PubkySocialFeedSort>().unwrap(),
            PubkySocialFeedSort::Popularity
        );

        // Invalid case
        assert!("invalid".parse::<PubkySocialFeedSort>().is_err());
    }
}
