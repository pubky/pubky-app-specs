# 🦄 Pubky App Specs (WASM) · `pubky-app-specs`

A WASM library for building and validating structured JSON models compatible with Pubky.App social powered by [`@synonymdev/pubky`](https://www.npmjs.com/package/@synonymdev/pubky). It handles domain objects like **Users**, **Posts**, **Feeds**, **Bookmarks**, **Tags**, and more. Each object is:

- **Sanitized** and **Validated** via Rust logic.
- **Auto-ID’ed** and **Auto-Pathed** according to your domain rules.
- **Exported** to JavaScript/TypeScript with minimal overhead.

## 🤔 Why Use This Crate Instead of [Manual JSONs](https://github.com/pubky/pubky-app-specs?tab=readme-ov-file#data-models)?

- **Validation Consistency**: Ensures your app uses the same sanitization and validation rules as [Pubky indexers](https://github.com/pubky/pubky-nexus), avoiding errors.
- **Schema Versioning**: Automatically stay up-to-date with schema changes, reducing maintenance overhead.
- **Auto IDs & Paths**: Generates unique IDs, paths, and URLs according to Pubky standards.
- **Rust-to-JavaScript Compatibility**: Type-safe models that work seamlessly across Rust and JavaScript/TypeScript.
- **Future-Proof**: Easily adapt to new Pubky object types without rewriting JSON manually.

---

## ⚙️ Installation

```bash
npm install pubky-app-specs
# or
yarn add pubky-app-specs
```

> **Note**: This package uses WASM. Ensure your bundler or environment supports loading WASM modules (e.g. Next.js, Vite, etc.).

---

## 🚀 Quick Start

1. **Initialize** the WASM module.
2. **Construct** a `PubkySpecsBuilder(pubkyId)` object.
3. **Create** validated domain objects (User, Post, Tag, etc.).
4. **Store** them on the [PubKy homeserver](https://github.com/synonymdev/pubky) or any distributed storage solution you prefer.

### Import & Initialize

```js
import init, { PubkySpecsBuilder } from "pubky-app-specs";

async function loadSpecs(pubkyId) {
  // 1. Initialize WASM
  await init();

  // 2. Create a specs builder instance
  const specs = new PubkySpecsBuilder(pubkyId);
  return specs;
}
```

---

## 🎨 Example Usage

Below are **succinct** examples to illustrate how to create or update data using **`pubky-app-specs`** and then **store** it with [`@synonymdev/pubky`](https://www.npmjs.com/package/@synonymdev/pubky).

### 1) Creating a New User

```js
import { Client, PublicKey } from "@synonymdev/pubky";
import { PubkySpecsBuilder } from "pubky-app-specs";

async function createUser(pubkyId) {
  const client = new Client();
  const specs = new PubkySpecsBuilder(pubkyId);

  // Create user object with minimal fields
  const userResult = specs.createUser(
    "Alice", // Name
    "Hello from WASM", // Bio
    null, // Image URL or File
    null, // Links
    "active" // Status
  );

  // userResult.meta contains { id, path, url }.
  // userResult.user is the Rust "PubkyAppUser" object.

  // We bring the Rust object to JS using the .toJson() method.
  const userJson = userResult.user.toJson();

  // Store in homeserver via pubky
  const response = await client.fetch(userResult.meta.url, {
    method: "PUT",
    body: JSON.stringify(userJson),
    credentials: "include",
  });

  if (!response.ok) {
    throw new Error(`Failed to store user: ${response.statusText}`);
  }

  console.log("User stored at:", userResult.meta.url);
  return userResult;
}
```

### 2) Creating a Post

```js
import { Client } from "@synonymdev/pubky";
import { PubkySpecsBuilder, PubkyAppPostKind } from "pubky-app-specs";

async function createPost(pubkyId, content) {
  // fileData can be a File (browser) or a raw Blob/Buffer (Node).
  const client = new Client();
  const specs = new PubkySpecsBuilder(pubkyId);

  // Create the Post object referencing your (optional) attachment
  const postResult = specs.createPost(
    content,
    PubkyAppPostKind.Short,
    null, // parent post
    null, // embed
    null // attachments list of urls
  );

  // Store the post
  const postJson = postResult.post.toJson();
  await client.fetch(postResult.meta.url, {
    method: "PUT",
    body: JSON.stringify(postJson),
  });

  console.log("Post stored at:", postResult.meta.url);
  return postResult;
}
```

### 3) Following a User

```js
import { Client } from "@synonymdev/pubky";
import { PubkySpecsBuilder } from "pubky-app-specs";

async function followUser(myPubkyId, userToFollow) {
  const client = new Client();
  const specs = new PubkySpecsBuilder(myPubkyId);

  const followResult = specs.createFollow(userToFollow);

  // We only need to store the JSON in the homeserver
  await client.fetch(followResult.meta.url, {
    method: "PUT",
    body: JSON.stringify(followResult.follow.toJson()),
  });

  console.log(`Successfully followed: ${userToFollow}`);
}
```

---

## 📁 Additional Models

This library supports many more domain objects beyond `User` and `Post`. Here are a few more you can explore:

- **Feeds**: `createFeed(...)`
- **Bookmarks**: `createBookmark(...)`
- **Tags**: `createTag(...)`
- **Mutes**: `createMute(...)`
- **Follows**: `createFollow(...)`
- **LastRead**: `createLastRead(...)`

Each has a `meta` field for storing relevant IDs/paths and a typed data object.

## 📌 Parsing a Pubky URI

The `parse_uri()` function converts a Pubky URI string into a strongly typed object.

**Usage:**

```js
import { parse_uri } from "pubky-app-specs";

try {
  const result = parse_uri("pubky://userID/pub/pubky.app/posts/postID");
  console.log(result.user_id); // "userID"
  console.log(result.resource); // e.g. "posts"
  console.log(result.resource_id); // "postID" or null
} catch (error) {
  console.error("URI parse error:", error);
}
```

**Returns:**

A `ParsedUriResult` object with:

- **user_id:** The parsed user identifier.
- **resource:** A string indicating the resource type.
- **resource_id:** An optional resource identifier.
