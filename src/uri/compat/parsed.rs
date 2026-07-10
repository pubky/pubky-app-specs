use crate::PubkyId;
use serde::{Deserialize, Serialize};
use std::convert::{From, TryFrom};

use super::tag::TagPath;
use super::super::{ParsedUri, Resource};

/// Parsed URI for ingest boundaries: strict [`super::super::ParsedUri`] paths
/// plus cross-app universal tag paths.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtendedParsedUri {
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
    },
}

impl ExtendedParsedUri {
    pub fn user_id(&self) -> &PubkyId {
        match self {
            ExtendedParsedUri::PubkyApp { user_id, .. }
            | ExtendedParsedUri::UniversalTag { user_id, .. } => user_id,
        }
    }

    pub fn resource(&self) -> &Resource {
        match self {
            ExtendedParsedUri::PubkyApp { resource, .. }
            | ExtendedParsedUri::UniversalTag { resource, .. } => resource,
        }
    }

    /// Returns the app name. `"pubky.app"` for [`ExtendedParsedUri::PubkyApp`] variants.
    pub fn app(&self) -> &str {
        match self {
            ExtendedParsedUri::PubkyApp { .. } => "pubky.app",
            ExtendedParsedUri::UniversalTag { app, .. } => app.as_str(),
        }
    }

    /// Returns the tag ID when this is a [`ExtendedParsedUri::UniversalTag`] with a tag resource.
    pub fn tag_id(&self) -> Option<&str> {
        match self {
            ExtendedParsedUri::PubkyApp { .. } => None,
            ExtendedParsedUri::UniversalTag { resource, .. } => match resource {
                Resource::Tag(id) => Some(id.as_str()),
                _ => None,
            },
        }
    }

    /// Converts back to a canonical URI string.
    ///
    /// For [`ExtendedParsedUri::PubkyApp`], delegates to [`ParsedUri::try_to_uri_str`].
    /// For [`ExtendedParsedUri::UniversalTag`], reconstructs the path from components.
    /// Query strings, fragments, and scheme casing are not preserved.
    pub fn try_to_uri_str(&self) -> Result<String, String> {
        match self {
            ExtendedParsedUri::PubkyApp { user_id, resource } => ParsedUri {
                user_id: user_id.clone(),
                resource: resource.clone(),
            }
            .try_to_uri_str(),
            ExtendedParsedUri::UniversalTag { user_id, app, resource } => {
                let Resource::Tag(tag_id) = resource else {
                    return Err(format!(
                        "Universal tag URI must have Tag resource, got {resource}"
                    ));
                };
                Ok(TagPath {
                    user_id: user_id.clone(),
                    app: app.clone(),
                    tag_id: tag_id.clone(),
                }
                .to_uri_str())
            }
        }
    }
}

impl From<ParsedUri> for ExtendedParsedUri {
    fn from(parsed: ParsedUri) -> Self {
        ExtendedParsedUri::PubkyApp {
            user_id: parsed.user_id,
            resource: parsed.resource,
        }
    }
}

impl TryFrom<&str> for ExtendedParsedUri {
    type Error = String;

    fn try_from(uri: &str) -> Result<Self, Self::Error> {
        if let Ok(parsed_uri) = ParsedUri::try_from(uri) {
            return Ok(parsed_uri.into());
        }

        if let Some(parsed) = TagPath::parse(uri) {
            return Ok(ExtendedParsedUri::UniversalTag {
                user_id: parsed.user_id,
                app: parsed.app,
                resource: Resource::Tag(parsed.tag_id),
            });
        }

        Err(format!(
            "URI is not a recognized pubky.app path or universal tag path: {uri}"
        ))
    }
}

impl TryFrom<String> for ExtendedParsedUri {
    type Error = String;

    fn try_from(uri: String) -> Result<Self, Self::Error> {
        ExtendedParsedUri::try_from(uri.as_str())
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
        let parsed = ExtendedParsedUri::try_from(uri.as_str()).expect("post URI should parse");

        assert!(matches!(parsed, ExtendedParsedUri::PubkyApp { .. }));
        assert_eq!(parsed.resource(), &Resource::Post(post_id.to_string()));
        assert_eq!(parsed.app(), "pubky.app");
    }

    #[test]
    fn test_parse_standard_tag_uri() {
        let tag_id = "8Z8CWH8NVYQY39ZEBFGKQWWEKG";
        let uri = format!("pubky://{USER_ID}/pub/pubky.app/tags/{tag_id}");
        let parsed = ExtendedParsedUri::try_from(uri.as_str()).expect("tag URI should parse");

        assert!(matches!(parsed, ExtendedParsedUri::PubkyApp { .. }));
        assert_eq!(parsed.resource(), &Resource::Tag(tag_id.to_string()));
        assert_eq!(parsed.app(), "pubky.app");
        assert_eq!(parsed.tag_id(), None);
    }

    #[test]
    fn test_parse_universal_tag_uri_mapky() {
        let tag_id = "ABC123";
        let uri = format!("pubky://{USER_ID}/pub/mapky/tags/{tag_id}");
        let parsed =
            ExtendedParsedUri::try_from(uri.as_str()).expect("mapky tag URI should parse");

        assert!(matches!(parsed, ExtendedParsedUri::UniversalTag { .. }));
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
            ExtendedParsedUri::try_from(uri.as_str()).expect("eventky tag URI should parse");

        assert!(matches!(parsed, ExtendedParsedUri::UniversalTag { .. }));
        assert_eq!(parsed.user_id(), &PubkyId::try_from(USER_ID).unwrap());
        assert_eq!(parsed.app(), "eventky.app");
        assert_eq!(parsed.resource(), &Resource::Tag(tag_id.to_string()));
        assert_eq!(parsed.tag_id(), Some(tag_id));
    }

    #[test]
    fn test_reject_universal_non_tag_path() {
        let uri = format!("pubky://{USER_ID}/pub/eventky.app/posts/123");
        assert!(ExtendedParsedUri::try_from(uri.as_str()).is_err());
    }

    #[test]
    fn test_reject_non_pubky_scheme() {
        assert!(ExtendedParsedUri::try_from("https://example.com/pub/pubky.app/").is_err());
    }

    #[test]
    fn test_reject_missing_user_id() {
        assert!(ExtendedParsedUri::try_from("pubky:///pub/pubky.app/").is_err());
    }

    #[test]
    fn test_uppercase_scheme() {
        let uri = format!("PUBKY://{USER_ID}/pub/mapky/tags/ABC123");
        assert!(ExtendedParsedUri::try_from(uri.as_str()).is_ok());
    }

    #[test]
    fn test_universal_tag_uri_with_query_string() {
        let uri = format!("pubky://{USER_ID}/pub/mapky/tags/ABC123?foo=bar");
        let parsed = ExtendedParsedUri::try_from(uri.as_str())
            .expect("universal tag URI with query should parse");
        assert!(matches!(parsed, ExtendedParsedUri::UniversalTag { .. }));
        assert_eq!(parsed.resource(), &Resource::Tag("ABC123".to_string()));
        assert_eq!(parsed.tag_id(), Some("ABC123"));
    }

    #[test]
    fn test_universal_tag_uri_with_fragment() {
        let uri = format!("pubky://{USER_ID}/pub/mapky/tags/ABC123#section");
        let parsed = ExtendedParsedUri::try_from(uri.as_str())
            .expect("universal tag URI with fragment should parse");
        assert!(matches!(parsed, ExtendedParsedUri::UniversalTag { .. }));
        assert_eq!(parsed.resource(), &Resource::Tag("ABC123".to_string()));
        assert_eq!(parsed.tag_id(), Some("ABC123"));
    }

    #[test]
    fn test_pubky_app_uri_roundtrip() {
        let uri = format!("pubky://{USER_ID}/pub/pubky.app/posts/0032SSN7Q4EVG");
        let parsed = ExtendedParsedUri::try_from(uri.as_str()).expect("post URI should parse");
        assert_eq!(
            parsed.try_to_uri_str().expect("roundtrip should succeed"),
            uri
        );
    }

    #[test]
    fn test_universal_tag_uri_roundtrip() {
        let uri = format!("pubky://{USER_ID}/pub/mapky/tags/ABC123");
        let parsed = ExtendedParsedUri::try_from(uri.as_str()).expect("mapky tag URI should parse");
        assert_eq!(
            parsed.try_to_uri_str().expect("roundtrip should succeed"),
            uri
        );
    }

    #[test]
    fn test_universal_tag_uri_roundtrip_normalizes_input() {
        let input = format!("PUBKY://{USER_ID}/pub/mapky/tags/ABC123?foo=bar#section");
        let expected = format!("pubky://{USER_ID}/pub/mapky/tags/ABC123");
        let parsed = ExtendedParsedUri::try_from(input.as_str()).expect("URI should parse");
        assert_eq!(
            parsed.try_to_uri_str().expect("roundtrip should succeed"),
            expected
        );
    }
}
