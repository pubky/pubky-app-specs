use crate::traits::{TimestampId, Validatable};
use serde::{Deserialize, Serialize};

/// Profile schema
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct PubkyAppFile {
    name: String,
    created_at: i64,
    src: String,
    content_type: String,
    size: i64,
}

impl TimestampId for PubkyAppFile {}

impl Validatable for PubkyAppFile {
    // TODO: content_type validation.
    fn validate(&self, id: &str) -> Result<(), String> {
        self.validate_id(id)?;
        // TODO: content_type validation.
        // TODO: size and other validation.
        Ok(())
    }
}
