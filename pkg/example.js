import {
  PubkyAppPostKind,
  PubkySpecsBuilder,
  PubkyAppPostEmbed,
  userUriBuilder,
  postUriBuilder,
  bookmarkUriBuilder,
  followUriBuilder,
  tagUriBuilder,
  muteUriBuilder,
  lastReadUriBuilder,
  blobUriBuilder,
  fileUriBuilder,
  feedUriBuilder,
} from "./index.js";

const OTTO = "8kkppkmiubfq4pxn6f73nqrhhhgkb5xyfprntc9si3np9ydbotto";
const RIO = "dzswkfy7ek3bqnoc89jxuqqfbzhjrj6mi8qthgbxxcqkdugm3rio";

// 👤 Create a user profile
console.log("👤 Creating User Profile...");
const specsBuilder = new PubkySpecsBuilder(OTTO);
const { user, meta: userMeta } = specsBuilder.createUser(
  "Alice Smith",
  "Software Developer",
  null,
  null,
  "active"
);
console.log("User Profile URL:", userMeta.url);
console.log("User Data:", JSON.stringify(user.toJson(), null, 2));
console.log("-".repeat(60));

// 📝 Create different posts
console.log("📝 Creating First Post...");
const { post, meta } = specsBuilder.createPost(
  "Hello, Pubky world! This is my first post.",
  PubkyAppPostKind.Short,
  null,
  null,
  null
);
console.log("Post ID:", meta.id);
console.log("Post URL:", meta.url);
console.log("Post Data:", JSON.stringify(post.toJson(), null, 2));
console.log("-".repeat(60));

console.log("💬 Creating Reply Post...");
const { post: replyPost, meta: replyMeta } = specsBuilder.createPost(
  "This is a reply to the first post!",
  PubkyAppPostKind.Short,
  userMeta.url,
  null,
  null
);
console.log("Reply Post ID:", replyMeta.id);
console.log("Reply Post URL:", replyMeta.url);
console.log("Reply Data:", JSON.stringify(replyPost.toJson(), null, 2));
console.log("-".repeat(60));

console.log("🔄 Creating Repost with Embed...");
let embeed = new PubkyAppPostEmbed(
  `pubky://${RIO}/pub/pubky.app/posts/0033SREKPC4N0`,
  PubkyAppPostKind.Video
);
const { post: repost, meta: repostMeta } = specsBuilder.createPost(
  "This is a repost to random post!",
  PubkyAppPostKind.Short,
  null,
  embeed,
  null
);
console.log("Repost Post ID:", repostMeta.id);
console.log("Repost Post URL:", repostMeta.url);
console.log("Repost Data:", JSON.stringify(repost.toJson(), null, 2));
console.log("-".repeat(60));

console.log("📎 Creating Post with Attachments...");
const { post: postWithAttachments, meta: postWithAttachmentsMeta } = specsBuilder.createPost(
  "Check out these photos from my trip!",
  PubkyAppPostKind.Image,
  null,
  null,
  [
    `pubky://${OTTO}/pub/pubky.app/files/0034A0X7NJ52G`,
    `pubky://${OTTO}/pub/pubky.app/files/0034A0X7NJ53H`,
  ]
);
console.log("Post ID:", postWithAttachmentsMeta.id);
console.log("Post URL:", postWithAttachmentsMeta.url);
console.log("Attachments:", postWithAttachments.toJson().attachments);
console.log("Post Data:", JSON.stringify(postWithAttachments.toJson(), null, 2));
console.log("-".repeat(60));

console.log("🔖 Creating Bookmark...");
let { bookmark, meta: bookmarkMeta } = specsBuilder.createBookmark(
  `pubky://${RIO}/pub/pubky.app/posts/0033SREKPC4N0`
);
console.log("Bookmark ID:", bookmarkMeta.id);
console.log("Bookmark URL:", bookmarkMeta.url);
console.log("Bookmark Data:", JSON.stringify(bookmark.toJson(), null, 2));
console.log("-".repeat(60));

console.log("👥 Creating Follow...");
let { follow, meta: followMeta } = specsBuilder.createFollow(RIO);
console.log("Follow ID:", followMeta.id);
console.log("Follow URL:", followMeta.url);
console.log("Follow Data:", JSON.stringify(follow.toJson(), null, 2));
console.log("-".repeat(60));

console.log("🏷️ Creating Tag...");
let { tag, meta: tagMeta } = specsBuilder.createTag(
  `pubky://${OTTO}/pub/pubky.app/profile.json`,
  "otto"
);
console.log("Tag ID:", tagMeta.id);
console.log("Tag URL:", tagMeta.url);
console.log("Tag Data:", JSON.stringify(tag.toJson(), null, 2));
console.log("-".repeat(60));

console.log("🔇 Creating Mute...");
let { mute, meta: muteMeta } = specsBuilder.createMute(RIO);
console.log("Mute ID:", muteMeta.id);
console.log("Mute URL:", muteMeta.url);
console.log("Mute Data:", JSON.stringify(mute.toJson(), null, 2));
console.log("-".repeat(60));

console.log("📖 Creating Last Read...");
let { last_read, meta: lastReadMeta } = specsBuilder.createLastRead(RIO);
console.log("LastRead Timestamp:", lastReadMeta.url);
console.log("LastRead Data:", JSON.stringify(last_read.toJson(), null, 2));
console.log("-".repeat(60));

console.log("💾 Creating Blob...");
let { blob, meta: blobMeta } = specsBuilder.createBlob(
  Array.from({ length: 8 }, () => Math.floor(Math.random() * 256))
);
console.log("Blob ID:", blobMeta.id);
console.log("Blob URL:", blobMeta.url);
console.log("Blob Data:", JSON.stringify(blob.toJson(), null, 2));
console.log("-".repeat(60));

console.log("📄 Creating File...");
let { file, meta: fileMeta } = specsBuilder.createFile(
  "My adventures",
  blobMeta.url,
  "application/pdf",
  88
);
console.log("File ID:", fileMeta.id);
console.log("File URL:", fileMeta.url);
console.log("File Data:", JSON.stringify(file.toJson(), null, 2));
console.log("-".repeat(60));

console.log("📰 Creating Feed...");
let { feed, meta: feedMeta } = specsBuilder.createFeed(
  ["mountain", "hike"],
  "all",
  "columns",
  "recent",
  "image",
  "nature"
);
console.log("Feed ID:", feedMeta.id);
console.log("Feed URL:", feedMeta.url);
console.log("Feed Data:", JSON.stringify(feed.toJson(), null, 2));
console.log("-".repeat(60));

console.log("🔗 Utility Functions...");
console.log("User URI:", userUriBuilder(OTTO));
console.log("Post URI:", postUriBuilder(OTTO, meta.id));
console.log("Bookmark URI:", bookmarkUriBuilder(OTTO, bookmarkMeta.id));
console.log("Follow URI:", followUriBuilder(OTTO, RIO));
console.log("Tag URI:", tagUriBuilder(OTTO, tagMeta.id));
console.log("Mute URI:", muteUriBuilder(OTTO, RIO));
console.log("LastRead URI:", lastReadUriBuilder(OTTO));
console.log("Blob URI:", blobUriBuilder(OTTO, blobMeta.id));
console.log("File URI:", fileUriBuilder(OTTO, fileMeta.id));

console.log("=".repeat(60));
console.log("🎉 All Pubky App Specs examples completed successfully!");
console.log("=".repeat(60));
