//! Validation limits for pubky-app-specs data models.
//!
//! These constants are the single source of truth for client-side validation.
//!
//! # Examples
//! Serialize the bundled limits for client consumption.
//! ```
//! use pubky_app_specs::VALIDATION_LIMITS;
//!
//! let limits_json = serde_json::to_value(&VALIDATION_LIMITS).unwrap();
//! assert!(limits_json.is_object());
//! ```

use serde::Serialize;

/// Bundled validation limits for quick consumption.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationLimits {
    /// Maximum blob/file size in bytes.
    ///
    /// Chosen to cap user storage per upload and align with homeserver limits.
    pub max_blob_size_bytes: usize,
    /// Maximum file size in bytes.
    ///
    /// Kept in sync with blob validation since files are blob-backed.
    pub max_file_size_bytes: usize,
    /// Minimum number of characters for tag labels.
    pub tag_label_min_length: usize,
    /// Maximum number of characters for tag labels.
    pub tag_label_max_length: usize,
    /// Disallowed characters, including common whitespace.
    pub tag_invalid_chars: &'static [char],
    /// Minimum username length in characters.
    pub user_name_min_length: usize,
    /// Maximum username length in characters.
    pub user_name_max_length: usize,
    /// Maximum bio length in characters.
    pub user_bio_max_length: usize,
    /// Maximum image URL length in characters.
    pub user_image_url_max_length: usize,
    /// Maximum number of profile links.
    pub user_links_max_count: usize,
    /// Maximum link title length in characters.
    pub user_link_title_max_length: usize,
    /// Maximum link URL length in characters.
    pub user_link_url_max_length: usize,
    /// Maximum status length in characters.
    pub user_status_max_length: usize,
    /// Maximum character count for short posts.
    pub post_short_content_max_length: usize,
    /// Maximum character count for long posts.
    pub post_long_content_max_length: usize,
    /// Maximum number of attachments per post.
    pub post_attachments_max_count: usize,
    /// Maximum length for attachment URLs.
    pub post_attachment_url_max_length: usize,
    /// Allowed protocols for attachment URLs.
    pub post_allowed_attachment_protocols: &'static [&'static str],
    /// Minimum file name length in characters.
    pub file_name_min_length: usize,
    /// Maximum file name length in characters.
    pub file_name_max_length: usize,
    /// Maximum file src length in characters.
    pub file_src_max_length: usize,
    /// Maximum number of tags allowed in a feed.
    pub feed_tags_max_count: usize,
}

/// All validation limits in a single bundle.
pub const VALIDATION_LIMITS: ValidationLimits = ValidationLimits {
    max_blob_size_bytes: 100 * (1 << 20), // 100 MB cap aligned with homeserver limits.
    max_file_size_bytes: 100 * (1 << 20), // Kept in sync with blob validation.
    tag_label_min_length: 1,
    tag_label_max_length: 20,
    tag_invalid_chars: &[',', ':', ' ', '\t', '\n', '\r'],
    user_name_min_length: 3,
    user_name_max_length: 50,
    user_bio_max_length: 160,
    user_image_url_max_length: 300,
    user_links_max_count: 5,
    user_link_title_max_length: 100,
    user_link_url_max_length: 300,
    user_status_max_length: 50,
    post_short_content_max_length: 2000,
    post_long_content_max_length: 50_000,
    post_attachments_max_count: 4,
    post_attachment_url_max_length: 200,
    post_allowed_attachment_protocols: &["pubky", "http", "https"],
    file_name_min_length: 1,
    file_name_max_length: 255,
    file_src_max_length: 1024,
    feed_tags_max_count: 5,
};
