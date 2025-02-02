pub mod blob;
pub mod bookmark;
pub mod feed;
pub mod file;
pub mod follow;
pub mod last_read;
pub mod mute;
pub mod post;
pub mod tag;
pub mod user;

/// A unified enum wrapping all PubkyApp objects.
#[derive(Debug, Clone)]
pub enum PubkyAppObject {
    User(user::PubkyAppUser),
    Post(post::PubkyAppPost),
    Follow(follow::PubkyAppFollow),
    Mute(mute::PubkyAppMute),
    Bookmark(bookmark::PubkyAppBookmark),
    Tag(tag::PubkyAppTag),
    File(file::PubkyAppFile),
    Blob(blob::PubkyAppBlob),
    Feed(feed::PubkyAppFeed),
    LastRead(last_read::PubkyAppLastRead),
}
