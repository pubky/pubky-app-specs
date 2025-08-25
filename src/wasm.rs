use crate::traits::{HasIdPath, HasPath, HashId, TimestampId, Validatable};
use crate::*;
use serde_wasm_bindgen::from_value;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

/// Each FFI function:
/// - Accepts minimal fields in a JavaScript-friendly manner (e.g. strings, JSON).
/// - Creates the Rust model, sanitizes, and validates it.
/// - Generates the ID (if applicable).
/// - Generates the path (if applicable).
/// - Returns { json, id, path, url } or a descriptive error.

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Meta {
    /// The unique ID for this object (empty if none)
    id: String,
    /// The final path (or empty if none)
    path: String,
    /// The final url (or empty if none)
    url: String,
}

// Implement wasm_bindgen methods to expose read-only fields.
#[wasm_bindgen]
impl Meta {
    // Getters clone the data out because String is not Copy.
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn path(&self) -> String {
        self.path.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn url(&self) -> String {
        self.url.clone()
    }
}

impl Meta {
    /// Internal helper. Generates meta's `id`, `path`, and `url`.
    pub fn from_object(object_id: Option<&str>, pubky_id: PubkyId, path: String) -> Self {
        let id = match object_id {
            Some(id) => id.to_string(),
            None => "".to_string(),
        };

        Self {
            id,
            url: format!("{}{}{}", PROTOCOL, pubky_id, path),
            path,
        }
    }
}

/// Represents a user's single link with a title and URL.
#[wasm_bindgen]
pub struct PubkySpecsBuilder {
    #[wasm_bindgen(skip)]
    pubky_id: PubkyId,
}

/// A macro to generate result structs and `wasm_bindgen`-exposed getters.
/// A struct for each `create_*()` function is needed if we want
/// correct TS types
///
/// This macro creates a struct with the specified name (`$struct_name`),
/// containing:
/// - A primary field (`$field_name`) of type `$field_type`.
/// - A `meta` field of type `Meta`.
///
/// It also generates getters for both fields.
///
/// # Usage
/// ```
/// # use pubky_app_specs::*;
/// # use wasm_bindgen::prelude::*;
/// result_struct!(PostResult, post, PubkyAppPost);
/// ```
/// Expands to:
/// ```
/// # use pubky_app_specs::*;
/// # use wasm_bindgen::prelude::*;
/// #[wasm_bindgen]
/// pub struct PostResult {
///     post: PubkyAppPost,
///     meta: Meta,
/// }
///
/// #[wasm_bindgen]
/// impl PostResult {
///     #[wasm_bindgen(getter)]
///     pub fn post(&self) -> PubkyAppPost { self.post.clone() }
///
///     #[wasm_bindgen(getter)]
///     pub fn meta(&self) -> Meta { self.meta.clone() }
/// }
/// ```
macro_rules! result_struct {
    ($struct_name:ident, $field_name:ident, $field_type:ty) => {
        #[wasm_bindgen]
        pub struct $struct_name {
            $field_name: $field_type,
            meta: Meta,
        }

        #[wasm_bindgen]
        impl $struct_name {
            #[wasm_bindgen(getter)]
            pub fn $field_name(&self) -> $field_type {
                self.$field_name.clone()
            }

            #[wasm_bindgen(getter)]
            pub fn meta(&self) -> Meta {
                self.meta.clone()
            }
        }
    };
}

result_struct!(UserResult, user, PubkyAppUser);
result_struct!(FileResult, file, PubkyAppFile);
result_struct!(FollowResult, follow, PubkyAppFollow);
result_struct!(PostResult, post, PubkyAppPost);
result_struct!(FeedResult, feed, PubkyAppFeed);
result_struct!(TagResult, tag, PubkyAppTag);
result_struct!(BookmarkResult, bookmark, PubkyAppBookmark);
result_struct!(MuteResult, mute, PubkyAppMute);
result_struct!(LastReadResult, last_read, PubkyAppLastRead);
result_struct!(BlobResult, blob, PubkyAppBlob);

#[wasm_bindgen]
impl PubkySpecsBuilder {
    /// Creates a new `PubkyAppBuilder` instance.
    #[wasm_bindgen(constructor)]
    pub fn new(pubky_id: String) -> Result<Self, String> {
        let pubky_id = PubkyId::try_from(&pubky_id)?;
        Ok(Self { pubky_id })
    }

    // // -----------------------------------------------------------------------------
    // // 1. PubkyAppUser
    // // -----------------------------------------------------------------------------

    #[wasm_bindgen(js_name = createUser)]
    pub fn create_user(
        &self,
        name: String,
        bio: Option<String>,
        image: Option<String>,
        links: JsValue, // a JS array of {title, url} or null
        status: Option<String>,
    ) -> Result<UserResult, String> {
        // 1) Convert JS 'links' -> Option<Vec<PubkyAppUserLink>>
        let links_vec: Option<Vec<PubkyAppUserLink>> = if links.is_null() || links.is_undefined() {
            None
        } else {
            from_value(links).map_err(|e| e.to_string())?
        };

        // 2) Build user domain object
        let user = PubkyAppUser::new(name, bio, image, links_vec, status);
        user.validate(None)?; // No ID-based validation for user

        // 3) Create the path and meta
        let path = user.create_path();
        let meta = Meta::from_object(None, self.pubky_id.clone(), path);

        // 4) Return a typed struct containing both
        Ok(UserResult { user, meta })
    }

    // -----------------------------------------------------------------------------
    // 2. PubkyAppFeed
    // -----------------------------------------------------------------------------

    #[wasm_bindgen(js_name = createFeed)]
    pub fn create_feed(
        &self,
        tags: JsValue,
        reach: String,
        layout: String,
        sort: String,
        content: Option<String>,
        name: String,
    ) -> Result<FeedResult, String> {
        let tags_vec: Option<Vec<String>> = if tags.is_null() || tags.is_undefined() {
            None
        } else {
            from_value(tags).map_err(|e| e.to_string())?
        };

        // Use `FromStr` to parse enums
        let reach = PubkyAppFeedReach::from_str(&reach)?;
        let layout = PubkyAppFeedLayout::from_str(&layout)?;
        let sort = PubkyAppFeedSort::from_str(&sort)?;
        let content = match content {
            Some(val) => Some(PubkyAppPostKind::from_str(&val)?),
            None => None,
        };

        // Create the feed
        let feed = PubkyAppFeed::new(tags_vec, reach, layout, sort, content, name);

        let feed_id = feed.create_id();
        feed.validate(Some(&feed_id))?;

        let path = feed.create_path(&feed_id);
        let meta = Meta::from_object(Some(&feed_id), self.pubky_id.clone(), path);

        Ok(FeedResult { feed, meta })
    }

    // -----------------------------------------------------------------------------
    // 3. PubkyAppFile
    // -----------------------------------------------------------------------------

    #[wasm_bindgen(js_name = createFile)]
    pub fn create_file(
        &self,
        name: String,
        src: String,
        content_type: String,
        size: usize,
    ) -> Result<FileResult, String> {
        let file = PubkyAppFile::new(name, src, content_type, size);
        let file_id = file.create_id();
        file.validate(Some(&file_id))?;

        let path = file.create_path(&file_id);
        let meta = Meta::from_object(Some(&file_id), self.pubky_id.clone(), path);

        Ok(FileResult { file, meta })
    }

    // -----------------------------------------------------------------------------
    // 4. PubkyAppPost
    // -----------------------------------------------------------------------------

    #[wasm_bindgen(js_name = createPost)]
    pub fn create_post(
        &self,
        content: String,
        kind: PubkyAppPostKind,
        parent: Option<String>,
        embed: Option<PubkyAppPostEmbed>,
        attachments: Option<Vec<String>>,
    ) -> Result<PostResult, String> {
        let post = PubkyAppPost::new(content, kind, parent, embed, attachments);
        let post_id = post.create_id();
        post.validate(Some(&post_id))?;

        let path = post.create_path(&post_id);
        let meta = Meta::from_object(Some(&post_id), self.pubky_id.clone(), path);

        Ok(PostResult { post, meta })
    }

    /// Edits an existing post by updating its content while preserving its original ID and timestamp.
    #[wasm_bindgen(js_name = editPost)]
    pub fn edit_post(
        &self,
        original_post: PubkyAppPost,
        post_id: String,
        new_content: String,
    ) -> Result<PostResult, String> {
        // Make a mutable copy so we can change its content.
        let mut post = original_post;
        post.content = new_content;

        // Re-sanitize the post (this should preserve the original created_at timestamp).
        post = post.sanitize();
        post.validate(Some(&post_id))?;

        // Recreate the path and meta using the unchanged ID.
        let path = [PUBLIC_PATH, APP_PATH, PubkyAppPost::PATH_SEGMENT, &post_id].concat();
        let meta = Meta::from_object(Some(&post_id), self.pubky_id.clone(), path);

        Ok(PostResult { post, meta })
    }

    // -----------------------------------------------------------------------------
    // 5. PubkyAppTag
    // -----------------------------------------------------------------------------

    #[wasm_bindgen(js_name = createTag)]
    pub fn create_tag(&self, uri: String, label: String) -> Result<TagResult, String> {
        let tag = PubkyAppTag::new(uri, label);
        let tag_id = tag.create_id();
        tag.validate(Some(&tag_id))?;

        let path = tag.create_path(&tag_id);
        let meta = Meta::from_object(Some(&tag_id), self.pubky_id.clone(), path);

        Ok(TagResult { tag, meta })
    }

    // -----------------------------------------------------------------------------
    // 6. PubkyAppBookmark
    // -----------------------------------------------------------------------------

    #[wasm_bindgen(js_name = createBookmark)]
    pub fn create_bookmark(&self, uri: String) -> Result<BookmarkResult, String> {
        let bookmark = PubkyAppBookmark::new(uri);
        let bookmark_id = bookmark.create_id();
        bookmark.validate(Some(&bookmark_id))?;

        let path = bookmark.create_path(&bookmark_id);
        let meta = Meta::from_object(Some(&bookmark_id), self.pubky_id.clone(), path);

        Ok(BookmarkResult { bookmark, meta })
    }

    // -----------------------------------------------------------------------------
    // 7. PubkyAppFollow
    // -----------------------------------------------------------------------------

    #[wasm_bindgen(js_name = createFollow)]
    pub fn create_follow(&self, followee_id: String) -> Result<FollowResult, String> {
        let follow = PubkyAppFollow::new();
        follow.validate(Some(&followee_id))?; // No ID in follow, so we pass user ID or empty

        // Path requires the user ID
        let path = follow.create_path(&followee_id);
        let meta = Meta::from_object(Some(&followee_id), self.pubky_id.clone(), path);

        Ok(FollowResult { follow, meta })
    }

    // -----------------------------------------------------------------------------
    // 8. PubkyAppMute
    // -----------------------------------------------------------------------------

    #[wasm_bindgen(js_name = createMute)]
    pub fn create_mute(&self, mutee_id: String) -> Result<MuteResult, String> {
        let mute = PubkyAppMute::new();
        mute.validate(Some(&mutee_id))?;

        let path = mute.create_path(&mutee_id);
        let meta = Meta::from_object(Some(&mutee_id), self.pubky_id.clone(), path);

        Ok(MuteResult { mute, meta })
    }

    // -----------------------------------------------------------------------------
    // 9. PubkyAppLastRead
    // -----------------------------------------------------------------------------

    #[wasm_bindgen(js_name = createLastRead)]
    pub fn create_last_read(&self) -> Result<LastReadResult, String> {
        let last_read = PubkyAppLastRead::new();
        last_read.validate(None)?;

        let path = last_read.create_path();
        let meta = Meta::from_object(None, self.pubky_id.clone(), path);

        Ok(LastReadResult { last_read, meta })
    }

    // -----------------------------------------------------------------------------
    // 10. PubkyAppBlob
    // -----------------------------------------------------------------------------

    #[wasm_bindgen(js_name = createBlob)]
    pub fn create_blob(&self, blob_data: JsValue) -> Result<BlobResult, String> {
        // Convert from JsValue (Uint8Array in JS) -> Vec<u8> in Rust
        let data_vec: Vec<u8> = from_value(blob_data).map_err(|e| e.to_string())?;

        // Create the PubkyAppBlob
        let blob = PubkyAppBlob(data_vec);

        // Generate ID and path
        let id = blob.create_id();
        blob.validate(Some(&id))?;

        let path = blob.create_path(&id);
        let meta = Meta::from_object(Some(&id), self.pubky_id.clone(), path);

        Ok(BlobResult { blob, meta })
    }
}

/// This object represents the result of parsing a Pubky URI. It contains:
/// - `user_id`: the parsed user ID as a string.
/// - `resource`: a string representing the kind of resource (derived from internal `Resource` enum Display).
/// - `resource_id`: an optional resource identifier (if applicable).
#[wasm_bindgen]
pub struct ParsedUriResult {
    #[wasm_bindgen(skip)]
    user_id: String,
    #[wasm_bindgen(skip)]
    resource: String,
    #[wasm_bindgen(skip)]
    resource_id: Option<String>,
}

#[wasm_bindgen]
impl ParsedUriResult {
    /// Returns the user ID.
    #[wasm_bindgen(getter)]
    pub fn user_id(&self) -> String {
        self.user_id.clone()
    }

    /// Returns the resource kind.
    #[wasm_bindgen(getter)]
    pub fn resource(&self) -> String {
        self.resource.clone()
    }

    /// Returns the resource ID if present.
    #[wasm_bindgen(getter)]
    pub fn resource_id(&self) -> Option<String> {
        self.resource_id.clone()
    }
}

/// Parses a Pubky URI and returns a strongly typed `ParsedUriResult`.
///
/// This function wraps the internal ParsedUri ust parsing logic. It converts the result into a
/// strongly typed object that is easier to use in TypeScript.
///
/// # Parameters
///
/// - `uri`: A string slice representing the Pubky URI. The URI should follow the format:
///   `pubky://<user_id>/pub/pubky.app/<resource>[/<id>]`.
///
/// # Returns
///
/// On success, returns a `ParsedUriResult` with:
/// - `user_id`: the parsed user ID,
/// - `resource`: a string (derived from the Display implementation of internal `Resource` enum),
/// - `resource_id`: an optional resource identifier (if applicable).
///
/// On failure, returns a JavaScript error (`String`) containing an error message.
///
/// # Example (TypeScript)
///
/// ```typescript
/// import { parse_uri } from "pubky-app-specs";
///
/// try {
///   const result = parse_uri("pubky://user123/pub/pubky.app/posts/abc123");
///   console.log(result.user_id);        // e.g. "user123"
///   console.log(result.resource);    // e.g. "posts"
///   console.log(result.resource_id);      // e.g. "abc123" or null
/// } catch (error) {
///   console.error("Error parsing URI:", error);
/// }
/// ```
#[wasm_bindgen]
pub fn parse_uri(uri: &str) -> Result<ParsedUriResult, String> {
    // Attempt to parse the URI using ParsedUri logic.
    let parsed = ParsedUri::try_from(uri)?;

    // Build and return the strongly typed result.
    Ok(ParsedUriResult {
        user_id: parsed.user_id.to_string(),
        resource: parsed.resource.to_string(),
        resource_id: parsed.resource.id(),
    })
}
