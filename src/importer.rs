use crate::models::PubkyAppObject;
use crate::traits::Validatable;
use crate::uri_parser::{ParsedUri, Resource};
use crate::{
    PubkyAppBlob, PubkyAppBookmark, PubkyAppFeed, PubkyAppFile, PubkyAppFollow, PubkyAppLastRead,
    PubkyAppMute, PubkyAppPost, PubkyAppTag, PubkyAppUser,
};

/// Given a URI and a blob (raw data from the homeserver),
/// this function returns the fully formed PubkyAppObject.
pub fn import_object(uri: &str, blob: &[u8]) -> Result<PubkyAppObject, String> {
    let parsed_uri = ParsedUri::try_from(uri)?;
    import_object_from_resource(&parsed_uri.resource, blob)
}

/// Given a Resource and a blob (raw data from the homeserver),
/// this function returns the fully formed PubkyAppObject.
pub fn import_object_from_resource(
    resource: &Resource,
    blob: &[u8],
) -> Result<PubkyAppObject, String> {
    match resource {
        Resource::User => {
            // For a user, no ID is needed (or you may use an empty string)
            let user = <PubkyAppUser as Validatable>::try_from(blob, "")?;
            Ok(PubkyAppObject::User(user))
        }
        Resource::Post(post_id) => {
            let post = <PubkyAppPost as Validatable>::try_from(blob, &post_id)?;
            Ok(PubkyAppObject::Post(post))
        }
        Resource::Follow(follow_id) => {
            // Use the follow id from the parsed URI.
            let follow = <PubkyAppFollow as Validatable>::try_from(blob, &follow_id)?;
            Ok(PubkyAppObject::Follow(follow))
        }
        Resource::Mute(muted_id) => {
            let mute = <PubkyAppMute as Validatable>::try_from(blob, &muted_id)?;
            Ok(PubkyAppObject::Mute(mute))
        }
        Resource::Bookmark(bookmark_id) => {
            let bookmark = <PubkyAppBookmark as Validatable>::try_from(blob, &bookmark_id)?;
            Ok(PubkyAppObject::Bookmark(bookmark))
        }
        Resource::Tag(tag_id) => {
            let tag = <PubkyAppTag as Validatable>::try_from(blob, &tag_id)?;
            Ok(PubkyAppObject::Tag(tag))
        }
        Resource::File(file_id) => {
            let file = <PubkyAppFile as Validatable>::try_from(blob, &file_id)?;
            Ok(PubkyAppObject::File(file))
        }
        Resource::Blob(blob_id) => {
            let blob_obj = <PubkyAppBlob as Validatable>::try_from(blob, &blob_id)?;
            Ok(PubkyAppObject::Blob(blob_obj))
        }
        Resource::Feed(feed_id) => {
            let feed = <PubkyAppFeed as Validatable>::try_from(blob, &feed_id)?;
            Ok(PubkyAppObject::Feed(feed))
        }
        Resource::LastRead => {
            let last_read = <PubkyAppLastRead as Validatable>::try_from(blob, "")?;
            Ok(PubkyAppObject::LastRead(last_read))
        }
        Resource::Unknown => Err(format!("Unrecognized resource {:?}", resource)),
    }
}
