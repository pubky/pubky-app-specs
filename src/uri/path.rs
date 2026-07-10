use crate::{PubkyId, PROTOCOL, PUBLIC_PATH};
use url::Url;

use super::scheme::is_pubky_scheme;

/// Generic parsed structure for `pubky://<user_id>/pub/<app>/...`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubkyPath {
    pub user_id: PubkyId,
    pub app: String,
    /// Path segments after `<app>`.
    pub segments: Vec<String>,
}

/// Parse the structural components of a pubky URI without app-specific rules.
pub fn try_parse_pubky_path(uri: &str) -> Result<PubkyPath, String> {
    let parsed_url = Url::parse(uri).map_err(|e| format!("Invalid URL: {e}"))?;

    if !is_pubky_scheme(parsed_url.scheme()) {
        return Err(format!(
            "Invalid URI, must start with '{PROTOCOL}': {uri}"
        ));
    }

    let user_id_str = parsed_url
        .host_str()
        .ok_or_else(|| format!("Missing user ID in URI: {uri}"))?;
    let user_id = PubkyId::try_from(user_id_str)?;

    let segments: Vec<&str> = parsed_url
        .path_segments()
        .ok_or_else(|| format!("Cannot parse path segments from URI: {uri}"))?
        .collect();

    if segments.len() < 2 {
        return Err(format!("Not enough path segments in URI: {uri}"));
    }
    if segments[0] != PUBLIC_PATH.trim_matches('/') {
        return Err(format!(
            "Expected public path '{}' but got '{}' in URI: {}",
            PUBLIC_PATH.trim_matches('/'),
            segments[0],
            uri
        ));
    }

    Ok(PubkyPath {
        user_id,
        app: segments[1].to_string(),
        segments: segments[2..].iter().map(|segment| segment.to_string()).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const USER_ID: &str = "8pinxxgqs41n4aididenw5apqp1urfmzdztr8jt4abrkdn435ewo";

    #[test]
    fn test_parse_pubky_app_post_path() {
        let uri = format!("pubky://{USER_ID}/pub/pubky.app/posts/0032SSN7Q4EVG");
        let path = try_parse_pubky_path(&uri).expect("post path should parse");
        assert_eq!(path.app, "pubky.app");
        assert_eq!(path.segments, vec!["posts", "0032SSN7Q4EVG"]);
    }

    #[test]
    fn test_parse_foreign_app_tag_path() {
        let uri = format!("pubky://{USER_ID}/pub/mapky/tags/ABC123");
        let path = try_parse_pubky_path(&uri).expect("foreign tag path should parse");
        assert_eq!(path.app, "mapky");
        assert_eq!(path.segments, vec!["tags", "ABC123"]);
    }

    #[test]
    fn test_reject_non_pubky_scheme() {
        assert!(try_parse_pubky_path("https://example.com/pub/pubky.app/").is_err());
    }

    #[test]
    fn test_reject_missing_user_id() {
        assert!(try_parse_pubky_path("pubky:///pub/pubky.app/").is_err());
    }
}
