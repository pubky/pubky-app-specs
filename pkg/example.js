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
  getValidMimeTypes,
} from "./index.js";

// =============================================================================
// ANSI color helpers for pretty output
// =============================================================================
const c = {
  reset: "\x1b[0m",
  bright: "\x1b[1m",
  dim: "\x1b[2m",
  cyan: "\x1b[36m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  blue: "\x1b[34m",
  magenta: "\x1b[35m",
  gray: "\x1b[90m",
  white: "\x1b[37m",
  bgBlue: "\x1b[44m",
};

const divider = () => console.log(c.gray + "─".repeat(70) + c.reset);
const header = (title) => {
  console.log();
  console.log(`${c.bright}${c.blue}${title}${c.reset}`);
  divider();
};
const field = (label, value) => {
  console.log(`  ${c.dim}${label.padEnd(12)}${c.reset} ${c.white}${value}${c.reset}`);
};

// =============================================================================
// Setup
// =============================================================================
const OTTO = "8kkppkmiubfq4pxn6f73nqrhhhgkb5xyfprntc9si3np9ydbotto";
const RIO = "dzswkfy7ek3bqnoc89jxuqqfbzhjrj6mi8qthgbxxcqkdugm3rio";

console.log();
console.log(`${c.bgBlue}${c.white}${c.bright}                                                                      ${c.reset}`);
console.log(`${c.bgBlue}${c.white}${c.bright}                    PUBKY APP SPECS - EXAMPLES                        ${c.reset}`);
console.log(`${c.bgBlue}${c.white}${c.bright}                                                                      ${c.reset}`);
console.log();

const specsBuilder = new PubkySpecsBuilder(OTTO);
console.log(`${c.dim}Using PubkyId: ${c.reset}${c.cyan}${OTTO}${c.reset}`);

// =============================================================================
// 1. User Profile
// =============================================================================
header("USER PROFILE");
const { user, meta: userMeta } = specsBuilder.createUser(
  "Alice Smith",
  "Software Developer",
  null,
  null,
  "active"
);
field("URL", userMeta.url);
field("Name", user.toJson().name);
field("Bio", user.toJson().bio);
field("Status", user.toJson().status);

// =============================================================================
// 2. Posts
// =============================================================================
header("POSTS");

// Simple post
console.log(`  ${c.yellow}▸ Simple Post${c.reset}`);
const { post, meta } = specsBuilder.createPost(
  "Hello, Pubky world! This is my first post.",
  PubkyAppPostKind.Short,
  null,
  null,
  null
);
field("ID", meta.id);
field("URL", meta.url);
field("Content", post.toJson().content);
console.log();

// Reply post
console.log(`  ${c.yellow}▸ Reply Post${c.reset}`);
const { post: replyPost, meta: replyMeta } = specsBuilder.createPost(
  "This is a reply to the first post!",
  PubkyAppPostKind.Short,
  userMeta.url,
  null,
  null
);
field("ID", replyMeta.id);
field("Parent", replyPost.toJson().parent);
console.log();

// Repost with embed
console.log(`  ${c.yellow}▸ Repost with Embed${c.reset}`);
const embed = new PubkyAppPostEmbed(
  `pubky://${RIO}/pub/pubky.app/posts/0033SREKPC4N0`,
  PubkyAppPostKind.Video
);
const { post: repost, meta: repostMeta } = specsBuilder.createPost(
  "Check out this awesome video!",
  PubkyAppPostKind.Short,
  null,
  embed,
  null
);
field("ID", repostMeta.id);
field("Embed URI", repost.toJson().embed.uri);
field("Embed Kind", repost.toJson().embed.kind);
console.log();

// Post with attachments
console.log(`  ${c.yellow}▸ Post with Attachments${c.reset}`);
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
field("ID", postWithAttachmentsMeta.id);
field("Attachments", `${postWithAttachments.toJson().attachments.length} files`);

// =============================================================================
// 3. Social Actions
// =============================================================================
header("SOCIAL ACTIONS");

// Bookmark
console.log(`  ${c.yellow}▸ Bookmark${c.reset}`);
const { bookmark, meta: bookmarkMeta } = specsBuilder.createBookmark(
  `pubky://${RIO}/pub/pubky.app/posts/0033SREKPC4N0`
);
field("ID", bookmarkMeta.id);
field("URI", bookmark.toJson().uri);
console.log();

// Follow
console.log(`  ${c.yellow}▸ Follow${c.reset}`);
const { follow, meta: followMeta } = specsBuilder.createFollow(RIO);
field("ID", followMeta.id);
field("URL", followMeta.url);
console.log();

// Tag
console.log(`  ${c.yellow}▸ Tag${c.reset}`);
const { tag, meta: tagMeta } = specsBuilder.createTag(
  `pubky://${OTTO}/pub/pubky.app/profile.json`,
  "developer"
);
field("ID", tagMeta.id);
field("Label", tag.toJson().label);
field("URI", tag.toJson().uri);
console.log();

// Mute
console.log(`  ${c.yellow}▸ Mute${c.reset}`);
const { mute, meta: muteMeta } = specsBuilder.createMute(RIO);
field("ID", muteMeta.id);
field("URL", muteMeta.url);

// =============================================================================
// 4. Files & Blobs
// =============================================================================
header("FILES & BLOBS");

// Blob
console.log(`  ${c.yellow}▸ Blob (raw data)${c.reset}`);
const blobData = Array.from({ length: 8 }, () => Math.floor(Math.random() * 256));
const { blob, meta: blobMeta } = specsBuilder.createBlob(blobData);
field("ID", blobMeta.id);
field("URL", blobMeta.url);
field("Size", `${blobData.length} bytes`);
console.log();

// File
console.log(`  ${c.yellow}▸ File (metadata)${c.reset}`);
const { file, meta: fileMeta } = specsBuilder.createFile(
  "vacation-photos.pdf",
  blobMeta.url,
  "application/pdf",
  1024
);
field("ID", fileMeta.id);
field("Name", file.toJson().name);
field("Type", file.toJson().content_type);
field("Size", `${file.toJson().size} bytes`);
field("Source", file.toJson().src);

// =============================================================================
// 5. Feeds & LastRead
// =============================================================================
header("FEEDS & LAST READ");

// Feed
console.log(`  ${c.yellow}▸ Custom Feed${c.reset}`);
const { feed, meta: feedMeta } = specsBuilder.createFeed(
  ["mountain", "hiking", "nature"],
  "all",
  "columns",
  "recent",
  "image",
  "Outdoor Adventures"
);
field("ID", feedMeta.id);
field("Name", feed.toJson().name);
field("Tags", feed.toJson().feed.tags.join(", "));
field("Layout", feed.toJson().feed.layout);
field("Sort", feed.toJson().feed.sort);
console.log();

// LastRead
console.log(`  ${c.yellow}▸ Last Read Marker${c.reset}`);
const { last_read, meta: lastReadMeta } = specsBuilder.createLastRead();
field("URL", lastReadMeta.url);
field("Timestamp", new Date(last_read.toJson().timestamp / 1000).toISOString());

// =============================================================================
// 6. URI Builders
// =============================================================================
header("URI BUILDERS");
const uris = [
  ["User", userUriBuilder(OTTO)],
  ["Post", postUriBuilder(OTTO, meta.id)],
  ["Bookmark", bookmarkUriBuilder(OTTO, bookmarkMeta.id)],
  ["Follow", followUriBuilder(OTTO, RIO)],
  ["Tag", tagUriBuilder(OTTO, tagMeta.id)],
  ["Mute", muteUriBuilder(OTTO, RIO)],
  ["LastRead", lastReadUriBuilder(OTTO)],
  ["Blob", blobUriBuilder(OTTO, blobMeta.id)],
  ["File", fileUriBuilder(OTTO, fileMeta.id)],
  ["Feed", feedUriBuilder(OTTO, feedMeta.id)],
];
uris.forEach(([name, uri]) => {
  console.log(`  ${c.dim}${name.padEnd(10)}${c.reset} ${c.cyan}${uri}${c.reset}`);
});

// =============================================================================
// 7. Valid MIME Types
// =============================================================================
header("VALID MIME TYPES");
const validMimeTypes = getValidMimeTypes();
console.log(`  ${c.dim}Total types:${c.reset} ${c.bright}${validMimeTypes.length}${c.reset}`);
console.log();

// Group by category
const categories = {
  "Images": validMimeTypes.filter(t => t.startsWith("image/")),
  "Videos": validMimeTypes.filter(t => t.startsWith("video/")),
  "Audio": validMimeTypes.filter(t => t.startsWith("audio/")),
  "Documents": validMimeTypes.filter(t => t.startsWith("application/") || t.startsWith("text/")),
};

Object.entries(categories).forEach(([category, types]) => {
  if (types.length > 0) {
    console.log(`  ${c.yellow}${category}:${c.reset}`);
    types.forEach(type => console.log(`    ${c.dim}-${c.reset} ${type}`));
    console.log();
  }
});

// Validation example
console.log(`  ${c.yellow}Validation Example:${c.reset}`);
const testTypes = ["image/png", "video/mp4", "application/x-executable"];
testTypes.forEach(type => {
  const isValid = validMimeTypes.includes(type);
  const icon = isValid ? `${c.green}[ok]${c.reset}` : `${c.magenta}[x]${c.reset}`;
  console.log(`    ${icon} ${type}`);
});

// =============================================================================
// Done!
// =============================================================================
console.log();
console.log(`${c.bgBlue}${c.white}${c.bright}                                                                      ${c.reset}`);
console.log(`${c.bgBlue}${c.white}${c.bright}                 ALL EXAMPLES COMPLETED SUCCESSFULLY!                 ${c.reset}`);
console.log(`${c.bgBlue}${c.white}${c.bright}                                                                      ${c.reset}`);
console.log();
