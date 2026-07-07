use crate::{PubkyId, APP_PATH};

use super::super::path::try_parse_pubky_path;

/// Parsed cross-app tag path: `pubky://<user_id>/pub/<app>/tags/<tag_id>`
pub(crate) struct TagPath {
    pub user_id: PubkyId,
    pub app: String,
    pub tag_id: String,
    #[allow(dead_code)]
    pub uri: String,
}

impl TagPath {
    /// Parse a non-`pubky.app` URI when it follows the universal tag shape.
    pub fn parse(uri: &str) -> Option<Self> {
        let path = try_parse_pubky_path(uri).ok()?;

        if path.app.is_empty() || path.app == APP_PATH.trim_end_matches('/') {
            return None;
        }

        let [resource, tag_id] = path.segments.as_slice() else {
            return None;
        };

        if resource != "tags" || tag_id.is_empty() {
            return None;
        }

        Some(Self {
            user_id: path.user_id,
            app: path.app,
            tag_id: tag_id.clone(),
            uri: uri.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE_URI: &str =
        "pubky://8pinxxgqs41n4aididenw5apqp1urfmzdztr8jt4abrkdn435ewo/pub/mapky/tags/ABC123";

    #[test]
    fn test_parse_mapky() {
        let parsed = TagPath::parse(BASE_URI).expect("mapky tag URI should parse");
        assert_eq!(parsed.app, "mapky");
        assert_eq!(parsed.tag_id, "ABC123");
        assert_eq!(parsed.uri, BASE_URI);
    }

    #[test]
    fn test_parse_eventky() {
        let uri = "pubky://8pinxxgqs41n4aididenw5apqp1urfmzdztr8jt4abrkdn435ewo/pub/eventky.app/tags/XYZ";
        let parsed = TagPath::parse(uri).expect("eventky tag URI should parse");
        assert_eq!(parsed.app, "eventky.app");
        assert_eq!(parsed.tag_id, "XYZ");
    }

    #[test]
    fn test_parse_pubky_app_returns_none() {
        let uri = "pubky://8pinxxgqs41n4aididenw5apqp1urfmzdztr8jt4abrkdn435ewo/pub/pubky.app/tags/123";
        assert!(TagPath::parse(uri).is_none());
    }

    #[test]
    fn test_parse_not_pubky() {
        assert!(TagPath::parse("https://example.com/tags/123").is_none());
    }

    #[test]
    fn test_parse_no_tags_segment() {
        let uri = "pubky://8pinxxgqs41n4aididenw5apqp1urfmzdztr8jt4abrkdn435ewo/pub/mapky/events/123";
        assert!(TagPath::parse(uri).is_none());
    }

    #[test]
    fn test_parse_uppercase_scheme() {
        let uri = "PUBKY://8pinxxgqs41n4aididenw5apqp1urfmzdztr8jt4abrkdn435ewo/pub/mapky/tags/ABC123";
        assert!(TagPath::parse(uri).is_some());
    }

    #[test]
    fn test_parse_mixed_case_scheme() {
        let uri = "Pubky://8pinxxgqs41n4aididenw5apqp1urfmzdztr8jt4abrkdn435ewo/pub/mapky/tags/XYZ";
        assert!(TagPath::parse(uri).is_some());
    }

    #[test]
    fn test_parse_slash_in_app_returns_none() {
        let uri = "pubky://8pinxxgqs41n4aididenw5apqp1urfmzdztr8jt4abrkdn435ewo/pub/my/app/tags/ABC";
        assert!(TagPath::parse(uri).is_none());
    }

    #[test]
    fn test_parse_slash_in_tag_returns_none() {
        let uri = "pubky://8pinxxgqs41n4aididenw5apqp1urfmzdztr8jt4abrkdn435ewo/pub/mapky/tags/ABC/DEF";
        assert!(TagPath::parse(uri).is_none());
    }

    #[test]
    fn test_parse_empty_app_returns_none() {
        let uri = "pubky://8pinxxgqs41n4aididenw5apqp1urfmzdztr8jt4abrkdn435ewo/pub//tags/ABC";
        assert!(TagPath::parse(uri).is_none());
    }

    #[test]
    fn test_parse_empty_tag_returns_none() {
        let uri = "pubky://8pinxxgqs41n4aididenw5apqp1urfmzdztr8jt4abrkdn435ewo/pub/mapky/tags/";
        assert!(TagPath::parse(uri).is_none());
    }

    #[test]
    fn test_parse_query_string_stripped() {
        let uri = "pubky://8pinxxgqs41n4aididenw5apqp1urfmzdztr8jt4abrkdn435ewo/pub/mapky/tags/ABC123?foo=bar";
        let parsed = TagPath::parse(uri).expect("URI with query string should parse");
        assert_eq!(parsed.tag_id, "ABC123");
    }

    #[test]
    fn test_parse_fragment_stripped() {
        let uri = "pubky://8pinxxgqs41n4aididenw5apqp1urfmzdztr8jt4abrkdn435ewo/pub/mapky/tags/ABC123#section";
        let parsed = TagPath::parse(uri).expect("URI with fragment should parse");
        assert_eq!(parsed.tag_id, "ABC123");
    }

    #[test]
    fn test_parse_empty_tag_after_query_returns_none() {
        let uri = "pubky://8pinxxgqs41n4aididenw5apqp1urfmzdztr8jt4abrkdn435ewo/pub/mapky/tags/?foo=bar";
        assert!(TagPath::parse(uri).is_none());
    }
}
