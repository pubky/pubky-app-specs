import { PubkySocialPost, PubkySocialPostKind, PubkySpecsBuilder, PubkySocialAttachment, postUriBuilder, bookmarkUriBuilder, followUriBuilder, userUriBuilder, getValidMimeTypes } from "./index.js";
import { createRequire } from "node:module";
import assert from "assert";

const require = createRequire(import.meta.url);
const { validationLimits, getValidationLimits } = require("./validationLimits.cjs");
const validationLimitsJson = require("./validationLimits.json");

const OTTO = "8kkppkmiubfq4pxn6f73nqrhhhgkb5xyfprntc9si3np9ydbotto";
const RIO = "dzswkfy7ek3bqnoc89jxuqqfbzhjrj6mi8qthgbxxcqkdugm3rio";

describe("PubkySpecs Example Objects Tests", () => {
  let specsBuilder;

  beforeEach(() => {
    specsBuilder = new PubkySpecsBuilder(OTTO);
  });

  describe("User Pubky-social-specs", () => {
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

    it("cannot create user with name too short", () => {
      assert.throws(
        () => {
          specsBuilder.createUser("AB", null, null, null, null); // 2 chars, min is 3
        },
        (err) => {
          const msg = err instanceof Error ? err.message : String(err);
          assert.ok(
            msg.includes("Invalid name length"),
            `Expected 'Invalid name length' error, got: "${msg}"`
          );
          return true;
        },
        "Expected validation error for name too short"
      );
    });

    it("should accept emoji name at max length (50 chars)", () => {
      const emojiName = "🔥".repeat(50); // 50 emoji = 50 Unicode chars (but many more bytes)
      assert.strictEqual([...emojiName].length, 50, "Should be 50 Unicode characters");

      const { user } = specsBuilder.createUser(emojiName, null, null, null, null);
      const userJson = user.toJson();
      assert.strictEqual(userJson.name, emojiName, "Emoji name should be preserved");
    });
  });

  describe("Post Pubky-social-specs", () => {
    it("should create basic post with correct properties", () => {
      const postContent = "Hello, Pubky world! This is my first post."
      const { post, meta } = specsBuilder.createPost(postContent, PubkySocialPostKind.Note);

      // Test meta properties
      assert.ok(meta.id, "Post should have an ID");
      assert.ok(meta.url, "Post should have a URL");
      const postChunks = meta.url.split("/")
      assert.strictEqual(postChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(postChunks[6], "posts", "URL should contain posts path");
      assert.strictEqual(postChunks[7], meta.id, "URL should contain post ID");
      assert.strictEqual(meta.path, "/pub/social/v1/posts/" + meta.id + "/" + meta.id + ".json", "post storage path");

      // Test post content
      const postJson = post.toJson();
      assert.strictEqual(postJson.content, postContent, "Post content should match");
      assert.strictEqual(postJson.kind, "note", "Post kind should match");
    });

    it("should create reply post with parent reference", () => {
      const parentPostUriRaw = `pubky://${RIO}/pub/social/v1/posts/0033SSE3B1FQ0`
      const parentPostUri = postUriBuilder(RIO, "0033SSE3B1FQ0")
      assert.strictEqual(parentPostUri, parentPostUriRaw, "Parent post URI should match");

      const { post: replyPost } = specsBuilder.createPost(
        "This is a reply to the first post!",
        PubkySocialPostKind.Note,
        parentPostUriRaw
      );

      // Test reply content
      const replyJson = replyPost.toJson();
      assert.strictEqual(replyJson.parent, parentPostUriRaw, "Reply should reference parent URL");
    });

    it("should create repost with embed", () => {
      const embedUriRaw = `pubky://${RIO}/pub/social/v1/posts/0033SREKPC4N0`
      const embedUriFromBuilder = postUriBuilder(RIO, "0033SREKPC4N0")
      assert.strictEqual(embedUriFromBuilder, embedUriRaw, "Embed URI should match");

      const { post: repost } = specsBuilder.createPost(
        "This is a repost to random post!",
        PubkySocialPostKind.Note,
        null,
        embedUriRaw
      );

      // Test repost content
      const repostJson = repost.toJson();
      assert.strictEqual(repostJson.embed, embedUriRaw, "Embed URI should match");
    });

    it("should carry attachment objects with alt and name", () => {
      const uri = `pubky://${OTTO}/pub/social/v1/files/0034A0X7NJ52G`;
      const attachment = new PubkySocialAttachment(uri, "a cat", "  cat.jpg  ");
      assert.strictEqual(attachment.uri, uri);
      assert.strictEqual(attachment.alt, "a cat");
      assert.strictEqual(attachment.name, "cat.jpg", "name should be trimmed");

      const { post } = specsBuilder.createPost("", PubkySocialPostKind.Image, null, null, [attachment]);
      const postJson = post.toJson();
      assert.deepStrictEqual(postJson.attachments, [{ uri, alt: "a cat", name: "cat.jpg" }]);
      assert.strictEqual(post.attachments[0].alt, "a cat", "getter should return attachment instances");
    });

    it("toJson returns a plain object and keeps unknown members", () => {
      const post = PubkySocialPost.fromJson({
        content: "x",
        kind: "note",
        parent: null,
        embed: null,
        attachments: [],
        later: 1,
      });
      const postJson = post.toJson();
      assert.strictEqual(Object.getPrototypeOf(postJson), Object.prototype, "toJson must not return a Map");
      assert.strictEqual(postJson.kind, "note");
      assert.strictEqual(postJson.later, 1, "unknown members survive a round trip");
    });

    it("cannot create post with too many attachments", () => {
      const attachments = [
        `pubky://${OTTO}/pub/social/v1/files/0034A0X7NJ52G`,
        `pubky://${OTTO}/pub/social/v1/files/0034A0X7NJ53H`,
        `pubky://${OTTO}/pub/social/v1/files/0034A0X7NJ54I`,
        `pubky://${OTTO}/pub/social/v1/files/0034A0X7NJ55J`,
        `pubky://${OTTO}/pub/social/v1/files/0034A0X7NJ55K`,
        `pubky://${OTTO}/pub/social/v1/files/0034A0X7NJ55L`,
        `pubky://${OTTO}/pub/social/v1/files/0034A0X7NJ55M`,
        `pubky://${OTTO}/pub/social/v1/files/0034A0X7NJ55N`,
        `pubky://${OTTO}/pub/social/v1/files/0034A0X7NJ55O`,
        `pubky://${OTTO}/pub/social/v1/files/0034A0X7NJ55P`,
        `pubky://${OTTO}/pub/social/v1/files/0034A0X7NJ55A`, // 11th attachment exceeds limit
      ].map((uri) => new PubkySocialAttachment(uri, null, null));

      assert.throws(
        () => {
          specsBuilder.createPost(
            "Post with too many attachments",
            PubkySocialPostKind.Image,
            null,
            null,
            attachments
          );
        },
        (err) => {
          const msg = err instanceof Error ? err.message : String(err);
          assert.ok(
            msg.includes("Too many attachments"),
            `Expected 'Too many attachments' error, got: "${msg}"`
          );
          return true;
        },
        "Expected validation error for too many attachments"
      );
    });

    describe("Post lock", () => {
      const validLockUrl = `pubky://${RIO}/pub/locks/0034A0X7NJ52G`;

      it("should create locked post with valid pubky lock URL", () => {
        const postContent = "Visible preview for locked content";
        const { post } = specsBuilder.createPost(
          postContent,
          PubkySocialPostKind.Note,
          null,
          null,
          null,
          validLockUrl
        );

        assert.strictEqual(post.lock, validLockUrl, "lock getter should return lock URL");
        const postJson = post.toJson();
        assert.strictEqual(postJson.content, postContent, "Post content should match");
        assert.strictEqual(postJson.lock, validLockUrl, "toJson should include lock URL");
      });

      it("should create unlocked post when lock is omitted", () => {
        const { post } = specsBuilder.createPost("Hello", PubkySocialPostKind.Note);
        const lock = post.lock;
        assert.ok(
          lock === null || lock === undefined,
          "createPost should produce unlocked post"
        );
        const postJson = post.toJson();
        assert.ok(
          postJson.lock === null || postJson.lock === undefined,
          "toJson should not include lock for unlocked post"
        );
      });

      it("should deserialize post without lock field as unlocked", () => {
        const post = PubkySocialPost.fromJson({
          content: "Hello World!",
          kind: "note",
          parent: null,
          embed: null,
        });
        const lock = post.lock;
        assert.ok(
          lock === null || lock === undefined,
          "fromJson without lock should deserialize as unlocked"
        );
      });

      it("cannot create post with non-pubky lock URL", () => {
        assert.throws(
          () => {
            specsBuilder.createPost(
              "Preview",
              PubkySocialPostKind.Note,
              null,
              null,
              null,
              "https://locks.example.com/session/0034A0X7NJ52G"
            );
          },
          (err) => {
            const msg = err instanceof Error ? err.message : String(err);
            assert.ok(
              msg.includes("lock"),
              `Expected lock error, got: "${msg}"`
            );
            return true;
          },
          "Expected validation error for non-pubky lock URL"
        );
      });

      it("cannot create post with hostless lock URL", () => {
        assert.throws(
          () => {
            specsBuilder.createPost(
              "Preview",
              PubkySocialPostKind.Note,
              null,
              null,
              null,
              "pubky:lock-id"
            );
          },
          (err) => {
            const msg = err instanceof Error ? err.message : String(err);
            assert.ok(msg.includes("lock"), `Expected lock error, got: "${msg}"`);
            return true;
          },
          "Expected validation error for hostless lock URL"
        );
      });
    });

    describe("Article posts", () => {
      it("should create article post with JSON envelope content", () => {
        const { post, meta } = specsBuilder.createArticlePost(
          "  On Pubky  ",
          "# Hello\n\nbody",
          `pubky://${RIO}/pub/social/v1/files/0034A0X7NJ52G`
        );
        assert.ok(meta.id, "Article post should have an ID");
        const postJson = post.toJson();
        assert.strictEqual(postJson.kind, "article", "Post kind should be article");
        assert.deepStrictEqual(postJson.attachments, []);
        const envelope = JSON.parse(postJson.content);
        assert.strictEqual(envelope.title, "On Pubky", "Title should be trimmed");
        assert.strictEqual(envelope.body, "# Hello\n\nbody");
        assert.strictEqual(envelope.cover_image, `pubky://${RIO}/pub/social/v1/files/0034A0X7NJ52G`);
      });

      it("should accept parent, embed, attachments and lock on an article", () => {
        const { post } = specsBuilder.createArticlePost(
          "Reply article",
          "body",
          null,
          `pubky://${RIO}/pub/social/v1/posts/0033SSE3B1FQ0`,
          "https://example.com/source",
          [new PubkySocialAttachment(`pubky://${RIO}/pub/social/v1/files/0034A0X7NJ52G`, "alt", "a.jpg")],
          `pubky://${RIO}/pub/app.locks/0034A0X7NJ52G.json`
        );
        const postJson = post.toJson();
        assert.strictEqual(postJson.parent, `pubky://${RIO}/pub/social/v1/posts/0033SSE3B1FQ0`);
        assert.strictEqual(postJson.embed, "https://example.com/source");
        assert.strictEqual(postJson.attachments.length, 1);
        assert.ok(postJson.lock);
      });

      it("cannot create article with empty title", () => {
        assert.throws(
          () => specsBuilder.createArticlePost("   ", "body", null),
          (err) => {
            const msg = err instanceof Error ? err.message : String(err);
            assert.ok(msg.includes("title"), `Expected title error, got: "${msg}"`);
            return true;
          }
        );
      });
    });

    describe("Collection posts", () => {
      const collectionItemUri = `pubky://${RIO}/pub/social/v1/posts/0033SREKPC4N0`;
      const coverImageUrl = `pubky://${RIO}/pub/social/v1/files/0034A0X7NJ52G`;

      it("should create collection post with JSON envelope content", () => {
        assert.strictEqual(
          typeof specsBuilder.createCollectionPost,
          "function",
          "PubkySpecsBuilder should expose createCollectionPost"
        );

        const { post, meta } = specsBuilder.createCollectionPost(
          "Favorite posts",
          "Posts worth revisiting",
          [collectionItemUri],
          coverImageUrl
        );

        assert.ok(meta.id, "Collection post should have an ID");
        assert.ok(meta.url, "Collection post should have a URL");
        const postChunks = meta.url.split("/");
        assert.strictEqual(postChunks[2], OTTO, "URL should contain user ID");
        assert.strictEqual(postChunks[6], "posts", "URL should contain posts path");
        assert.strictEqual(postChunks[7], meta.id, "URL should contain post ID");
      assert.strictEqual(meta.path, "/pub/social/v1/posts/" + meta.id + "/" + meta.id + ".json", "post storage path");

        const postJson = post.toJson();
        assert.strictEqual(postJson.kind, "collection", "Post kind should be collection");
        assert.deepStrictEqual(postJson.attachments, [], "Collection items should not be stored in post.attachments");

        const envelope = JSON.parse(postJson.content);
        assert.strictEqual(envelope.name, "Favorite posts", "Collection name should match");
        assert.strictEqual(
          envelope.description,
          "Posts worth revisiting",
          "Collection description should match"
        );
        assert.deepStrictEqual(envelope.items, [collectionItemUri], "Collection items should match");
        assert.strictEqual(envelope.cover_image, coverImageUrl, "Collection cover image should match");
      });

      it("cannot create collection post with too many items", () => {
        assert.strictEqual(
          typeof specsBuilder.createCollectionPost,
          "function",
          "PubkySpecsBuilder should expose createCollectionPost"
        );

        const tooManyItems = Array.from(
          { length: validationLimits.collectionItemsMaxCount + 1 },
          (_, index) => `pubky://${RIO}/pub/social/v1/posts/${String(index).padStart(13, "0")}`
        );

        assert.throws(
          () => {
            specsBuilder.createCollectionPost("Too many", null, tooManyItems, null);
          },
          (err) => {
            const msg = err instanceof Error ? err.message : String(err);
            assert.ok(
              msg.includes(`${validationLimits.collectionItemsMaxCount} items`),
              `Expected collection item limit error, got: "${msg}"`
            );
            return true;
          },
          "Expected validation error for too many collection items"
        );
      });
    });
  });

  describe("Bookmark Pubky-social-specs", () => {
    it("should create bookmark with correct properties", () => {
      const postUriRaw = `pubky://${RIO}/pub/social/v1/posts/0033SREKPC4N0`

      const { bookmark, meta: bookmarkMeta } = specsBuilder.createBookmark(postUriRaw);
      const bookmarkUriFromBuilder = bookmarkUriBuilder(OTTO, bookmarkMeta.id)
      assert.strictEqual(bookmarkUriFromBuilder, bookmarkMeta.url, "Bookmark URI should match");

      // Test meta properties
      assert.ok(bookmarkMeta.id, "Bookmark should have an ID");
      assert.ok(bookmarkMeta.url, "Bookmark should have a URL");
      const bookmarkChunks = bookmarkMeta.url.split("/")
      assert.strictEqual(bookmarkChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(bookmarkChunks[6], "bookmarks", "URL should contain bookmarks path");
      assert.strictEqual(bookmarkChunks[7], bookmarkMeta.id + ".json", "URL should contain bookmark ID");

      // Test bookmark content
      const bookmarkJson = bookmark.toJson();
      assert.strictEqual(bookmarkJson.uri, postUriRaw, "Bookmark URI should match");
      assert.ok(bookmarkJson.created_at, "Bookmark should have created_at timestamp");
      assert.ok(typeof bookmarkJson.created_at === "number", "created_at should be a number");
    });
  });

  describe("Follow Pubky-social-specs", () => {
    it("should create follow with correct properties", () => {
      const { follow, meta: followMeta } = specsBuilder.createFollow(RIO);
      const followUriFromBuilder = followUriBuilder(OTTO, RIO)
      assert.strictEqual(followUriFromBuilder, followMeta.url, "Follow URI should match");

      // Test meta properties
      assert.strictEqual(followMeta.id, RIO, "Follow ID should be the user being followed");
      assert.ok(followMeta.url, "Follow should have a URL");
      const followChunks = followMeta.url.split("/")
      assert.strictEqual(followChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(followChunks[6], "follows", "URL should contain follows path");
      assert.strictEqual(followChunks[7], RIO + ".json", "URL should contain follow ID");

      // Test follow content
      const followJson = follow.toJson();
      assert.ok(followJson.created_at, "Follow should have created_at timestamp");
      assert.ok(typeof followJson.created_at === "number", "created_at should be a number");
    });
  });

  describe("Tag Pubky-social-specs", () => {
    it("should create tag with correct properties", () => {
      const userUriRaw = `pubky://${OTTO}/pub/social/v1/profile.json`;
      const userUriFromBuilder = userUriBuilder(OTTO)
      assert.strictEqual(userUriFromBuilder, userUriRaw, "User URI should match");

      const { tag, meta: tagMeta } = specsBuilder.createTag(userUriRaw, "otto");

      // Test meta properties
      assert.ok(tagMeta.id, "Tag should have an ID");
      assert.ok(tagMeta.url, "Tag should have a URL");
      const tagChunks = tagMeta.url.split("/")
      assert.strictEqual(tagChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(tagChunks[6], "tags", "URL should contain tags path");
      assert.strictEqual(tagChunks[7], tagMeta.id + ".json", "URL should contain tag ID");

      // Test tag content
      const tagJson = tag.toJson();
      assert.strictEqual(tagJson.uri, userUriRaw, "Tag URI should match");
      assert.strictEqual(tagJson.label, "otto", "Tag label should match");
      assert.ok(tagJson.created_at, "Tag should have created_at timestamp");
      assert.ok(typeof tagJson.created_at === "number", "created_at should be a number");
    });
    it("cannot create a tag with invalid characters (comma, colon, space)", () => {
      const userUriRaw = `pubky://${OTTO}/pub/social/v1/profile.json`;
      const userUriFromBuilder = userUriBuilder(OTTO);
      assert.strictEqual(userUriFromBuilder, userUriRaw, "User URI should match");

      const invalidCases = [
        { label: "otto,rio", invalidChar: ",", isWhitespace: false },
        { label: "otto:rio", invalidChar: ":", isWhitespace: false },
        { label: "otto rio", invalidChar: " ", isWhitespace: true },
      ];

      invalidCases.forEach(({ label, invalidChar, isWhitespace }) => {
        assert.throws(
          () => {
            specsBuilder.createTag(userUriRaw, label);
          },
          (err) => {
            const msg = err instanceof Error ? err.message : String(err);

            if (isWhitespace) {
              // Whitespace has a different error message format
              assert.strictEqual(
                msg,
                `Validation Error: Tag '${label}' contains whitespace characters`,
                `Unexpected error message for whitespace: "${msg}"`
              );
            } else {
              assert.strictEqual(
                msg,
                `Validation Error: Tag '${label}' contains invalid character: ${invalidChar}`,
                `Unexpected error message for invalid char '${invalidChar}': "${msg}"`
              );
            }

            return true;
          },
          `Expected validation error when creating tag with invalid char '${invalidChar}' in label`
        );
      });
    });
  });

  describe("Mute Pubky-social-specs", () => {
    it("should create mute with correct properties", () => {
      const { mute, meta: muteMeta } = specsBuilder.createMute(RIO);

      // Test meta properties
      assert.ok(muteMeta.id, "Mute should have an ID");
      assert.ok(muteMeta.url, "Mute should have a URL");
      const muteChunks = muteMeta.url.split("/")
      assert.strictEqual(muteChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(muteChunks[6], "mutes", "URL should contain mutes path");
      assert.strictEqual(muteChunks[7], muteMeta.id + ".json", "URL should contain mute ID");

      // Test mute content
      const muteJson = mute.toJson();
      assert.ok(muteJson.created_at, "Mute should have created_at timestamp");
      assert.ok(typeof muteJson.created_at === "number", "created_at should be a number");
    });
  });

  describe("Blob/File Pubky-social-specs", () => {
    it("should create blob with correct properties", () => {
      const length = 8
      const randomData = Array.from({length}, () => Math.floor(Math.random() * 256));
      const { blob, meta: blobMeta } = specsBuilder.createBlob(randomData);

      // Test meta properties
      assert.ok(blobMeta.id, "Blob should have an ID");
      assert.ok(blobMeta.url, "Blob should have a URL");
      const blobChunks = blobMeta.url.split("/")
      assert.strictEqual(blobChunks[2], OTTO, "URL should contain user ID");
      assert.strictEqual(blobChunks[6], "blobs", "URL should contain blobs path");
      assert.strictEqual(blobChunks[7], blobMeta.id, "URL should contain blob ID");

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
      assert.strictEqual(fileChunks[6], "files", "URL should contain files path");
      assert.strictEqual(fileChunks[7], fileMeta.id + ".json", "URL should contain file ID");

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

  describe("Feed Pubky-social-specs", () => {
    it("should create feed with correct properties", () => {
      const { feed, meta: feedMeta } = specsBuilder.createFeed({
        tags: ["mountain", "hike"],
        reach: "all",
        layout: "columns",
        sort: "recent",
        content: "image",
        name: "nature",
        icon: "mountain",
      });

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
      assert.strictEqual(feedJson.icon, "mountain", "Feed icon should match");
      assert.ok(feedJson.created_at, "Feed should have created_at timestamp");
      assert.ok(typeof feedJson.created_at === "number", "created_at should be a number");
    });

    it("should create feed with wot reach and domain_tags", () => {
      const { feed } = specsBuilder.createFeed({
        tags: ["rust"],
        reach: "wot",
        layout: "columns",
        sort: "recent",
        content: "image",
        name: "WoT Feed",
        domainTags: ["synonym"],
        icon: "users",
      });

      const feedJson = feed.toJson();
      assert.strictEqual(feedJson.feed.reach, "wot", "Feed reach should be wot");
      assert.deepStrictEqual(
        feedJson.feed.domain_tags,
        ["synonym"],
        "Feed domain_tags should match"
      );
    });

    it("should create feed with me reach without domain_tags", () => {
      const { feed } = specsBuilder.createFeed({
        reach: "me",
        layout: "list",
        sort: "popularity",
        name: "My Posts",
        icon: "user",
      });

      const feedJson = feed.toJson();
      assert.strictEqual(feedJson.feed.reach, "me", "Feed reach should be me");
      assert.ok(
        feedJson.feed.domain_tags == null,
        "Feed domain_tags should be absent or null when not provided"
      );
      assert.strictEqual(feedJson.icon, "user", "Feed icon should match");
    });

    it("should reject icons outside a-z, 0-9 and -", () => {
      for (const icon of ["bad icon", "bad_icon"]) {
        assert.throws(() =>
          specsBuilder.createFeed({
            reach: "all",
            layout: "columns",
            sort: "recent",
            name: "Bad Icon",
            icon,
          })
        );
      }
    });

    it("should require an icon when creating a feed", () => {
      assert.throws(() =>
        specsBuilder.createFeed({
          reach: "all",
          layout: "columns",
          sort: "recent",
          name: "Missing Icon",
        })
      );
    });
  });

  describe("Valid MIME Types", () => {
    it("should return an array of valid MIME types", () => {
      const mimeTypes = getValidMimeTypes();
      
      assert.ok(Array.isArray(mimeTypes), "Should return an array");
      assert.ok(mimeTypes.length > 0, "Should have at least one MIME type");
    });

    it("should include common image MIME types", () => {
      const mimeTypes = getValidMimeTypes();
      
      assert.ok(mimeTypes.includes("image/png"), "Should include image/png");
      assert.ok(mimeTypes.includes("image/jpeg"), "Should include image/jpeg");
      assert.ok(mimeTypes.includes("image/gif"), "Should include image/gif");
      assert.ok(mimeTypes.includes("image/webp"), "Should include image/webp");
    });

    it("should include common video MIME types", () => {
      const mimeTypes = getValidMimeTypes();
      
      assert.ok(mimeTypes.includes("video/mp4"), "Should include video/mp4");
      assert.ok(mimeTypes.includes("video/mpeg"), "Should include video/mpeg");
    });

    it("should include common document MIME types", () => {
      const mimeTypes = getValidMimeTypes();
      
      assert.ok(mimeTypes.includes("application/pdf"), "Should include application/pdf");
      assert.ok(mimeTypes.includes("application/json"), "Should include application/json");
      assert.ok(mimeTypes.includes("text/plain"), "Should include text/plain");
    });

    it("should be usable for file validation before upload", () => {
      const mimeTypes = getValidMimeTypes();
      
      // Valid file types
      assert.ok(mimeTypes.includes("image/png"), "image/png should be valid");
      assert.ok(mimeTypes.includes("application/pdf"), "application/pdf should be valid");
      
      // Invalid file types (not in the list)
      assert.ok(!mimeTypes.includes("application/x-executable"), "application/x-executable should not be valid");
      assert.ok(!mimeTypes.includes("application/x-msdownload"), "application/x-msdownload should not be valid");
    });

    it("should create file with valid MIME type from the list", () => {
      const mimeTypes = getValidMimeTypes();
      const validMimeType = mimeTypes[0]; // Pick the first valid MIME type
      
      const { blob, meta: blobMeta } = specsBuilder.createBlob([1, 2]);
      assert.strictEqual(blobMeta.id, "PZBQ010FF079VVZPQG1RNFN6DR", "blake3 known answer for [1, 2]");
      const { file } = specsBuilder.createFile(
        "test-file",
        blobMeta.url,
        validMimeType,
        100
      );
      
      const fileJson = file.toJson();
      assert.strictEqual(fileJson.content_type, validMimeType, "File should have valid MIME type");
    });
  });

  describe("Validation limits exports", () => {
    it("should expose validationLimits from JS exports", () => {
      assert.ok(validationLimits, "validationLimits should be defined");
      assert.deepStrictEqual(
        validationLimits,
        validationLimitsJson,
        "validationLimits should match validationLimits.json"
      );
      assert.strictEqual(
        validationLimits.userNameMinLength,
        3,
        "userNameMinLength should match the Rust limits"
      );
      assert.ok(
        Array.isArray(validationLimits.tagInvalidChars),
        "tagInvalidChars should be an array"
      );
    });

    it("getValidationLimits should return a copy that matches validationLimits", () => {
      const limitsCopy = getValidationLimits();

      assert.deepStrictEqual(
        limitsCopy,
        validationLimits,
        "getValidationLimits should match validationLimits"
      );
      assert.notStrictEqual(
        limitsCopy,
        validationLimits,
        "getValidationLimits should return a new object"
      );
    });

    it("builder.validationLimits should match the JS exports", () => {
      const builderLimits = JSON.parse(JSON.stringify(specsBuilder.validationLimits));

      assert.deepStrictEqual(
        builderLimits,
        validationLimits,
        "builder.validationLimits should match exported validationLimits"
      );
    });
  });
});
