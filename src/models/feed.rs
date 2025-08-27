use crate::{
    common::timestamp,
    traits::{HasIdPath, HashId, Validatable},
    PubkyAppPostKind, APP_PATH, PUBLIC_PATH,
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
pub enum PubkyAppFeedReach {
    Following,
    Followers,
    Friends,
    All,
}

/// Enum representing the layout of the feed.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum PubkyAppFeedLayout {
    Columns,
    Wide,
    Visual,
}

/// Enum representing the sort order of the feed.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum PubkyAppFeedSort {
    Recent,
    Popularity,
}

/// Configuration object for the feed.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppFeedConfig {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub tags: Option<Vec<String>>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub reach: PubkyAppFeedReach,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub layout: PubkyAppFeedLayout,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub sort: PubkyAppFeedSort,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub content: Option<PubkyAppPostKind>,
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppFeedConfig {
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

    /// Getter for `name`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn reach(&self) -> PubkyAppFeedReach {
        self.reach.clone()
    }

    /// Getter for `layout`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn layout(&self) -> PubkyAppFeedLayout {
        self.layout.clone()
    }

    /// Getter for `sort`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn sort(&self) -> PubkyAppFeedSort {
        self.sort.clone()
    }

    /// Getter for `content`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn content(&self) -> Option<PubkyAppPostKind> {
        self.content.clone()
    }
}

impl Validatable for PubkyAppFeedConfig {
    fn validate(&self, _id: Option<&str>) -> Result<(), String> {
        // TODO: validate config?
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl Json for PubkyAppFeedConfig {}

/// Represents a feed configuration.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppFeed {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub feed: PubkyAppFeedConfig,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub name: String,
    pub created_at: i64,
}

impl PubkyAppFeed {
    /// Creates a new `PubkyAppFeed` instance and sanitizes it.
    pub fn new(
        tags: Option<Vec<String>>,
        reach: PubkyAppFeedReach,
        layout: PubkyAppFeedLayout,
        sort: PubkyAppFeedSort,
        content: Option<PubkyAppPostKind>,
        name: String,
    ) -> Self {
        let created_at = timestamp();
        let feed = PubkyAppFeedConfig {
            tags,
            reach,
            layout,
            sort,
            content,
        };
        Self {
            feed,
            name,
            created_at,
        }
        .sanitize()
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppFeed {
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
    pub fn feed(&self) -> PubkyAppFeedConfig {
        self.feed.clone()
    }

    /// Getter for `name`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

#[cfg(target_arch = "wasm32")]
impl Json for PubkyAppFeed {}

impl HashId for PubkyAppFeed {
    /// Generates an ID based on the serialized `feed` object.
    fn get_id_data(&self) -> String {
        serde_json::to_string(&self.feed).unwrap_or_default()
    }
}

impl HasIdPath for PubkyAppFeed {
    const PATH_SEGMENT: &'static str = "feeds/";

    fn create_path(id: &str) -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT, id].concat()
    }
}

impl Validatable for PubkyAppFeed {
    fn validate(&self, id: Option<&str>) -> Result<(), String> {
        // Validate the feed ID
        if let Some(id) = id {
            self.validate_id(id)?;
        }

        // Validate name
        if self.name.trim().is_empty() {
            return Err("Validation Error: Feed name cannot be empty".into());
        }

        // Additional validations can be added here
        Ok(())
    }

    fn sanitize(self) -> Self {
        // Sanitize name
        let name = self.name.trim().to_string();

        // Sanitize tags
        let feed = PubkyAppFeedConfig {
            tags: self.feed.tags.map(|tags| {
                tags.into_iter()
                    .map(|tag| tag.trim().to_lowercase())
                    .collect()
            }),
            ..self.feed
        };

        PubkyAppFeed {
            feed,
            name,
            created_at: self.created_at,
        }
    }
}

impl FromStr for PubkyAppFeedReach {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "following" => Ok(PubkyAppFeedReach::Following),
            "followers" => Ok(PubkyAppFeedReach::Followers),
            "friends" => Ok(PubkyAppFeedReach::Friends),
            "all" => Ok(PubkyAppFeedReach::All),
            _ => Err(format!("Invalid feed reach: {}", s)),
        }
    }
}

impl FromStr for PubkyAppFeedLayout {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "columns" => Ok(PubkyAppFeedLayout::Columns),
            "wide" => Ok(PubkyAppFeedLayout::Wide),
            "visual" => Ok(PubkyAppFeedLayout::Visual),
            _ => Err(format!("Invalid feed layout: {}", s)),
        }
    }
}

impl FromStr for PubkyAppFeedSort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "recent" => Ok(PubkyAppFeedSort::Recent),
            "popularity" => Ok(PubkyAppFeedSort::Popularity),
            _ => Err(format!("Invalid feed sort: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Validatable;

    #[test]
    fn test_new() {
        let feed = PubkyAppFeed::new(
            Some(vec!["bitcoin".to_string(), "rust".to_string()]),
            PubkyAppFeedReach::Following,
            PubkyAppFeedLayout::Columns,
            PubkyAppFeedSort::Recent,
            Some(PubkyAppPostKind::Image),
            "Rust Bitcoiners".to_string(),
        );

        let feed_config = PubkyAppFeedConfig {
            tags: Some(vec!["bitcoin".to_string(), "rust".to_string()]),
            reach: PubkyAppFeedReach::Following,
            layout: PubkyAppFeedLayout::Columns,
            sort: PubkyAppFeedSort::Recent,
            content: Some(PubkyAppPostKind::Image),
        };
        assert_eq!(feed.feed, feed_config);
        assert_eq!(feed.name, "Rust Bitcoiners");
        // Check that created_at is recent
        let now = timestamp();
        assert!(feed.created_at <= now && feed.created_at >= now - 1_000_000);
    }

    #[test]
    fn test_create_id() {
        let feed = PubkyAppFeed::new(
            Some(vec!["bitcoin".to_string(), "rust".to_string()]),
            PubkyAppFeedReach::Following,
            PubkyAppFeedLayout::Columns,
            PubkyAppFeedSort::Recent,
            None,
            "Rust Bitcoiners".to_string(),
        );

        let feed_id = feed.create_id();
        println!("Feed ID: {}", feed_id);
        // The ID should not be empty
        assert!(!feed_id.is_empty());
    }

    #[test]
    fn test_validate() {
        let feed = PubkyAppFeed::new(
            Some(vec!["bitcoin".to_string(), "rust".to_string()]),
            PubkyAppFeedReach::Following,
            PubkyAppFeedLayout::Columns,
            PubkyAppFeedSort::Recent,
            None,
            "Rust Bitcoiners".to_string(),
        );
        let feed_id = feed.create_id();

        let result = feed.validate(Some(&feed_id));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_id() {
        let feed = PubkyAppFeed::new(
            Some(vec!["bitcoin".to_string(), "rust".to_string()]),
            PubkyAppFeedReach::Following,
            PubkyAppFeedLayout::Columns,
            PubkyAppFeedSort::Recent,
            None,
            "Rust Bitcoiners".to_string(),
        );
        let invalid_id = "INVALIDID";
        let result = feed.validate(Some(invalid_id));
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize() {
        let feed = PubkyAppFeed::new(
            Some(vec!["  BiTcoin  ".to_string(), " RUST   ".to_string()]),
            PubkyAppFeedReach::Following,
            PubkyAppFeedLayout::Columns,
            PubkyAppFeedSort::Recent,
            None,
            "  Rust Bitcoiners".to_string(),
        );
        assert_eq!(feed.name, "Rust Bitcoiners");
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

        let feed: PubkyAppFeed = serde_json::from_str(feed_json).unwrap();
        let feed_id = feed.create_id();

        let blob = feed_json.as_bytes();
        let feed_parsed = <PubkyAppFeed as Validatable>::try_from(blob, &feed_id).unwrap();

        assert_eq!(feed_parsed.name, "My Feed");
        assert_eq!(
            feed_parsed.feed.tags,
            Some(vec!["bitcoin".to_string(), "rust".to_string()])
        );
    }
}
