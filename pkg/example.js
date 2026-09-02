import {
  PubkySocialPostKind,
  PubkySpecsBuilder,
  PubkySocialAttachment,
  userUriBuilder,
  postUriBuilder,
  bookmarkUriBuilder,
  followUriBuilder,
  tagUriBuilder,
  muteUriBuilder,
  blobUriBuilder,
  fileUriBuilder,
  feedUriBuilder,
  getValidMimeTypes,
} from "./index.js";
import { getValidationLimits, validationLimits } from "./validationLimits.js";

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
  bgGreen: "\x1b[42m",
  black: "\x1b[30m",
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
  PubkySocialPostKind.Note
);
field("ID", meta.id);
field("URL", meta.url);
field("Content", post.toJson().content);
console.log();

// Reply post
console.log(`  ${c.yellow}▸ Reply Post${c.reset}`);
const { post: replyPost, meta: replyMeta } = specsBuilder.createPost(
  "This is a reply to the first post!",
  PubkySocialPostKind.Note,
  userMeta.url
);
field("ID", replyMeta.id);
field("Parent", replyPost.toJson().parent);
console.log();

// Repost with embed
console.log(`  ${c.yellow}▸ Repost with Embed${c.reset}`);
const { post: repost, meta: repostMeta } = specsBuilder.createPost(
  "Check out this awesome video!",
  PubkySocialPostKind.Note,
  null,
  `pubky://${RIO}/pub/social/v1/posts/0033SREKPC4N0`
);
field("ID", repostMeta.id);
field("Embed", repost.toJson().embed);
console.log();

// Post with attachments
console.log(`  ${c.yellow}▸ Post with Attachments${c.reset}`);
const { post: postWithAttachments, meta: postWithAttachmentsMeta } = specsBuilder.createPost(
  "Check out these photos from my trip!",
  PubkySocialPostKind.Image,
  null,
  null,
  [
    new PubkySocialAttachment(`pubky://${OTTO}/pub/social/v1/files/0034A0X7NJ52G`, "beach", "beach.jpg"),
    new PubkySocialAttachment(`pubky://${OTTO}/pub/social/v1/files/0034A0X7NJ53H`, null, null),
  ]
);
field("ID", postWithAttachmentsMeta.id);
field("Attachments", `${postWithAttachments.toJson().attachments.length} files`);
console.log();

// Article
console.log(`  ${c.yellow}▸ Article${c.reset}`);
const { post: articlePost, meta: articleMeta } = specsBuilder.createArticlePost(
  "Why Pubky",
  "# Why Pubky\n\nBecause keys, not accounts.",
  `pubky://${OTTO}/pub/social/v1/files/0034A0X7NJ52G`
);
field("ID", articleMeta.id);
field("Title", JSON.parse(articlePost.toJson().content).title);
console.log();

// Locked post (gated behind a lock server)
console.log(`  ${c.yellow}▸ Locked Post${c.reset}`);
const lockUrl = `pubky://${RIO}/pub/locks/0034A0X7NJ52G`;
const { post: lockedPost, meta: lockedPostMeta } = specsBuilder.createPost(
  "We were reckless adopting Lightning without understanding the tradeoffs.",
  PubkySocialPostKind.Note,
  null,
  null,
  null,
  lockUrl
);
field("ID", lockedPostMeta.id);
field("Content", lockedPost.toJson().content);
field("Lock", lockedPost.lock);

// =============================================================================
// 3. Social Actions
// =============================================================================
header("SOCIAL ACTIONS");

// Bookmark
console.log(`  ${c.yellow}▸ Bookmark${c.reset}`);
const { bookmark, meta: bookmarkMeta } = specsBuilder.createBookmark(
  `pubky://${RIO}/pub/social/v1/posts/0033SREKPC4N0`
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
  `pubky://${OTTO}/pub/social/v1/profile.json`,
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
// 5. Feeds
// =============================================================================
header("FEEDS");

// Feed
console.log(`  ${c.yellow}▸ Custom Feed${c.reset}`);
const { feed, meta: feedMeta } = specsBuilder.createFeed({
  tags: ["mountain", "hiking", "nature"],
  reach: "all",
  layout: "columns",
  sort: "recent",
  content: "image",
  name: "Outdoor Adventures",
  icon: "mountain",
});
field("ID", feedMeta.id);
field("Name", feed.toJson().name);
field("Icon", feed.toJson().icon);
field("Tags", feed.toJson().feed.tags.join(", "));
field("Layout", feed.toJson().feed.layout);
field("Sort", feed.toJson().feed.sort);
console.log();

// WoT feed with domain tags
console.log(`  ${c.yellow}▸ WoT Feed with Domain Tags${c.reset}`);
const { feed: wotFeed, meta: wotFeedMeta } = specsBuilder.createFeed({
  tags: ["rust"],
  reach: "wot",
  layout: "columns",
  sort: "recent",
  content: "image",
  name: "Rust WoT",
  domainTags: ["synonym"],
  icon: "users",
});
field("ID", wotFeedMeta.id);
field("Reach", wotFeed.toJson().feed.reach);
field("Domain Tags", wotFeed.toJson().feed.domain_tags.join(", "));
console.log();

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
// 8. Validation Limits
// =============================================================================
header("VALIDATION LIMITS");
const limitsCopy = getValidationLimits();

field("User name max", validationLimits.userNameMaxLength);
field("Note post max", validationLimits.postNoteContentMaxLength);
field("Max attachments", validationLimits.postAttachmentsMaxCount);
field("Copy matches", JSON.stringify(limitsCopy) === JSON.stringify(validationLimits));

// =============================================================================
// Done!
// =============================================================================
console.log();
console.log(`${c.bgGreen}${c.black}${c.bright}                                                                      ${c.reset}`);
console.log(`${c.bgGreen}${c.black}${c.bright}                 ALL EXAMPLES COMPLETED SUCCESSFULLY!                 ${c.reset}`);
console.log(`${c.bgGreen}${c.black}${c.bright}                                                                      ${c.reset}`);
console.log();
