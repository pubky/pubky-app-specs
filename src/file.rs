use crate::traits::{TimestampId, Validatable};
use crate::types::DynError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Profile schema
#[derive(Deserialize, Serialize, Debug)]
pub struct PubkyAppFile {
    name: String,
    created_at: i64,
    src: String,
    content_type: String,
    size: i64,
}

impl TimestampId for PubkyAppFile {}

#[async_trait]
impl Validatable for PubkyAppFile {
    // TODO: content_type validation.
    async fn validate(&self, id: &str) -> Result<(), DynError> {
        self.validate_id(id).await?;
        // TODO: content_type validation.
        // TODO: size and other validation.
        Ok(())
    }
}
