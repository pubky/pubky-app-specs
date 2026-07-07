use crate::PubkyId;
use serde::{Deserialize, Serialize};
use std::convert::{From, TryFrom};

use super::tag::TagPath;
use super::super::{ParsedUri, Resource};

/// Parsed URI for ingest boundaries: strict [`super::super::ParsedUri`] paths
/// plus cross-app universal tag paths.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompatParsedUri {
    /// Standard pubky.app path.
    PubkyApp {
        user_id: PubkyId,
        resource: Resource,
    },
    /// Universal tag URI from a different app.
    UniversalTag {
        user_id: PubkyId,
        app: String,
        resource: Resource,
        tag_id: String,
    },
}

impl CompatParsedUri {
    pub fn user_id(&self) -> &PubkyId {
        match self {
            CompatParsedUri::PubkyApp { user_id, .. }
            | CompatParsedUri::UniversalTag { user_id, .. } => user_id,
        }
    }

    pub fn resource(&self) -> &Resource {
        match self {
            CompatParsedUri::PubkyApp { resource, .. }
            | CompatParsedUri::UniversalTag { resource, .. } => resource,
        }
    }

    /// Returns the app name. `"pubky.app"` for [`CompatParsedUri::PubkyApp`] variants.
    pub fn app(&self) -> &str {
        match self {
            CompatParsedUri::PubkyApp { .. } => "pubky.app",
            CompatParsedUri::UniversalTag { app, .. } => app.as_str(),
        }
    }

    /// Returns the tag ID when this is a [`CompatParsedUri::UniversalTag`] with a tag resource.
    pub fn tag_id(&self) -> Option<&str> {
        match self {
            CompatParsedUri::PubkyApp { .. } => None,
            CompatParsedUri::UniversalTag { tag_id, .. } => Some(tag_id.as_str()),
        }
    }
}

impl From<ParsedUri> for CompatParsedUri {
    fn from(parsed: ParsedUri) -> Self {
        CompatParsedUri::PubkyApp {
            user_id: parsed.user_id,
            resource: parsed.resource,
        }
    }
}

impl TryFrom<&str> for CompatParsedUri {
    type Error = String;

    fn try_from(uri: &str) -> Result<Self, Self::Error> {
        if let Ok(parsed_uri) = ParsedUri::try_from(uri) {
            return Ok(parsed_uri.into());
        }

        if let Some(parsed) = TagPath::parse(uri) {
            return Ok(CompatParsedUri::UniversalTag {
                user_id: parsed.user_id,
                app: parsed.app,
                resource: Resource::Tag(parsed.tag_id.clone()),
                tag_id: parsed.tag_id,
            });
        }

        Err(format!(
            "URI is not a recognized pubky.app path or universal tag path: {uri}"
        ))
    }
}

impl TryFrom<String> for CompatParsedUri {
    type Error = String;

    fn try_from(uri: String) -> Result<Self, Self::Error> {
        CompatParsedUri::try_from(uri.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const USER_ID: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";

    #[test]
    fn test_parse_standard_post_uri() {
        let post_id = "0032SSN7Q4EVG";
        let uri = format!("pubky://{USER_ID}/pub/pubky.app/posts/{post_id}");
        let parsed = CompatParsedUri::try_from(uri.as_str()).expect("post URI should parse");

        assert!(matches!(parsed, CompatParsedUri::PubkyApp { .. }));
        assert_eq!(parsed.resource(), &Resource::Post(post_id.to_string()));
        assert_eq!(parsed.app(), "pubky.app");
    }

    #[test]
    fn test_parse_standard_tag_uri() {
        let tag_id = "8Z8CWH8NVYQY39ZEBFGKQWWEKG";
        let uri = format!("pubky://{USER_ID}/pub/pubky.app/tags/{tag_id}");
        let parsed = CompatParsedUri::try_from(uri.as_str()).expect("tag URI should parse");

        assert!(matches!(parsed, CompatParsedUri::PubkyApp { .. }));
        assert_eq!(parsed.resource(), &Resource::Tag(tag_id.to_string()));
        assert_eq!(parsed.app(), "pubky.app");
        assert_eq!(parsed.tag_id(), None);
    }

    #[test]
    fn test_parse_universal_tag_uri_mapky() {
        let tag_id = "ABC123";
        let uri = format!("pubky://{USER_ID}/pub/mapky/tags/{tag_id}");
        let parsed =
            CompatParsedUri::try_from(uri.as_str()).expect("mapky tag URI should parse");

        assert!(matches!(parsed, CompatParsedUri::UniversalTag { .. }));
        assert_eq!(parsed.user_id(), &PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(parsed.app(), "mapky");
        assert_eq!(parsed.resource(), &Resource::Tag(tag_id.to_string()));
        assert_eq!(parsed.tag_id(), Some(tag_id));
    }

    #[test]
    fn test_parse_universal_tag_uri_eventky() {
        let tag_id = "XYZ789";
        let uri = format!("pubky://{USER_ID}/pub/eventky.app/tags/{tag_id}");
        let parsed =
            CompatParsedUri::try_from(uri.as_str()).expect("eventky tag URI should parse");

        assert!(matches!(parsed, CompatParsedUri::UniversalTag { .. }));
        assert_eq!(parsed.user_id(), &PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(parsed.app(), "eventky.app");
        assert_eq!(parsed.resource(), &Resource::Tag(tag_id.to_string()));
        assert_eq!(parsed.tag_id(), Some(tag_id));
    }

    #[test]
    fn test_reject_universal_non_tag_path() {
        let uri = format!("pubky://{USER_ID}/pub/eventky.app/posts/123");
        assert!(CompatParsedUri::try_from(uri.as_str()).is_err());
    }

    #[test]
    fn test_reject_non_pubky_scheme() {
        assert!(CompatParsedUri::try_from("https://example.com/pub/pubky.app/").is_err());
    }

    #[test]
    fn test_reject_missing_user_id() {
        assert!(CompatParsedUri::try_from("pubky:///pub/pubky.app/").is_err());
    }

    #[test]
    fn test_uppercase_scheme() {
        let uri = format!("PUBKY://{USER_ID}/pub/mapky/tags/ABC123");
        assert!(CompatParsedUri::try_from(uri.as_str()).is_ok());
    }

    #[test]
    fn test_universal_tag_uri_with_query_string() {
        let uri = format!("pubky://{USER_ID}/pub/mapky/tags/ABC123?foo=bar");
        let parsed = CompatParsedUri::try_from(uri.as_str())
            .expect("universal tag URI with query should parse");
        assert!(matches!(parsed, CompatParsedUri::UniversalTag { .. }));
        assert_eq!(parsed.resource(), &Resource::Tag("ABC123".to_string()));
        assert_eq!(parsed.tag_id(), Some("ABC123"));
    }

    #[test]
    fn test_universal_tag_uri_with_fragment() {
        let uri = format!("pubky://{USER_ID}/pub/mapky/tags/ABC123#section");
        let parsed = CompatParsedUri::try_from(uri.as_str())
            .expect("universal tag URI with fragment should parse");
        assert!(matches!(parsed, CompatParsedUri::UniversalTag { .. }));
        assert_eq!(parsed.resource(), &Resource::Tag("ABC123".to_string()));
        assert_eq!(parsed.tag_id(), Some("ABC123"));
    }
}
