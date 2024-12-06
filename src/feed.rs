use crate::{
    common::timestamp,
    traits::{HasPath, HashId, Validatable},
    PubkyAppPostKind, APP_PATH,
};
use serde::{Deserialize, Serialize};
use serde_json;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Enum representing the reach of the feed.
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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum PubkyAppFeedLayout {
    Columns,
    Wide,
    Visual,
}

/// Enum representing the sort order of the feed.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum PubkyAppFeedSort {
    Recent,
    Popularity,
}

/// Configuration object for the feed.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppFeedConfig {
    pub tags: Option<Vec<String>>,
    pub reach: PubkyAppFeedReach,
    pub layout: PubkyAppFeedLayout,
    pub sort: PubkyAppFeedSort,
    pub content: Option<PubkyAppPostKind>,
}

/// Represents a feed configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppFeed {
    pub feed: PubkyAppFeedConfig,
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

impl HashId for PubkyAppFeed {
    /// Generates an ID based on the serialized `feed` object.
    fn get_id_data(&self) -> String {
        serde_json::to_string(&self.feed).unwrap_or_default()
    }
}

impl HasPath for PubkyAppFeed {
    fn create_path(&self) -> String {
        format!("{}feeds/{}", APP_PATH, self.create_id())
    }
}

impl Validatable for PubkyAppFeed {
    fn validate(&self, id: &str) -> Result<(), String> {
        self.validate_id(id)?;

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

        let result = feed.validate(&feed_id);
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
        let result = feed.validate(&invalid_id);
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
        let feed_parsed = <PubkyAppFeed as Validatable>::try_from(&blob, &feed_id).unwrap();

        assert_eq!(feed_parsed.name, "My Feed");
        assert_eq!(
            feed_parsed.feed.tags,
            Some(vec!["bitcoin".to_string(), "rust".to_string()])
        );
    }
}
