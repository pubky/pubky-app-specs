use crate::{
    common::timestamp,
    traits::{HasIdPath, Validatable},
    APP_PATH, PUBLIC_PATH,
};
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use crate::traits::Json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// A user's pointer to a Collection post, identified by `(owner, post_id)`.
///
/// One primitive, one path:
///
/// `/pub/pubky.app/collections/<owner_id>/<post_id>`
///
/// The role of a given pointer is **inferred at read time** by comparing the
/// path's `<owner_id>` to the homeserver user (the URI host):
///
/// - `owner_id == homeserver_user` — **own-pointer**: a sovereign index entry
///   declaring "I created this collection." Lets a client list its own
///   collections by prefix-scan on its homeserver without a Nexus dependency.
///
/// - `owner_id != homeserver_user` — **follow-pointer**: a subscription to
///   someone else's collection. Indexers (e.g. Nexus) materialize this as a
///   `(:User)-[:FOLLOWS_COLLECTION]->(:Post {kind:'collection'})` edge and
///   emit a follow-notification to the target's owner. Deleting the pointer
///   removes the edge; no unfollow-notification is fired.
///
/// Body is intentionally minimal — `created_at` only — matching the
/// `PubkyAppFollow` precedent. The `(owner, post_id)` pair is fully encoded
/// in the path, so the spec primitive itself carries no role field, no
/// target field, and no subfolder discriminator.
///
/// Example URIs:
///
/// - own:    `pubky://A/pub/pubky.app/collections/A/0034A0X7NJ52G`
/// - follow: `pubky://A/pub/pubky.app/collections/B/0034A0X7NJ52G`
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppCollectionPointer {
    pub created_at: i64,
}

impl PubkyAppCollectionPointer {
    /// Creates a new pointer with `created_at` set to the current timestamp.
    pub fn new() -> Self {
        Self {
            created_at: timestamp(),
        }
    }

    /// Canonical builder. Works for **both** own-pointers (pass your own
    /// `user_id` as `owner_id`) and follow-pointers (pass the target
    /// collection's owner). Role is determined at read time by callers
    /// comparing `owner_id` against the URI host.
    pub fn create_path(owner_id: &str, post_id: &str) -> String {
        [
            PUBLIC_PATH,
            APP_PATH,
            Self::PATH_SEGMENT,
            owner_id,
            "/",
            post_id,
        ]
        .concat()
    }
}

impl HasIdPath for PubkyAppCollectionPointer {
    const PATH_SEGMENT: &'static str = "collections/";

    /// Trait form. `id` here is the composite `<owner_id>/<post_id>`.
    /// Provided so trait-generic call sites (e.g. `try_to_uri_str`) keep
    /// working; new callers should prefer the two-arg
    /// [`PubkyAppCollectionPointer::create_path`].
    fn create_path(id: &str) -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT, id].concat()
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppCollectionPointer {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = fromJson))]
    pub fn from_json(js_value: &JsValue) -> Result<Self, String> {
        Self::import_json(js_value)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = toJson))]
    pub fn to_json(&self) -> Result<JsValue, String> {
        self.export_json()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = createPath))]
    pub fn create_path_wasm(owner_id: &str, post_id: &str) -> String {
        Self::create_path(owner_id, post_id)
    }
}

#[cfg(target_arch = "wasm32")]
impl Json for PubkyAppCollectionPointer {}

impl Validatable for PubkyAppCollectionPointer {
    fn validate(&self, _id: Option<&str>) -> Result<(), String> {
        // Body is just `created_at`. Path components (owner pubkey, post_id)
        // are pre-validated by the URI parser before this is reached.
        // `created_at` bounds-checking matches the PubkyAppFollow precedent
        // (a TODO there, deferred symmetrically here).
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Validatable;

    const OWNER: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";
    const POST_ID: &str = "0034A0X7NJ52G";

    #[test]
    fn test_new() {
        let p = PubkyAppCollectionPointer::new();
        let now = timestamp();
        // within 1 second
        assert!(p.created_at <= now && p.created_at >= now - 1_000_000);
    }

    #[test]
    fn test_create_path() {
        let path = PubkyAppCollectionPointer::create_path(OWNER, POST_ID);
        assert_eq!(
            path,
            "/pub/pubky.app/collections/operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/0034A0X7NJ52G"
        );
    }

    #[test]
    fn test_create_path_trait_form() {
        // The HasIdPath trait form takes a composite "<owner>/<post>" id.
        let composite = format!("{OWNER}/{POST_ID}");
        let path = <PubkyAppCollectionPointer as HasIdPath>::create_path(&composite);
        let canonical = PubkyAppCollectionPointer::create_path(OWNER, POST_ID);
        assert_eq!(path, canonical);
    }

    #[test]
    fn test_round_trip() {
        let json = r#"{ "created_at": 1627849723 }"#;
        let p =
            <PubkyAppCollectionPointer as Validatable>::try_from(json.as_bytes(), POST_ID).unwrap();
        assert_eq!(p.created_at, 1627849723);
    }

    // ---------- WASM-target tests ----------

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn test_create_path_wasm_static() {
        // WASM-bound static path builder mirrors the Rust `create_path`.
        let path = PubkyAppCollectionPointer::create_path_wasm(OWNER, POST_ID);
        assert_eq!(
            path,
            "/pub/pubky.app/collections/operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo/0034A0X7NJ52G"
        );
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn test_create_own_collection_pointer_wasm_builder() {
        // Builder method that fills owner = self.pubky_id implicitly.
        use crate::PubkySpecsBuilder;
        let builder =
            PubkySpecsBuilder::new(OWNER.to_string()).expect("Failed to construct builder");
        let result = builder
            .create_own_collection_pointer(POST_ID.to_string())
            .expect("createOwnCollectionPointer should succeed");

        let pointer = result.collection_pointer();
        let now = timestamp();
        assert!(pointer.created_at <= now && pointer.created_at >= now - 1_000_000);

        let meta = result.meta();
        let expected_path = format!("/pub/pubky.app/collections/{OWNER}/{POST_ID}");
        assert_eq!(meta.path(), expected_path);
        // Composite id mirrors Resource::CollectionPointer.id() in uri_parser.
        assert_eq!(meta.id(), format!("{OWNER}/{POST_ID}"));
        // URL host is the builder's own pubky_id; for an own-pointer it
        // matches the path's owner segment (which is the role-inference
        // invariant tested in models/mod.rs).
        let expected_url = format!("pubky://{OWNER}{expected_path}");
        assert_eq!(meta.url(), expected_url);
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn test_create_followed_collection_pointer_wasm_builder() {
        // Builder method that takes an explicit target owner. URI host is
        // the builder's own pubky_id (the follower); path's owner segment
        // is the target's. owner != user_id ⇒ follow-pointer.
        use crate::PubkySpecsBuilder;
        let follower = OWNER;
        let target_owner = "pxnu33x7jtpx9ar1ytsi4yxbp6a5o36gwhffs8zoxmbuptici1jy";
        let builder =
            PubkySpecsBuilder::new(follower.to_string()).expect("Failed to construct builder");
        let result = builder
            .create_followed_collection_pointer(target_owner.to_string(), POST_ID.to_string())
            .expect("createFollowedCollectionPointer should succeed");

        let meta = result.meta();
        let expected_path = format!("/pub/pubky.app/collections/{target_owner}/{POST_ID}");
        assert_eq!(meta.path(), expected_path);
        // URL host = follower (builder's own), not the target.
        let expected_url = format!("pubky://{follower}{expected_path}");
        assert_eq!(meta.url(), expected_url);
        // Composite id reflects the path's <owner>/<post_id> segments.
        assert_eq!(meta.id(), format!("{target_owner}/{POST_ID}"));
    }
}
