import {
  init,
  validationLimits,
  createUser,
  createPost,
  createArticlePost,
  createCollectionPost,
  createVersion,
  planPublish,
  createBookmark,
  bookmarkTarget,
  createFollow,
  createTag,
  createMute,
  createFile,
  createFeed,
  feedLifecycle,
  readObject,
  validate,
  parseUri,
  deletionPaths,
  listPrefix,
  userUriBuilder,
  postUriBuilder,
  bookmarkUriBuilder,
  followUriBuilder,
  tagUriBuilder,
  muteUriBuilder,
  fileUriBuilder,
  feedUriBuilder,
  validMimeTypes,
  mimeToExt,
} from "./index.js";

// =============================================================================
// ANSI color helpers for pretty output
// =============================================================================
const c = {
  reset: "\x1b[0m",
  bright: "\x1b[1m",
  dim: "\x1b[2m",
  cyan: "\x1b[36m",
  yellow: "\x1b[33m",
  blue: "\x1b[34m",
  gray: "\x1b[90m",
  white: "\x1b[37m",
  bgBlue: "\x1b[44m",
};

const header = (title) => {
  console.log();
  console.log(`${c.bright}${c.blue}${title}${c.reset}`);
  console.log(c.gray + "-".repeat(70) + c.reset);
};
const item = (title) => console.log(`  ${c.yellow}> ${title}${c.reset}`);
const field = (label, value) => {
  console.log(`  ${c.dim}${label.padEnd(12)}${c.reset} ${c.white}${value}${c.reset}`);
};

// =============================================================================
// Setup: nothing runs until init()
// =============================================================================
const OTTO = "8kkppkmiubfq4pxn6f73nqrhhhgkb5xyfprntc9si3np9ydbotto";
const RIO = "dzswkfy7ek3bqnoc89jxuqqfbzhjrj6mi8qthgbxxcqkdugm3rio";

// Limits are plain data, readable before the wasm loads
console.log(`${c.dim}User name max before init():${c.reset} ${validationLimits.userNameMaxLength}`);

await init();

console.log();
console.log(`${c.bgBlue}${c.white}${c.bright}                    PUBKY SOCIAL SPECS - EXAMPLES                     ${c.reset}`);
console.log(`${c.dim}Writing as: ${c.reset}${c.cyan}${OTTO}${c.reset}`);

// =============================================================================
// 1. Profile
// =============================================================================
header("PROFILE");
const user = createUser(OTTO, { name: "Alice Smith", bio: "Software Developer", status: "active" });
field("URL", user.meta.url);
field("Stored", JSON.stringify(user.object));

// =============================================================================
// 2. Posts: every builder returns {object, meta}, object is what you PUT
// =============================================================================
header("POSTS");

item("Note");
const note = createPost(OTTO, { content: "Hello, Pubky world! This is my first post." });
field("ID", note.meta.id);
field("Path", note.meta.path);
console.log();

item("Reply");
const reply = createPost(OTTO, { content: "A reply to the first post", parent: postUriBuilder(OTTO, note.meta.id) });
field("Parent", reply.object.parent);
console.log();

item("Repost with embed");
const repost = createPost(OTTO, { content: "Look at this", embed: `pubky://${RIO}/pub/social/v1/posts/0033SREKPC4N0` });
field("Embed", repost.object.embed);
console.log();

item("Image post with attachments");
const photos = createPost(OTTO, {
  content: "Photos from my trip",
  kind: "image",
  attachments: [
    { uri: `pubky://${OTTO}/pub/social/v1/files/PZBQ010FF079VVZPQG1RNFN6DR.jpg`, alt: "beach", name: "beach.jpg" },
  ],
});
field("Attachments", JSON.stringify(photos.object.attachments));
console.log();

item("Article");
const article = createArticlePost(OTTO, { title: "Why Pubky", body: "# Why Pubky\n\nBecause keys, not accounts." });
field("Title", JSON.parse(article.object.content).title);
console.log();

item("Collection");
const collection = createCollectionPost(OTTO, {
  name: "Worth revisiting",
  items: [{ uri: postUriBuilder(RIO, "0033SREKPC4N0"), note: "the best one" }],
  layout: "list",
});
field("Envelope", collection.object.content);
console.log();

item("Locked post");
const locked = createPost(OTTO, { content: "A preview anyone can read", lock: `pubky://${RIO}/pub/locks/0034A0X7NJ52G` });
field("Lock", locked.object.lock);

// =============================================================================
// 3. Drafts: write under /priv/, publish by plan
// =============================================================================
header("DRAFT AND PUBLISH");
const draftMedia = createFile(OTTO, new Uint8Array([1, 2, 3]), "image/png", "private");
const draft = { content: "Not public yet", kind: "image", parent: null, embed: null, attachments: [{ uri: draftMedia.meta.url }] };
const version = createVersion(OTTO, draft, { root: "private", slug: "first-draft" });
field("Draft", version.path);
const plan = planPublish(OTTO, version.id, version.editId, draft);
field("Copy media", plan.mediaCopies.map(([from, to]) => `${from} -> ${to}`).join(", "));
field("Then PUT", plan.destPath);

// =============================================================================
// 4. Social actions
// =============================================================================
header("SOCIAL ACTIONS");

item("Bookmark (the target lives in the filename, under /priv/)");
const bookmark = createBookmark(OTTO, `pubky://${RIO}/pub/social/v1/posts/0033SREKPC4N0`);
field("Filename", bookmark.meta.id);
field("Target", bookmarkTarget(bookmark.meta.id, bookmark.object));
console.log();

item("Follow");
field("URL", createFollow(OTTO, RIO).meta.url);
console.log();

item("Tag");
const tag = createTag(OTTO, userUriBuilder(OTTO), "developer");
field("ID", tag.meta.id);
field("Label", tag.object.label);
console.log();

item("Mute");
field("URL", createMute(OTTO, RIO).meta.url);

// =============================================================================
// 5. Media
// =============================================================================
header("MEDIA");
const file = createFile(OTTO, new Uint8Array([1, 2]), "application/pdf");
field("ID", file.meta.id);
field("URL", file.meta.url);
field("Size", `${file.object.bytes.length} bytes`);
field("Extension", `application/pdf maps to .${mimeToExt("application/pdf")}`);
field("Picker hint", `${validMimeTypes.length} types`);

// =============================================================================
// 6. Feeds: private by default, published by copy
// =============================================================================
header("FEEDS");
const feed = createFeed(OTTO, {
  tags: ["mountain", "hiking", "nature"],
  reach: "all",
  layout: "columns",
  sort: "recent",
  content: "image",
  name: "Outdoor Adventures",
  icon: "mountain",
});
field("ID", feed.meta.id);
field("Tags", feed.object.feed.tags.join(", ")); // folded, deduplicated and sorted
const lifecycle = feedLifecycle(feed.meta.id);
field("Publish", `${lifecycle.publish.from} -> ${lifecycle.publish.to}`);

// =============================================================================
// 7. Reading back and editing
// =============================================================================
header("READ, EDIT, VALIDATE");
const bytes = new TextEncoder().encode(JSON.stringify({ ...user.object, ext: { badge: "early" } }));
const read = readObject(user.meta.url, bytes);
field("Kind", read.kind);
read.object.status = "shipping";
validate(user.meta.url, read.object); // throws when the edit broke a rule
field("Kept", JSON.stringify(read.object.ext));

// =============================================================================
// 8. URIs, prefixes and deletion
// =============================================================================
header("URIS");
const uris = [
  ["User", userUriBuilder(OTTO)],
  ["Post", postUriBuilder(OTTO, note.meta.id)],
  ["Bookmark", bookmarkUriBuilder(OTTO, bookmark.meta.id)],
  ["Follow", followUriBuilder(OTTO, RIO)],
  ["Tag", tagUriBuilder(OTTO, tag.meta.id)],
  ["Mute", muteUriBuilder(OTTO, RIO)],
  ["File", fileUriBuilder(OTTO, `${file.meta.id}.pdf`)],
  ["Feed", feedUriBuilder(OTTO, feed.meta.id)],
];
for (const [name, uri] of uris) {
  console.log(`  ${c.dim}${name.padEnd(10)}${c.reset} ${c.cyan}${uri}${c.reset} ${c.dim}(${parseUri(uri).resource.kind})${c.reset}`);
}
field("List prefix", listPrefix(OTTO, "private"));
field("Delete file", deletionPaths({ kind: "file", id: file.meta.id, listings: [file.meta.path] }).join(", "));
console.log();
