import { PubkyAppPostKind, PubkySpecsBuilder, PubkyAppPostEmbed, postUriBuilder, bookmarkUriBuilder, followUriBuilder, userUriBuilder } from "./index.js";
import assert from "assert";

const OTTO = "8kkppkmiubfq4pxn6f73nqrhhhgkb5xyfprntc9si3np9ydbotto";
const RIO = "dzswkfy7ek3bqnoc89jxuqqfbzhjrj6mi8qthgbxxcqkdugm3rio";

describe("PubkySpecs Example Objects Tests", () => {
  let specsBuilder;

  beforeEach(() => {
    specsBuilder = new PubkySpecsBuilder(OTTO);
  });

  describe("User Pubky-app-specs", () => {
    it("should create user with correct properties", () => {
      const { user, meta: userMeta } = specsBuilder.createUser(
        "Alice Smith",
        "Software Developer", 
        null, 
        null, 
        "active"
      );

      // Test meta properties
      assert.ok(userMeta.url, "User should have a URL");
      assert.ok(userMeta.url.includes(OTTO), "URL should contain user ID");
      assert.ok(userMeta.url.includes("profile.json"), "URL should point to profile.json");

      // Test user object content
      const userJson = user.toJson();
      assert.strictEqual(userJson.name, "Alice Smith", "User name should match");
      assert.strictEqual(userJson.bio, "Software Developer", "User bio should match");
      assert.strictEqual(userJson.status, "active", "User status should match");
    });
  });

  describe("Post Pubky-app-specs", () => {
    it("should create basic post with correct properties", () => {
      const postContent = "Hello, Pubky world! This is my first post."
      const { post, meta } = specsBuilder.createPost(
        postContent, 
        PubkyAppPostKind.Short, 
        null, 
        null, 
        null
      );

      // Test meta properties
      assert.ok(meta.id, "Post should have an ID");
      assert.ok(meta.url, "Post should have a URL");
      const postChunks = meta.url.split("/")
      assert.strictEqual(postChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(postChunks[5], "posts", "URL should contain posts path");
      assert.strictEqual(postChunks[6], meta.id, "URL should contain post ID");

      // Test post content
      const postJson = post.toJson();
      assert.strictEqual(postJson.content, postContent, "Post content should match");
      assert.strictEqual(postJson.kind, "short", "Post kind should match");
    });

    it("should create reply post with parent reference", () => {
      const parentPostUriRaw = `pubky://${RIO}/pub/pubky.app/posts/0033SSE3B1FQ0`
      const parentPostUri = postUriBuilder(RIO, "0033SSE3B1FQ0")
      assert.strictEqual(parentPostUri, parentPostUriRaw, "Parent post URI should match");

      const { post: replyPost } = specsBuilder.createPost(
        "This is a reply to the first post!", 
        PubkyAppPostKind.Short, 
        parentPostUriRaw, 
        null, 
        null
      );

      // Test reply content
      const replyJson = replyPost.toJson();
      assert.strictEqual(replyJson.parent, parentPostUriRaw, "Reply should reference parent URL");
    });

    it("should create repost with embed", () => {
      const embedUriRaw = `pubky://${RIO}/pub/pubky.app/posts/0033SREKPC4N0`
      const embedUriFromBuilder = postUriBuilder(RIO, "0033SREKPC4N0")
      assert.strictEqual(embedUriFromBuilder, embedUriRaw, "Embed URI should match");

      const embed = new PubkyAppPostEmbed(embedUriRaw, PubkyAppPostKind.Video);
      const { post: repost } = specsBuilder.createPost(
        "This is a repost to random post!", 
        PubkyAppPostKind.Short, 
        null, 
        embed, 
        null
      );

      // Test repost content
      const repostJson = repost.toJson();
      assert.ok(repostJson.embed, "Repost should have embed");
      assert.strictEqual(repostJson.embed.uri, embedUriRaw, "Embed URI should match");
      assert.strictEqual(repostJson.embed.kind, "video", "Embed kind should match");
    });
  });

  describe("Bookmark Pubky-app-specs", () => {
    it("should create bookmark with correct properties", () => {
      const postUriRaw = `pubky://${RIO}/pub/pubky.app/posts/0033SREKPC4N0`

      const { bookmark, meta: bookmarkMeta } = specsBuilder.createBookmark(postUriRaw);
      const bookmarkUriFromBuilder = bookmarkUriBuilder(OTTO, bookmarkMeta.id)
      assert.strictEqual(bookmarkUriFromBuilder, bookmarkMeta.url, "Bookmark URI should match");

      // Test meta properties
      assert.ok(bookmarkMeta.id, "Bookmark should have an ID");
      assert.ok(bookmarkMeta.url, "Bookmark should have a URL");
      const bookmarkChunks = bookmarkMeta.url.split("/")
      assert.strictEqual(bookmarkChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(bookmarkChunks[5], "bookmarks", "URL should contain bookmarks path");
      assert.strictEqual(bookmarkChunks[6], bookmarkMeta.id, "URL should contain bookmark ID");

      // Test bookmark content
      const bookmarkJson = bookmark.toJson();
      assert.strictEqual(bookmarkJson.uri, postUriRaw, "Bookmark URI should match");
      assert.ok(bookmarkJson.created_at, "Bookmark should have created_at timestamp");
      assert.ok(typeof bookmarkJson.created_at === "number", "created_at should be a number");
    });
  });

  describe("Follow Pubky-app-specs", () => {
    it("should create follow with correct properties", () => {
      const { follow, meta: followMeta } = specsBuilder.createFollow(RIO);
      const followUriFromBuilder = followUriBuilder(OTTO, RIO)
      assert.strictEqual(followUriFromBuilder, followMeta.url, "Follow URI should match");

      // Test meta properties
      assert.strictEqual(followMeta.id, RIO, "Follow ID should be the user being followed");
      assert.ok(followMeta.url, "Follow should have a URL");
      const followChunks = followMeta.url.split("/")
      assert.strictEqual(followChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(followChunks[5], "follows", "URL should contain follows path");
      assert.strictEqual(followChunks[6], RIO, "URL should contain follow ID");

      // Test follow content
      const followJson = follow.toJson();
      assert.ok(followJson.created_at, "Follow should have created_at timestamp");
      assert.ok(typeof followJson.created_at === "number", "created_at should be a number");
    });
  });

  describe("Tag Pubky-app-specs", () => {
    it("should create tag with correct properties", () => {
      const userUriRaw = `pubky://${OTTO}/pub/pubky.app/profile.json`;
      const userUriFromBuilder = userUriBuilder(OTTO)
      assert.strictEqual(userUriFromBuilder, userUriRaw, "User URI should match");

      const { tag, meta: tagMeta } = specsBuilder.createTag(userUriRaw, "otto");

      // Test meta properties
      assert.ok(tagMeta.id, "Tag should have an ID");
      assert.ok(tagMeta.url, "Tag should have a URL");
      const tagChunks = tagMeta.url.split("/")
      assert.strictEqual(tagChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(tagChunks[5], "tags", "URL should contain tags path");
      assert.strictEqual(tagChunks[6], tagMeta.id, "URL should contain tag ID");

      // Test tag content
      const tagJson = tag.toJson();
      assert.strictEqual(tagJson.uri, userUriRaw, "Tag URI should match");
      assert.strictEqual(tagJson.label, "otto", "Tag label should match");
      assert.ok(tagJson.created_at, "Tag should have created_at timestamp");
      assert.ok(typeof tagJson.created_at === "number", "created_at should be a number");
    });
    it("cannot create a tag with invalid characters (comma, colon, space)", () => {
      const userUriRaw = `pubky://${OTTO}/pub/pubky.app/profile.json`;
      const userUriFromBuilder = userUriBuilder(OTTO);
      assert.strictEqual(userUriFromBuilder, userUriRaw, "User URI should match");

      const invalidCases = [
        { label: "otto,rio", invalidChar: "," },
        { label: "otto:rio", invalidChar: ":" },
        { label: "otto rio", invalidChar: " " },
      ];

      invalidCases.forEach(({ label, invalidChar }) => {
        assert.throws(
          () => {
            specsBuilder.createTag(userUriRaw, label);
          },
          (err) => {
            const msg = err instanceof Error ? err.message : String(err);

            if (invalidChar === " ") {
              // Current implementation uses a slightly different message for whitespace
              assert.ok(
                msg.startsWith("Validation Error: Tag label has"),
                `Unexpected error message for whitespace: "${msg}"`
              );
            } else {
              assert.strictEqual(
                msg,
                `Validation Error: Tag label has invalid char: ${invalidChar}`
              );
            }

            return true;
          },
          `Expected validation error when creating tag with invalid char '${invalidChar}' in label`
        );
      });
    });
  });

  describe("Mute Pubky-app-specs", () => {
    it("should create mute with correct properties", () => {
      const { mute, meta: muteMeta } = specsBuilder.createMute(RIO);

      // Test meta properties
      assert.ok(muteMeta.id, "Mute should have an ID");
      assert.ok(muteMeta.url, "Mute should have a URL");
      const muteChunks = muteMeta.url.split("/")
      assert.strictEqual(muteChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(muteChunks[5], "mutes", "URL should contain mutes path");
      assert.strictEqual(muteChunks[6], muteMeta.id, "URL should contain mute ID");

      // Test mute content
      const muteJson = mute.toJson();
      assert.ok(muteJson.created_at, "Mute should have created_at timestamp");
      assert.ok(typeof muteJson.created_at === "number", "created_at should be a number");
    });
  });

  describe("LastRead Pubky-app-specs", () => {
    it("should create last_read with correct properties", () => {
      const { last_read, meta: lastReadMeta } = specsBuilder.createLastRead(RIO);

      // Test meta properties
      assert.ok(lastReadMeta.url, "LastRead should have a URL");
      const lastReadChunks = lastReadMeta.url.split("/")
      assert.strictEqual(lastReadChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(lastReadChunks[5], "last_read", "URL should contain last_read path");
      assert.strictEqual(lastReadChunks.length, 6, "URL should have 6 segments");

      // Test last_read content
      const lastReadJson = last_read.toJson();
      assert.ok(lastReadJson.timestamp, "LastRead should have timestamp");
      assert.ok(typeof lastReadJson.timestamp === "number", "timestamp should be a number");
    });
  });

  describe("Blob/File Pubky-app-specs", () => {
    it("should create blob with correct properties", () => {
      const length = 8
      const randomData = Array.from({length}, () => Math.floor(Math.random() * 256));
      const { blob, meta: blobMeta } = specsBuilder.createBlob(randomData);

      // Test meta properties
      assert.ok(blobMeta.id, "Blob should have an ID");
      assert.ok(blobMeta.url, "Blob should have a URL");
      const blobChunks = blobMeta.url.split("/")
      assert.strictEqual(blobChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(blobChunks[5], "blobs", "URL should contain blobs path");
      assert.strictEqual(blobChunks[6], blobMeta.id, "URL should contain blob ID");

      // Test blob content
      const blobJson = blob.toJson();
      // Blob JSON is just the raw array data
      assert.ok(Array.isArray(blobJson), "Blob should be an array");
      assert.strictEqual(blobJson.length, length, "Blob data should have correct length");

      // Create a file from the blob
      const { file, meta: fileMeta } = specsBuilder.createFile(
        "Pubky adventures", 
        blobMeta.url, 
        "application/pdf", 
        88
      );

      // Test meta properties
      assert.ok(fileMeta.id, "File should have an ID");
      assert.ok(fileMeta.url, "File should have a URL");
      const fileChunks = fileMeta.url.split("/")
      assert.strictEqual(fileChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(fileChunks[5], "files", "URL should contain files path");
      assert.strictEqual(fileChunks[6], fileMeta.id, "URL should contain file ID");

      // Test file content
      const fileJson = file.toJson();
      assert.strictEqual(fileJson.name, "Pubky adventures", "File name should match");
      assert.strictEqual(fileJson.src, blobMeta.url, "File src should reference blob URL");
      assert.strictEqual(fileJson.content_type, "application/pdf", "File content_type should match");
      assert.strictEqual(fileJson.size, 88, "File size should match");
      assert.ok(fileJson.created_at, "File should have created_at timestamp");
      assert.ok(typeof fileJson.created_at === "number", "created_at should be a number");
    });
  });

  describe("Feed Pubky-app-specs", () => {
    it("should create feed with correct properties", () => {
      const { feed, meta: feedMeta } = specsBuilder.createFeed(
        ["mountain","hike"], 
        "all", 
        "columns", 
        "recent", 
        "image", 
        "nature"
      );

      // Test meta properties
      assert.ok(feedMeta.id, "Feed should have an ID");
      assert.ok(feedMeta.url, "Feed should have a URL");
      assert.ok(feedMeta.url.includes(OTTO), "URL should contain user ID");
      assert.ok(feedMeta.url.includes("feeds"), "URL should contain feeds path");
      assert.ok(feedMeta.url.includes(feedMeta.id), "URL should contain feed ID");

      // Test feed content
      const feedJson = feed.toJson();
      assert.ok(feedJson.feed, "Feed should have feed property");
      assert.ok(Array.isArray(feedJson.feed.tags), "Feed tags should be an array");
      assert.deepStrictEqual(feedJson.feed.tags, ["mountain","hike"], "Feed tags should match");
      assert.strictEqual(feedJson.feed.reach, "all", "Feed reach should match");
      assert.strictEqual(feedJson.feed.layout, "columns", "Feed layout should match");
      assert.strictEqual(feedJson.feed.sort, "recent", "Feed sort should match");
      assert.strictEqual(feedJson.feed.content, "image", "Feed content should match");
      assert.strictEqual(feedJson.name, "nature", "Feed name should match");
      assert.ok(feedJson.created_at, "Feed should have created_at timestamp");
      assert.ok(typeof feedJson.created_at === "number", "created_at should be a number");
    });
  });
});
