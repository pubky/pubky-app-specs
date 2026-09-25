# pubky-social-specs

[![npm version](https://img.shields.io/npm/v/pubky-social-specs)](https://www.npmjs.com/package/pubky-social-specs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

JavaScript and TypeScript bindings for Pubky social data models, compiled from the canonical Rust crate to WebAssembly.

Every export is a plain function over plain objects. Nothing runs when the package is imported: `await init()` once, and every call after that is synchronous. The ESM (`import`) and CommonJS (`require`) entries each hold their own wasm instance, so each needs its own `init()`.

## Why Use This Package Instead of Manual JSONs?

- **Validation Consistency**: the same validation rules as [Pubky indexers](https://github.com/pubky/pubky-nexus), from the same code. Builders trim and fold what you pass them; readers take stored objects exactly as written and reject what breaks a rule, never rewriting it.
- **Ids, Paths and URLs**: generated the way every other client generates them.
- **Unknown members kept**: an object read and written back keeps the members this version does not know.
- **Typed**: `types.d.ts` declares every object and every function.

## Installation

```bash
npm install pubky-social-specs
```

## Quick Start

```js
import { init, createUser, createPost } from "pubky-social-specs";

await init(); // loads the wasm; every other export throws until it resolves

const owner = "8kkppkmiubfq4pxn6f73nqrhhhgkb5xyfprntc9si3np9ydbotto";

const user = createUser(owner, { name: "Alice", bio: "Building on Pubky" });
console.log(user.meta.url); // pubky://.../pub/social/v1/profile.json
// PUT JSON.stringify(user.object) at user.meta.url with your pubky client

const post = createPost(owner, { content: "Hello, Pubky!" });
console.log(post.meta.path); // /pub/social/v1/posts/{id}/{id}.json
```

Every builder takes the writing user first and returns `{object, meta}`:

- `object` is the stored object exactly as it is written: `JSON.stringify(object)` is the body to PUT. Media is `{bytes: Uint8Array}`.
- `meta` is `{id, path, url}`: the generated id (`""` for the profile), the owner-relative `path`, and the full `pubky://` `url`.

Every rejection is a thrown `Error` carrying the crate's own message, `Validation Error: ...` for a value the rules refuse. Ill-formed UTF-16 (a lone surrogate) is refused: a string argument before it reaches the wasm, with `Validation Error: text must be well-formed UTF-16`, and a string inside an object by the JSON parser, since objects cross as `JSON.stringify` text.

## Builders

```js
createUser(owner, { name, bio?, image?, links?: [{ title, url }], status? });
createPost(owner, { content, kind?, parent?, embed?, attachments?: [{ uri, alt?, name? }], lock?, root? }); // kind defaults to "note"
createArticlePost(owner, { title, body, coverImage?, parent?, embed?, attachments?, lock?, root? });
createCollectionPost(owner, { name, description?, items?: [{ uri, note? }], coverImage?, layout?, root? });
createFeed(owner, { tags?, domainTags?, reach, layout, sort, content?, name, icon });
createTag(owner, uri, label);
createBookmark(owner, target);
createFollow(owner, followee);
createMute(owner, mutee);
createFile(owner, bytes, declaredType, root?); // bytes: Uint8Array
```

`root` is `"public"` (the default) or `"private"`, the same words `parseUri` reports as `visibility`; the path spells them `pub` and `priv`. A post under `"private"` is a draft, and only a draft may reference the owner's private media.

Every optional input member takes `null` or `undefined` for absent. An input member the builder does not know is an error, so a misspelled option never goes missing silently. So is an argument of the wrong type, and an object with no JSON form (one that refers to itself).

`createUser` stores `image` and every `links[].url` as written, so they must already be canonical: an image is a `pubky://`, `http://` or `https://` URI, a link url is `http://` or `https://`, and surrounding whitespace or the short `pubky<pk>` form rejects. The builder trims `name`, `bio`, `status` and every link title; a blank `bio` or `status` is left out. A stored profile is never rewritten on read, padding included, and `validate` does not trim either: it refuses a blank `bio` or `status`, so an edit path maps blank to `null` itself.

`parent` and `embed` are any URI (`pubky://`, `https://`, `nostr:`, `geo:`, ...), stored exactly as written; a thread can be rooted at a post, a user or an external resource. A post reference is always versionless (`.../posts/{id}`, never a version file). `createPost` trims `content`; `readObject` reads it as stored, so content that is only whitespace rejects unless the post has an embed or attachments. The stored post always carries `attachments`, `[]` when empty. The builder trims an attachment `name`; after that it is stored and counted as written.

`createArticlePost` and `createCollectionPost` write their envelope into `content` as JSON: `{title, body, cover_image?}` and `{name, description?, items: [{uri, note?}], cover_image?, layout?}`. A collection item can point anywhere (a post, a user, a web page, a `nostr:` event) and its note is optional but never blank. The builder trims the collection `name`, its `description` and each item `note`, leaving a blank description or note out; a stored description that is empty or whitespace-only rejects. A collection takes no parent, embed or attachments.

`createTag(owner, uri, label)` stores the uri as written, so it must already be canonical; the builder trims and ASCII-lowercases the label.

`createBookmark(owner, target)` puts the target in the filename: `meta.id` is the canonical target in unpadded base64url and the path is `/priv/social/v1/bookmarks/{filename}.json`, so listing every bookmark is one LIST with no GETs, and one target always lands on one filename. A target over 187 UTF-8 bytes overflows: the filename becomes `~{hash}` and the object carries `target`. `bookmarkTarget(filename, object?)` reads an entry back and throws when it breaks those rules, which is how a reader tells an invalid entry from one to show. The object is needed only for a `~` overflow filename, so a primary entry reads from the LIST alone, with no GET. `bookmarkFilename(target)` gives the filename without building an object.

`createFile` stores the bytes as they are: `meta.id` is the hash of the bytes and the path is `files/{hash}.{ext}`, the extension coming from the declared type. The declared type is read once, here, and never stored. Pass `"private"` as `root` for a draft's media. The 0.x File metadata object and its Blob are gone: this one call makes the one media object.

Feeds are private by default: `createFeed` writes under `/priv/`. The id is derived from the filter alone, so `tags` and `domainTags` are folded, deduplicated and sorted by the builder, and editing the filter gives a new id: `feedId(feed)` derives it from an edited object, so the object keeps its unknown members. `icon` is required, a [Lucide](https://lucide.dev/icons) icon name (at most 50 chars of `a-z`, `0-9`, `-`). `feedPaths(id)` gives `{private, public}` and `feedLifecycle(id)` gives `{publish: {from, to}, unpublish: [...], delete: [...]}`: run each in the order given; a delete of a missing path is a skip, and the publish copy always runs because the name and icon live outside the id.

## Reading and Editing

```js
import { readObject, validate } from "pubky-social-specs";

const bytes = new Uint8Array(await (await fetch(url)).arrayBuffer());
const { kind, object } = readObject(url, bytes); // kind: "user" | "post" | ... | "file"

object.status = "away";
validate(url, object); // throws when the edit broke a rule
// PUT JSON.stringify(object) back at url
```

The interfaces in `types.d.ts` list the known members only, so a misspelled field does not compile; unknown members still survive at runtime through read, edit, `validate` and PUT (widen with `& Extra` to reach them). `validate` checks exactly what `JSON.stringify(object)` gives, the bytes a PUT sends. `readObject(uri, bytes)` reads whatever is stored at `uri`, validated against the id, the root and the author the URI names, and returns `{kind, object}`; TypeScript narrows `object` on `kind`. Media comes back as `{kind: "file", object: {bytes}}`. Edit a stored object this way, GET, `readObject`, change the fields, `validate`, PUT: the object keeps every member this version does not know. Rebuilding it through a builder would drop them.

## Posts: Drafts, Versions and the Lifecycle

```js
const version = createVersion(owner, post, { root: "private", slug: "my-draft" }); // {id, editId, path, url}
const edit = editVersion(owner, post, { id: version.id, head: version.editId, root: "private" });

const plan = planPublish(owner, version.id, version.editId, post);
// copy each of plan.mediaCopies ([from, to]) first, then PUT plan.rewrittenPost at plan.destPath

planUnpublish(postId, publicPaths, legacyPaths, privateHeadPath); // {copyBacks, deletes}
planDelete(owner, postId, legacyPaths, [{ root, path }], versions); // {deletes, mediaGcCandidates}
```

`root` defaults to `"public"` in the options too. Editing a post keeps its id and writes a new version above the newest one:

```js
const dir = `/pub/social/v1/posts/${postId}/`;
const newest = (await list(dir)) // LIST, owner-relative paths
  .map((path) => parseUri(`pubky://${owner}${path}`).resource)
  .filter((r) => r.kind === "post" && r.version)
  .map((r) => r.version)
  .sort()
  .at(-1);
const url = `pubky://${owner}${dir}${newest}.json`;
const { object: post } = readObject(url, await get(url));
post.content = "edited";
const next = editVersion(owner, post, { id: postId, head: newest });
validate(next.url, post);
// PUT JSON.stringify(post) at next.url
```

The planners do no I/O: they take the paths the caller listed and return the operations in the order to run them.

`deletionPaths({kind, id, listings})` names every stored copy of one object, legacy first. Every public kind spans the epochs, because on resync the highest understood epoch with a surviving copy wins and a surviving legacy copy would bring the object back: a post across both roots and the legacy epoch, a file across both roots plus the legacy `blobs/` bytes and the v0 File objects the caller lists, a tag plus the v0 tags the caller lists, the profile and a follow plus their legacy path. A feed is its two v1 copies; a mute and a bookmark are their one private path.

A listing is a path, spelled exactly as its epoch writes it, except for the two legacy copies whose path cannot name the object: a v0 File object is `{path, src}`, and it counts only when its stored `src` resolves to this file's bytes; a v0 tag is `{path, uri, label, src?, contentType?}`, and its path must be the 0.x id of its stored `uri` and `label` while that target and label, respelled as v1 writes them, must derive the v1 id being deleted; a v0 tag on a v0 File object also carries that object's `src` and `content_type`, which spell the v1 media file the tag targets. Every entry is tied to the object being deleted; anything else throws, naming the entry.

## URIs

```js
import { parseUri, stableId, resolveDeref, listPrefix, postUriBuilder } from "pubky-social-specs";

parseUri(postUriBuilder(owner, "0033SSE3B1FQ0"));
// { userId, visibility: "public", resource: { kind: "post", id: "0033SSE3B1FQ0" }, path: "/pub/social/v1/posts/0033SSE3B1FQ0" }

stableId("pub/pubky.app/posts/0033SSE3B1FQ0"); // { kind: "key", key: "posts/0033SSE3B1FQ0" }
listPrefix(owner, "private"); // "pubky://.../priv/social/v1/", a LIST prefix, not a URI
legacyListPrefix(owner); // "pubky://.../pub/pubky.app/", the 0.x tree an account delete or export walks
```

`parseUri` reports paths under another namespace, an epoch this version does not speak, and anything else as `foreign`, `unsupportedVersion` and `unknown` kinds; it throws only on a string that is not a canonical `pubky://` URI. `stableId` keys every epoch spelling of one object together, or returns `{kind: "needsDeref", tsid}` for a legacy media reference that `resolveDeref(tsid, src)` completes from its v0 File object.

The URI builders check the owner key and throw on a malformed one. They are `userUriBuilder`, `postUriBuilder`, `followUriBuilder`, `muteUriBuilder`, `bookmarkUriBuilder`, `tagUriBuilder`, `fileUriBuilder` (the whole `{hash}.{ext}` filename) and `feedUriBuilder`.

## MIME Types

```js
import { validMimeTypes, mimeToExt, essence, mimeToExtTable } from "pubky-social-specs";

const accept = validMimeTypes.join(","); // a picker hint only; it gates nothing
mimeToExt("IMAGE/PNG; charset=x"); // "png", and "bin" for anything unmapped
essence("IMAGE/PNG; charset=x"); // "image/png", null when malformed
mimeToExtTable["image/png"]; // "png", from the whole frozen map
```

`validMimeTypes` and `mimeToExtTable` are frozen data, readable before `init()`, and also published without the wasm as `pubky-social-specs/mimeTypes`.

## Validation Limits

`validationLimits` is a frozen plain object, readable before `init()`:

```js
import { validationLimits } from "pubky-social-specs";

validationLimits.userNameMaxLength;
```

The same values are published without the wasm at all. Under Node's ESM loader (and TypeScript's `nodenext`), a JSON import needs its attribute:

```js
import { validationLimits } from "pubky-social-specs/validationLimits";
import limitsJson from "pubky-social-specs/validationLimits.json" with { type: "json" };
```

## Migration

The 0.x to 1.x transforms ship in the same wasm. A run is one handle per owner and one `migrate` call per stored path; the engine around it, LIST, GET, PUT, is yours:

```js
import { createMigration, migrate, legacyListPrefix } from "pubky-social-specs";

const run = createMigration(owner);
const urls = await list(legacyListPrefix(owner)); // full pubky:// URLs, fed in as they come
// The File objects first: they name the blobs and carry the names everything else references
const files = urls.filter((u) => u.includes("/pub/pubky.app/files/"));
const skipped = {};
for (const url of [...files, ...urls.filter((u) => !files.includes(u))]) {
  const result = migrate(run, url, await get(url));
  if ("skip" in result) {
    (skipped[result.skip] ??= []).push(url);
    continue;
  }
  for (const { kind, object, meta } of result.writes) {
    await put(meta.url, kind === "file" ? object.bytes : JSON.stringify(object));
  }
  if (result.dropped.length) console.warn(url, "dropped", result.dropped);
}
run.free();
```

`migrate(run, path, bytes)` takes the owner-relative path (`pub/pubky.app/...`) or the full `pubky://` URL a LIST returns, and returns `{writes, dropped}` or `{skip}`. Each write is `{kind, object, meta}`: the object as `readObject` reads it (media as `{bytes}`) and `meta` as a builder gives it, so the PUT is the same as for anything built here, and `validate(meta.url, object)` already holds. A 0.x File object writes nothing: its name, blob and content type feed the run, so walk `files/` before the posts, tags and profile that reference them. A reference the run cannot resolve stays as written, since the legacy URI keeps resolving.

`skip` is one of `skipReasons`, frozen data readable before `init()` and published without the wasm as `pubky-social-specs/skipReasons`, so a report can count categories without a table of its own. A `[DELETED]` post or profile skips as `tombstone`; `not_migrated` is a path with no 1.x counterpart, such as `last_read`, or another owner's path. A File object is not a skip: it writes nothing and feeds the run. Compare `bytes.length` with `validationLimits.maxFileSizeBytes` before calling `migrate` on a blob: the cap is frozen data, so an `oversize` skip costs no copy into the wasm. A blob that skips leaves the references already rewritten to it dangling (they were rewritten when their post or tag migrated), so count every skip and report it. `dropped` lists `profile_image` and `profile_link[i]`: a value the 1.x gates refuse, left out so the profile still migrates.

The handle holds the run's memory in the wasm: call `free()` when the run ends (a handle that is garbage collected is freed too, through the glue's `FinalizationRegistry`). It works only with the entry that made it, since the ESM and CommonJS entries hold separate instances. What the transforms do not decide stays with the engine: which destinations already exist, the re-check of the source after each PUT, and the order across types.

## Reading 0.x data

The frozen 0.x reader (`legacy_v0`) is Rust only. This package exposes the 1.x surface and the migration; a JS consumer that has to read un-migrated data as such goes through a Rust service.

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
