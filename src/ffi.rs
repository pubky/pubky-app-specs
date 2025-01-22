use crate::traits::{HasPath, HasPubkyIdPath, HashId, TimestampId, Validatable};
use crate::*;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

/// Each FFI function:
/// - Accepts minimal fields in a JavaScript-friendly manner (e.g. strings, JSON).
/// - Creates the Rust model, sanitizes, and validates it.
/// - Generates the ID (if applicable).
/// - Generates the path (if applicable).
/// - Returns { json, id, path } or a descriptive error.

/// Returned by each FFI function to JS.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateResult {
    /// The fully validated and sanitized object, as JSON
    pub json: serde_json::Value,
    /// The unique ID for this object (empty if none)
    pub id: String,
    /// The final path (or empty if none)
    pub path: String,
}

/// Utility to convert a Rust object into `{ json, id, path }` for JS.
fn build_create_result<T>(obj: &T, id: &str, path: &str) -> Result<JsValue, JsValue>
where
    T: Serialize,
{
    // 1) Serialize `obj` into a JavaScript object
    let json_val = to_value(obj).map_err(|e| JsValue::from_str(&format!("JSON Error: {}", e)))?;

    // 2) Construct the final object { id, path, json } just like `get_data()` does
    let ret_obj = js_sys::Object::new();
    js_sys::Reflect::set(&ret_obj, &JsValue::from_str("id"), &JsValue::from_str(id))?;
    js_sys::Reflect::set(
        &ret_obj,
        &JsValue::from_str("path"),
        &JsValue::from_str(path),
    )?;
    js_sys::Reflect::set(&ret_obj, &JsValue::from_str("json"), &json_val)?;

    Ok(ret_obj.into())
}

// -----------------------------------------------------------------------------
// 1. PubkyAppUser
// -----------------------------------------------------------------------------

#[wasm_bindgen]
pub fn create_pubky_app_user(
    name: String,
    bio: Option<String>,
    image: Option<String>,
    links: JsValue, // JSON array of PubkyAppUserLink
    status: Option<String>,
) -> Result<JsValue, JsValue> {
    // Convert links to Option<Vec<PubkyAppUserLink>>
    let links_vec: Option<Vec<PubkyAppUserLink>> = if links.is_null() || links.is_undefined() {
        None
    } else {
        from_value(links)?
    };

    // Create user, sanitize, then validate
    let user = PubkyAppUser::new(name, bio, image, links_vec, status);
    user.validate("")?; // no ID-based validation

    // We have no ID for PubkyAppUser. The path is always profile.json
    let path = user.create_path();

    build_create_result(&user, "", &path)
}

// -----------------------------------------------------------------------------
// 2. PubkyAppFeed
// -----------------------------------------------------------------------------

#[wasm_bindgen]
pub fn create_pubky_app_feed(
    tags: JsValue, // JSON array of strings
    reach: String,
    layout: String,
    sort: String,
    content: Option<String>,
    name: String,
) -> Result<JsValue, JsValue> {
    // Convert tags
    let tags_vec: Option<Vec<String>> = if tags.is_null() || tags.is_undefined() {
        None
    } else {
        from_value(tags)?
    };

    // Convert feed reach
    let reach_enum = match reach.as_str() {
        "following" => PubkyAppFeedReach::Following,
        "followers" => PubkyAppFeedReach::Followers,
        "friends" => PubkyAppFeedReach::Friends,
        "all" => PubkyAppFeedReach::All,
        _ => return Err(JsValue::from_str("Invalid feed reach")),
    };

    // Convert layout
    let layout_enum = match layout.as_str() {
        "columns" => PubkyAppFeedLayout::Columns,
        "wide" => PubkyAppFeedLayout::Wide,
        "visual" => PubkyAppFeedLayout::Visual,
        _ => return Err(JsValue::from_str("Invalid feed layout")),
    };

    // Convert sort
    let sort_enum = match sort.as_str() {
        "recent" => PubkyAppFeedSort::Recent,
        "popularity" => PubkyAppFeedSort::Popularity,
        _ => return Err(JsValue::from_str("Invalid feed sort")),
    };

    // Convert content kind
    let content_kind = match content.as_deref() {
        Some("short") => Some(PubkyAppPostKind::Short),
        Some("long") => Some(PubkyAppPostKind::Long),
        Some("image") => Some(PubkyAppPostKind::Image),
        Some("video") => Some(PubkyAppPostKind::Video),
        Some("link") => Some(PubkyAppPostKind::Link),
        Some("file") => Some(PubkyAppPostKind::File),
        None => None,
        Some(_) => return Err(JsValue::from_str("Invalid content kind")),
    };

    // Build feed, sanitize, validate
    let feed = PubkyAppFeed::new(
        tags_vec,
        reach_enum,
        layout_enum,
        sort_enum,
        content_kind,
        name,
    );
    let feed_id = feed.create_id();
    feed.validate(&feed_id)?;

    let path = feed.create_path();
    build_create_result(&feed, &feed_id, &path)
}

// -----------------------------------------------------------------------------
// 3. PubkyAppFile
// -----------------------------------------------------------------------------

#[wasm_bindgen]
pub fn create_pubky_app_file(
    name: String,
    src: String,
    content_type: String,
    size: i64,
) -> Result<JsValue, JsValue> {
    let file = PubkyAppFile::new(name, src, content_type, size);
    let file_id = file.create_id();
    file.validate(&file_id)?;

    let path = file.create_path();
    build_create_result(&file, &file_id, &path)
}

// -----------------------------------------------------------------------------
// 4. PubkyAppPost
// -----------------------------------------------------------------------------

#[wasm_bindgen]
pub fn create_pubky_app_post(
    content: String,
    kind: String,
    parent: Option<String>,
    embed: JsValue,       // JSON object { kind: string, uri: string } or null
    attachments: JsValue, // JSON array of string or null
) -> Result<JsValue, JsValue> {
    // Convert kind
    let kind_enum = match kind.as_str() {
        "short" => PubkyAppPostKind::Short,
        "long" => PubkyAppPostKind::Long,
        "image" => PubkyAppPostKind::Image,
        "video" => PubkyAppPostKind::Video,
        "link" => PubkyAppPostKind::Link,
        "file" => PubkyAppPostKind::File,
        _ => return Err(JsValue::from_str("Invalid post kind")),
    };

    // Convert embed
    let embed_option: Option<PubkyAppPostEmbed> = if embed.is_null() || embed.is_undefined() {
        None
    } else {
        from_value(embed)?
    };

    // Convert attachments
    let attachments_vec: Option<Vec<String>> =
        if attachments.is_null() || attachments.is_undefined() {
            None
        } else {
            from_value(attachments)?
        };

    // Build the post, sanitize, validate
    let post = PubkyAppPost::new(content, kind_enum, parent, embed_option, attachments_vec);
    let post_id = post.create_id();
    post.validate(&post_id)?;

    let path = post.create_path();
    build_create_result(&post, &post_id, &path)
}

// -----------------------------------------------------------------------------
// 5. PubkyAppTag
// -----------------------------------------------------------------------------

#[wasm_bindgen]
pub fn create_pubky_app_tag(uri: String, label: String) -> Result<JsValue, JsValue> {
    let tag = PubkyAppTag::new(uri, label);
    let tag_id = tag.create_id();
    tag.validate(&tag_id)?;

    let path = tag.create_path();
    build_create_result(&tag, &tag_id, &path)
}

// -----------------------------------------------------------------------------
// 6. PubkyAppBookmark
// -----------------------------------------------------------------------------

#[wasm_bindgen]
pub fn create_pubky_app_bookmark(uri: String) -> Result<JsValue, JsValue> {
    let bookmark = PubkyAppBookmark::new(uri);
    let bookmark_id = bookmark.create_id();
    bookmark.validate(&bookmark_id)?;

    let path = bookmark.create_path();
    build_create_result(&bookmark, &bookmark_id, &path)
}

// -----------------------------------------------------------------------------
// 7. PubkyAppFollow
// -----------------------------------------------------------------------------

#[wasm_bindgen]
pub fn create_pubky_app_follow(pubky_id: String) -> Result<JsValue, JsValue> {
    let follow = PubkyAppFollow::new();
    follow.validate(&pubky_id)?; // No ID in follow, so we pass user ID or empty

    // Path requires the user ID
    let path = follow.create_path(&pubky_id);

    // Return an empty ID for follow
    build_create_result(&follow, "", &path)
}

// -----------------------------------------------------------------------------
// 8. PubkyAppMute
// -----------------------------------------------------------------------------

#[wasm_bindgen]
pub fn create_pubky_app_mute(pubky_id: String) -> Result<JsValue, JsValue> {
    let mute = PubkyAppMute::new();
    mute.validate(&pubky_id)?;

    let path = mute.create_path(&pubky_id);
    build_create_result(&mute, "", &path)
}

// -----------------------------------------------------------------------------
// 9. PubkyAppLastRead
// -----------------------------------------------------------------------------

#[wasm_bindgen]
pub fn create_pubky_app_last_read() -> Result<JsValue, JsValue> {
    let last_read = PubkyAppLastRead::new();
    last_read.validate("")?;

    let path = last_read.create_path();
    build_create_result(&last_read, "", &path)
}

// -----------------------------------------------------------------------------
// 10. PubkyAppBlob
// -----------------------------------------------------------------------------

/// A small wrapper for JSON-serializing the blob data as base64.
#[derive(Serialize, Deserialize, Clone)]
pub struct PubkyAppBlobJson {
    pub data_base64: String,
}

// #[wasm_bindgen]
// pub fn create_pubky_app_blob(blob_data: JsValue) -> Result<JsValue, JsValue> {
//     // 1) Convert from JsValue (Uint8Array in JS) -> Vec<u8> in Rust
//     let data_vec: Vec<u8> = from_value(blob_data)
//         .map_err(|e| JsValue::from_str(&format!("Invalid blob bytes: {}", e)))?;

//     // 2) Build the PubkyAppBlob
//     let blob = PubkyAppBlob(data_vec);

//     // 3) Generate ID and path
//     let id = blob.create_id();
//     let path = blob.create_path();

//     // 4) Provide a minimal JSON representation (e.g. base64)
//     let json_blob = PubkyAppBlobJson {
//         data_base64: base64::encode(&blob.0),
//     };

//     // 5) Return { json, id, path }
//     build_create_result(&json_blob, &id, &path)
// }
