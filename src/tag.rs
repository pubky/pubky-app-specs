use crate::traits::{HashId, Validatable};
use crate::types::DynError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

// Validation
const MAX_TAG_LABEL_LENGTH: usize = 20;

/// Represents raw homeserver tag with id
/// URI: /pub/pubky.app/tags/:tag_id
///
/// Example URI:
///
/// `/pub/pubky.app/tags/FPB0AM9S93Q3M1GFY1KV09GMQM`
///
/// Where tag_id is Crockford-base32(Blake3("{uri_tagged}:{label}")[:half])
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PubkyAppTag {
    pub uri: String,
    pub label: String,
    pub created_at: i64,
}

#[async_trait]
impl HashId for PubkyAppTag {
    /// Tag ID is created based on the hash of the URI tagged and the label used
    fn get_id_data(&self) -> String {
        format!("{}:{}", self.uri, self.label)
    }
}

#[async_trait]
impl Validatable for PubkyAppTag {
    async fn sanitize(self) -> Result<Self, DynError> {
        // Remove spaces from the tag and keep it as one word
        let mut label = self
            .label
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();
        // Convert label to lowercase and trim
        label = label.trim().to_lowercase();

        // Enforce maximum label length safely
        label = label.chars().take(MAX_TAG_LABEL_LENGTH).collect::<String>();

        // Sanitize URI
        let uri = match Url::parse(&self.uri) {
            Ok(url) => url.to_string(),
            Err(_) => return Err("Invalid URI in tag".into()),
        };

        Ok(PubkyAppTag {
            uri,
            label,
            created_at: self.created_at,
        })
    }

    async fn validate(&self, id: &str) -> Result<(), DynError> {
        self.validate_id(id).await?;

        // Validate label length based on characters
        if self.label.chars().count() > MAX_TAG_LABEL_LENGTH {
            return Err("Tag label exceeds maximum length".into());
        }

        // TODO: more validation?

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[test]
    fn test_label_id() {
        // Precomputed earlier
        let tag_id = "CBYS8P6VJPHC5XXT4WDW26662W";
        // Create new tag
        let tag = PubkyAppTag {
            uri: "pubky://user_id/pub/pubky.app/posts/post_id".to_string(),
            created_at: 1627849723,
            label: "cool".to_string(),
        };
        // Check if the tag ID is correct
        assert_eq!(
            tag.create_id(),
            tag_id
        );

        let wrong_tag = PubkyAppTag {
            uri: "pubky://user_id/pub/pubky.app/posts/post_id".to_string(),
            created_at: 1627849723,
            label: "co0l".to_string(),
        };

        // Assure that the new tag has wrong ID
        assert_ne!(
            wrong_tag.create_id(),
            tag_id
        );
    }

    #[tokio::test]
    async fn test_incorrect_label() -> Result<(), DynError> {
        let tag = PubkyAppTag {
            uri: "user_id/pub/pubky.app/posts/post_id".to_string(),
            created_at: 1627849723,
            label: "cool".to_string(),
        };

        match tag.sanitize().await {
            Err(e) => assert_eq!(e.to_string(), "Invalid URI in tag".to_string(), "The error message is not related URI or the message description is wrong"),
            _ => ()
        };

        let tag = PubkyAppTag {
            uri: "pubky://user_id/pub/pubky.app/posts/post_id".to_string(),
            created_at: 1627849723,
            label: "coolc00lcolaca0g00llooll".to_string(),
        };

        // Precomputed earlier
        let label_id = "8WXXXXHK028RH8AWBZZNHJYDN4";

        match tag.validate(label_id).await {
            Err(e) => assert_eq!(e.to_string(), "Tag label exceeds maximum length".to_string(), "The error message is not related tag length or the message description is wrong"),
            _ => ()
        };

        Ok(())


    }

    #[tokio::test]
    async fn test_white_space_tag() -> Result<(), DynError> {
        // All the tags has to be that label after sanitation
        let label = "cool";

        let leading_whitespace = PubkyAppTag {
            uri: "pubky://user_id/pub/pubky.app/posts/post_id".to_string(),
            created_at: 1627849723,
            label: " cool".to_string(),
        };
        let mut sanitazed_label = leading_whitespace.sanitize().await?;
        assert_eq!(sanitazed_label.label, label);

        let trailing_whitespace = PubkyAppTag {
            uri: "pubky://user_id/pub/pubky.app/posts/post_id".to_string(),
            created_at: 1627849723,
            label: " cool".to_string(),
        };
        sanitazed_label = trailing_whitespace.sanitize().await?;
        assert_eq!(sanitazed_label.label, label);

        let space_between = PubkyAppTag {
            uri: "pubky://user_id/pub/pubky.app/posts/post_id".to_string(),
            created_at: 1627849723,
            label: "co ol".to_string(),
        };
        sanitazed_label = space_between.sanitize().await?;
        assert_eq!(sanitazed_label.label, label);

        let space_between = PubkyAppTag {
            uri: "pubky://user_id/pub/pubky.app/posts/post_id".to_string(),
            created_at: 1627849723,
            label: "   co ol ".to_string(),
        };
        sanitazed_label = space_between.sanitize().await?;
        assert_eq!(sanitazed_label.label, label);

        Ok(())
    }
}
