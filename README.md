# Pubky.app Data Model Specification

_Version 0.6.0_

> ⚠️ **Warning: Rapid Development Phase**  
> This specification is in an **early development phase** and is evolving quickly. Expect frequent changes and updates as the system matures. Consider this a **v0 draft**.
>
> When we reach the first stable, long-term support version of the schemas, paths will adopt the format: `pubky.app/v1/` to indicate compatibility and stability.

### JS package

The package is available as an npm module [pubky-app-specs](https://www.npmjs.com/package/pubky-app-specs). Alternatively, you can build from source using the provided build scripts:

```bash
cd pkg
npm run build
```

Test with:

```bash
cd pkg
npm run install
npm run test
```

Examples with:

```bash
cd pkg
npm run example
```

---

## Table of Contents

- [Pubky.app Data Model Specification](#pubkyapp-data-model-specification)
  - [JS package](#js-package)
  - [Table of Contents](#table-of-contents)
  - [Introduction](#introduction)
  - [Quick Start](#quick-start)
    - [Concepts:](#concepts)
  - [Data Models](#data-models)
    - [PubkyAppUser](#pubkyappuser)
    - [PubkyAppFile](#pubkyappfile)
    - [PubkyAppPost](#pubkyapppost)
    - [PubkyAppTag](#pubkyapptag)
    - [PubkyAppBookmark](#pubkyappbookmark)
    - [PubkyAppFollow](#pubkyappfollow)
    - [PubkyAppFeed](#pubkyappfeed)
  - [Validation Rules](#validation-rules)
    - [Common Rules](#common-rules)
  - [License](#license)

---

## Introduction

This document specifies the data models and validation rules for the **Pubky.app** clients interactions. It defines the structure of data entities, their properties, and the validation rules to ensure data integrity and consistency. This is intended for developers building compatible libraries or clients.

This document intents to be a faithful representation of our [Rust pubky.app models](https://github.com/pubky/pubky-app-specs/tree/main/src). If you intend to develop in Rust, use them directly. In case of disagreement between this document and the Rust implementation, the Rust implementation prevails.

---

## Quick Start

Pubky.app models are designed for decentralized content sharing. The system uses a combination of timestamp-based IDs and Blake3-hashed IDs encoded in Crockford Base32 to ensure unique identifiers for each entity.

### Concepts:

- **Timestamp IDs** for sequential objects like posts and files.
- **Hash IDs** for content-based uniqueness (e.g., tags and bookmarks).
- **Validation Rules** ensure consistent and interoperable data formats.

---

## Data Models

### PubkyAppUser

**Description:** Represents a user's profile information.

**URI:** `/pub/pubky.app/profile.json`

| **Field** | **Type** | **Description**                         | **Validation Rules**                                                                         |
| --------- | -------- | --------------------------------------- | -------------------------------------------------------------------------------------------- |
| `name`    | String   | User's name.                            | Required. Length: 3–50 characters. Cannot be `"[DELETED]"`.                                  |
| `bio`     | String   | Short biography.                        | Optional. Maximum length: 160 characters.                                                    |
| `image`   | String   | URL to the user's profile image.        | Optional. Valid URL. Maximum length: 300 characters.                                         |
| `links`   | Array    | List of associated links (title + URL). | Optional. Maximum of 5 links, each with title (100 chars max) and valid URL (300 chars max). |
| `status`  | String   | User's current status.                  | Optional. Maximum length: 50 characters.                                                     |

**Validation Notes:**

- Reserved keyword `[DELETED]` cannot be used for `name`.
- Each `UserLink` in `links` must have a valid title and URL.

**Example: Valid User**

```json
{
  "name": "Alice",
  "bio": "Toxic maximalist.",
  "image": "pubky://user_id/pub/pubky.app/files/0000000000000",
  "links": [
    {
      "title": "GitHub",
      "url": "https://github.com/alice"
    }
  ],
  "status": "Exploring decentralized tech."
}
```

---

### PubkyAppFile

**Description:** Represents a file uploaded by the user, containing its metadata, including a reference to the actual blob of the file in `src` property.

**URI:** `/pub/pubky.app/files/:file_id`

| **Field**      | **Type** | **Description**             | **Validation Rules**                           |
| -------------- | -------- | --------------------------- | ---------------------------------------------- |
| `name`         | String   | Name of the file.           | Required. Must be 1-255 characters             |
| `created_at`   | Integer  | Unix timestamp of creation. | Required.                                      |
| `src`          | String   | File blob URL               | Required. must be a valid URL. Max length 1024 |
| `content_type` | String   | MIME type of the file.      | Required. Valid IANA mime types                |
| `size`         | Integer  | Size of the file in bytes.  | Required. Positive integer. Max size is 10Mb   |

**Validation Notes:**

- The `file_id` in the URI must be a valid **Timestamp ID**.

---

### PubkyAppPost

**Description:** Represents a user's post.

**URI:** `/pub/pubky.app/posts/:post_id`

| **Field**     | **Type** | **Description**                      | **Validation Rules**                                                       |
| ------------- | -------- | ------------------------------------ | -------------------------------------------------------------------------- |
| `content`     | String   | Content of the post.                 | Required. Max length: 2000 (short), 50000 (long). Cannot be `"[DELETED]"`. |
| `kind`        | String   | Type of post.                        | Required. Must be a valid `PubkyAppPostKind` value.                        |
| `parent`      | String   | URI of the parent post (if a reply). | Optional. Must be a valid URI if present.                                  |
| `embed`       | Object   | Reposted content (type + URI).       | Optional. URI must be valid if present.                                    |
| `attachments` | Array    | List of attachment URIs.             | Optional. Each must be a valid URI.                                        |
| `lock`        | String   | Lock server URL for protected posts. | Optional. If present, must be a valid `pubky://` URL with a host, up to 200 characters. Missing or `null` means unlocked. |

**Post Kinds:**

- `short`
- `long`
- `image`
- `video`
- `link`
- `file`
- `collection`

**Example: Valid Post**

```json
{
  "content": "Hello world! This is my first post.",
  "kind": "short",
  "parent": null,
  "embed": {
    "kind": "short",
    "uri": "pubky://user_id/pub/pubky.app/posts/0000000000000"
  },
  "attachments": ["pubky://user_id/pub/pubky.app/files/0000000000000"],
  "lock": "pubky://lock_server_id/pub/locks/0000000000000"
}
```

**Locking:**

Posts are unlocked by default. A post may include `lock` to advertise that the full post is protected behind a lock server. When present, `lock` must be a valid `pubky://` URL with a host, up to 200 characters. Consumers that receive JSON without `lock`, or JSON with `"lock": null`, must treat the post as a regular unlocked post.

Locking has three layers, kept deliberately separate:

1. **The teaser post**: its `kind` (short/long/collection/video/…) stays the real kind so the post indexes and renders normally (a locked article still shows in article streams, a locked collection on the collections page). Its `lock` field is the `pubky://` URL of layer 2.
2. **The public lock-metadata file**: author-signed public gate metadata (routing, pricing, and the content-hash ID committing to the private bundle). Its shape is owned by the lock-server model and is **out of scope for this crate**.
3. **The private bundle** (`PubkyAppLockedPost`, below): the gated payload the lock server serves after verifying access.

The `lock` field points at layer 2, not the bundle. The bundle's content-hash commitment lives in the author-signed layer 2, which is what lets a client trust an untrusted lock server.

**`PubkyAppLockedPost` (private bundle):**

The full unlocked post plus all of its attachment files, packed into a single content-addressed **binary** blob stored at `/priv/pubky.app/posts/:id`. Files are raw bytes (no base64), so there is no encoding overhead. Container layout:

```text
magic "PALP" (4) | version (1) | manifest_len: u32 LE (4) | manifest JSON | file bytes…
```

The manifest is `{ post, files: [{ name, content_type, size }] }` (`size` is a `u64`); the file bytes follow concatenated in order and are sliced back out by `size`. This crate is the canonical writer; the golden-vector test pins the exact bytes and ID.

- **Identity:** the ID is Crockford-Base32 of the first half of `blake3(bytes)` over the exact stored bytes.
- **Integrity:** the lock server is untrusted. After fetching, a client MUST recompute the ID over the received bytes, check it against the ID committed in the author-signed layer-2 metadata, and run `validate()` (packed post, file count/names/MIME types, total size) before presenting anything.
- **Invariants:** the packed post must not set its own `attachments` (the bundle's `files` are the source of truth) and must not itself carry a `lock`. File count and total size reuse the attachment and blob limits.
- **Unlock:** reconstructing the post's `attachments` from `files` (writing them out and mapping to URIs) is the client/SDK's responsibility, out of scope for this crate.

Build it with `createLockedPost(post, files)` where `files` is an array of `{ name, content_type, data }` (`data` a `Uint8Array`); read it back with `PubkyAppLockedPost.fromBytes(bytes)` and upload bytes via `result.lockedPost.toBytes()`.

**Note on `kind = collection`:**

Collection posts use a typed JSON envelope as their `content`. The envelope shape is:

```json
{
  "name": "AI papers",
  "description": "Best stuff",
  "cover_image": "pubky://userA/pub/pubky.app/files/0034A0X7NJ52C",
  "items": [
    "pubky://userA/pub/pubky.app/posts/0034A0X7NJ52A",
    "pubky://userB/pub/pubky.app/posts/0034A0X7NJ52B"
  ]
}
```

- `name`: required, 1 to 100 unicode scalars, non-whitespace-only.
- `description`: optional, max 500 scalars.
- `cover_image`: optional hero/cover image URL (max 200 chars). Validated as a general attachment URL — protocol must be `pubky`, `http`, or `https`.
- `items`: ordered list of pubky.app post URIs (max 100). Each URI must be in exact canonical form `pubky://<pubky-id>/pub/pubky.app/posts/<post-id>` (94 chars); any deviation (extra path segments, query, fragment, userinfo, etc.) is rejected.

For `kind = collection`, `parent`, `embed`, and `post.attachments` must be unset. The `content` field is bounded by 40000 scalars instead of the regular short/long caps.

---

### PubkyAppTag

**Description:** Represents a tag applied to a URI.

**URI:** `/pub/pubky.app/tags/:tag_id`

| **Field**    | **Type** | **Description**             | **Validation Rules**                                     |
| ------------ | -------- | --------------------------- | -------------------------------------------------------- |
| `uri`        | String   | URI of the tagged object.   | Required. Must be a valid URI.                           |
| `label`      | String   | Label for the tag.          | Required. Trimmed, lowercase. Max length: 20 characters. |
| `created_at` | Integer  | Unix timestamp of creation. | Required.                                                |

**Validation Notes:**

- The `tag_id` is a **Hash ID** derived from the `uri` and `label`.

---

### PubkyAppBookmark

**Description:** Represents a bookmark to a URI.

**URI:** `/pub/pubky.app/bookmarks/:bookmark_id`

| **Field**    | **Type** | **Description**        | **Validation Rules**           |
| ------------ | -------- | ---------------------- | ------------------------------ |
| `uri`        | String   | URI of the bookmark.   | Required. Must be a valid URI. |
| `created_at` | Integer  | Timestamp of creation. | Required.                      |

**Validation Notes:**

- The `bookmark_id` is a **Hash ID** derived from the `uri`.

---

### PubkyAppFollow

**Description:** Represents a follow relationship.

**URI:** `/pub/pubky.app/follows/:user_id`

| **Field**    | **Type** | **Description**        | **Validation Rules** |
| ------------ | -------- | ---------------------- | -------------------- |
| `created_at` | Integer  | Timestamp of creation. | Required.            |

---

### PubkyAppFeed

**Description:** Represents a feed configuration.

**URI:** `/pub/pubky.app/feeds/:feed_id`

| **Field** | **Type** | **Description**                           | **Validation Rules**               |
| --------- | -------- | ----------------------------------------- | ---------------------------------- |
| `tags`    | Array    | List of tags for filtering.               | Optional. Strings must be trimmed. |
| `reach`   | String   | Feed visibility (e.g., `all`, `friends`). | Required. Must be a valid reach.   |
| `layout`  | String   | Feed layout style (e.g., `columns`).      | Required. Must be valid layout.    |
| `sort`    | String   | Sort order (e.g., `recent`).              | Required. Must be valid sort.      |
| `content` | String   | Type of content filtered.                 | Optional.                          |
| `name`    | String   | Name of the feed.                         | Required.                          |

---

## Validation Rules

### Common Rules

1. **Timestamp IDs:** 13-character Crockford Base32 strings derived from timestamps (in microseconds).
2. **Hash IDs:** First half of the bytes from the resulting Blake3-hashed strings encoded in Crockford Base32.
3. **URLs:** All URLs must pass standard validation.

---

## License

This specification is released under the MIT License.
