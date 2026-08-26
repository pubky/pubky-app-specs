# pubky-social-specs

[![npm version](https://img.shields.io/npm/v/pubky-social-specs)](https://www.npmjs.com/package/pubky-social-specs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

JavaScript and TypeScript bindings for Pubky social data models, generated from the canonical Rust crate.

The package initializes WASM automatically, so no manual `.wasm` loading is required.

## Why Use This Package Instead of Manual JSONs?

- **Validation Consistency**: Ensures your app uses the same sanitization and validation rules as [Pubky indexers](https://github.com/pubky/pubky-nexus), avoiding errors.
- **Schema Versioning**: Automatically stay up-to-date with schema changes, reducing maintenance overhead.
- **Auto IDs & Paths**: Generates unique IDs, paths, and URLs according to Pubky standards.
- **Rust-to-JavaScript Compatibility**: Type-safe models that work seamlessly across Rust and JavaScript/TypeScript.
- **Future-Proof**: Easily adapt to new Pubky object types without rewriting JSON manually.

## Installation

```bash
npm install pubky-social-specs
```

```bash
yarn add pubky-social-specs
```

## Quick Start

```js
import { PubkySocialPostKind, PubkySpecsBuilder } from "pubky-social-specs";

const pubkyId = "8kkppkmiubfq4pxn6f73nqrhhhgkb5xyfprntc9si3np9ydbotto";
const specs = new PubkySpecsBuilder(pubkyId);

const { user, meta: userMeta } = specs.createUser(
  "Alice",
  "Building on Pubky",
  null,
  null,
  "active"
);

console.log(userMeta.url); // pubky://.../pub/pubky.app/profile.json
console.log(user.toJson());

const { post, meta: postMeta } = specs.createPost(
  "Hello, Pubky!",
  PubkySocialPostKind.Short
);

console.log(postMeta.url);
console.log(post.toJson());
```

Each create method returns:

- `meta`: generated `id`, storage `path`, and full `url`
- a typed WASM model object with `.toJson()`

## Common Models

```js
const { user, meta } = specs.createUser(name, bio, image, links, status);
const { post, meta } = specs.createPost(content, kind, parent, embed, attachments, lock);
const { file, meta } = specs.createFile(name, src, contentType, size);
const { blob, meta } = specs.createBlob(bytes);
const { bookmark, meta } = specs.createBookmark(uri);
const { tag, meta } = specs.createTag(uri, label);
const { follow, meta } = specs.createFollow(pubkyId);
const { mute, meta } = specs.createMute(pubkyId);
const { feed, meta } = specs.createFeed({
  tags,
  reach,
  layout,
  sort,
  content,
  name,
  domainTags,
  icon,
});
const { last_read, meta } = specs.createLastRead();
```

`domainTags` is optional and can be omitted. `icon` is required and is a [Lucide](https://lucide.dev/icons) icon name (max 50 chars, `a-z`, `0-9`, `-`); legacy feeds may have a missing or `null` icon. Reach accepts `wot` and `me` in addition to `following`, `followers`, `friends`, and `all`.

For runnable examples covering posts, embeds, files, feeds, URI helpers, and MIME type validation, see [`example.js`](https://github.com/pubky/pubky-social-specs/blob/main/pkg/example.js).

## URI Helpers

```js
import {
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
  parse_uri,
} from "pubky-social-specs";

const userUri = userUriBuilder(pubkyId);
const postUri = postUriBuilder(pubkyId, "0033SSE3B1FQ0");
const parsed = parse_uri(postUri);

console.log(parsed.user_id);
console.log(parsed.resource);
console.log(parsed.resource_id);
```

## Validation Limits

Validation limits are published as JSON so apps can reuse canonical limits without initializing WASM.

```js
import limits, {
  getValidationLimits,
  validationLimits,
} from "pubky-social-specs/validationLimits";

console.log(validationLimits.userNameMaxLength);
console.log(limits.postShortContentMaxLength);

const copy = getValidationLimits();
```

For raw JSON imports:

```js
import limitsJson from "pubky-social-specs/validationLimits.json";

console.log(limitsJson.postAttachmentsMaxCount);
```

## MIME Types

```js
import { getValidMimeTypes } from "pubky-social-specs";

const validMimeTypes = getValidMimeTypes();

if (!validMimeTypes.includes(file.type)) {
  throw new Error(`Unsupported file type: ${file.type}`);
}
```

## Specification

The 1.x design is in [`docs/rfc-v1-social-specs.md`](https://github.com/pubky/pubky-social-specs/blob/main/docs/rfc-v1-social-specs.md). The legacy 0.x layout is in [`docs/SPEC_V0.md`](https://github.com/pubky/pubky-social-specs/blob/main/docs/SPEC_V0.md), for reading un-migrated data.

## Building from Source

Prerequisites: Rust, the `wasm32-unknown-unknown` target, [`wasm-pack`](https://rustwasm.github.io/wasm-pack/), and Node.js.

```bash
rustup target add wasm32-unknown-unknown

cd pkg
npm install
npm run build
npm run test
npm run example
```

## License

MIT
