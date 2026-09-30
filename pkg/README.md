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

Every rejection is a thrown `Error` whose message starts with `Validation Error: `, the crate's own text for the value the rules refuse, whatever the entry point. Ill-formed UTF-16 (a lone surrogate) is refused: a string argument before it reaches the wasm, with `Validation Error: text must be well-formed UTF-16`, and a string inside an object by the JSON parser, since objects cross as `JSON.stringify` text.

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

The package carries the whole 0.x to 1.x migration: the transforms, compiled into the same wasm, and the engine that walks a tree with them, published as `pubky-social-specs/migration`. The engine uses no browser or Node global, so the same code runs in pubky-app, in a standalone web tool, and under Node for a CLI or a server run. A Node port adapter and a CLI are follow-ups.

```js
import { runMigration } from "pubky-social-specs/migration";

const report = await runMigration({
  owner,
  port, // your adapter over the homeserver client, below
  caps: session.capabilities, // optional, a string or a list of scopes
  lock: (name, fn) => navigator.locks.request(name, { ifAvailable: true }, fn), // optional
  onProgress: (event) => progress.set(event),
  signal: controller.signal,
});
```

`runMigration` calls `init()` itself. It resolves with a report in every case; only a programming error (an owner that is not a pubky, an unknown `mode`, a fault inside the package) rejects.

What a run does, in order:

1. With `caps` given, checks that the session covers `ENGINE_CAPS`, before any request, and aborts with `CAPS_MISSING` otherwise, the scopes to ask for in `error.caps`. A session that holds only the 0.x scope hears about its caps first. The engine never prompts.
2. Probes the private root by a HEAD of `priv/social/v1/_migrated.json`. A homeserver without `/priv/`, where the HEAD throws `unsupported`, aborts the run with `PRIV_UNSUPPORTED` and a message saying so: the private types and the flag live there. Any other failure of the probe aborts with `IO_ERROR`. When the flag records a `transform_rev` equal to or above `transformRev`, the run returns `already_migrated` without listing anything, unless `rescan: true`.
3. Lists `pub/social/v1/` and `priv/social/v1/` and keys every path through `stableId`. That set is the journal: a key present in either root is never written again, so a post unpublished to a draft is not copied back to the public root, and an edit made after an earlier run survives. Media is the exception: it counts as present by its exact URL, since a private copy, or one under another extension, does not serve the public references the run writes. So every blob is read and migrated, and the write it gives is what gets checked.
4. Lists `pub/pubky.app/` and walks it by type: the File objects first, since the run reads them to rewrite media references, then blobs, posts, tags, follows, the profile, feeds, bookmarks and mutes. `settings.json`, `last_read` and anything else no 1.x type takes count as `not_migrated` without a read. A File object that cannot be read stops the run with `IO_ERROR`: every blob and post after it depends on it, and would be copied with a wrong extension or media URL that no later run rewrites. A File that reads but does not parse is only its own skip.
5. For each object whose key is not present: GET, `migrate`, PUT every write whose key is still not present, with `ifAbsent`, then a HEAD of the 0.x object. If the owner deleted it meanwhile, or the HEAD fails, the copies just written are deleted (`deleted_mid_run`, or `io_error` for a copy the next run makes again). A destination someone else wrote since the LIST stays as it is and counts `already_present`; the run deletes only what it wrote. Two objects are in flight at a time, and an object whose write folds to a key the other is writing waits for it, and writes itself if that copy did not land. A blob over `validationLimits.maxFileSizeBytes` skips as `oversize` before it reaches the wasm.
6. Writes the flag `{migrated_at, transform_rev, skipped}`: microseconds, `transformRev`, and the 0.x paths that did not land, by outcome. A walk with any `io_error` writes no flag and ends `incomplete`, so the next run walks again.

The run never modifies the 0.x tree. `mode: "dry"` reads and counts exactly as a run does and makes no PUT, re-check or DELETE, the flag included. An interrupted run leaves nothing to clean up: run it again and it resumes from what the 1.x tree already holds.

The report is `{status, mode, done, total, counts, dropped, droppedValues, skipped, notes, error?}`:

- `status` is `done`, `already_migrated`, `incomplete` (the walk ended but some objects hit `io_error`; run again), `paused` or `aborted`.
- `counts` has one number per outcome: every `skipReasons` entry, `written`, `already_present`, `deleted_mid_run`, `io_error` and `put_rejected`. A File object that reads counts in `done` only: it writes nothing and feeds the run.
- `skipped` maps each outcome but `written` and `already_present` to its 0.x paths, the same object the flag stores; `notes` carries the detail for some of them, such as the status of a refused PUT.
- `dropped` and `droppedValues` report the values the 1.x rules refused inside an object that still migrated (`profile_image`, `profile_link[i]`), so the host can tell the user.
- `error` is `{code, message, needBytes?, caps?}`. `QUOTA` pauses the run with `needBytes`, the sizes the File objects declare for the blobs the walk has not yet copied or found present; free space or raise the quota, then run again. `needBytes` is left out when no blob is pending. `SESSION_EXPIRED`, `IO_ERROR` (a LIST, a File object or the flag failed), `ALREADY_RUNNING`, `UNSUPPORTED_EPOCH` and `ABORTED` (the signal fired) abort it.

`onProgress` gets `{phase, kind?, done, total, counts, dropped, current?, error?}` after every object and at every phase change; `phase` is `probe`, `listing`, `migrating`, `flag`, `done`, `incomplete`, `paused` or `aborted`, and `kind` names the type being walked.

Three constants go with it. `ENGINE_CAPS`, `/pub/social/v1/:rw,/priv/social/v1/:rw`, is what the engine writes and all it checks `caps` for; reading and listing the 0.x tree is anonymous. `MIGRATION_CAPS` is the full grant a migrating pubky-app holds, `ENGINE_CAPS` plus `/priv/app.pubky/v1/:rw,/pub/pubky.app/:rw`: the app's own private namespace, and the 0.x tree, which deleting a migrated object later still reaches. The engine never checks it; it is what the app asks for when it upgrades a session. `transformRev` (from the package entry, and from `pubky-social-specs/migrationData` without the wasm) is the revision of the transforms; it goes up when a transform changes what it writes. A tree recorded under a lower one is walked again, which picks up the objects an earlier revision skipped; a walk never rewrites a destination that exists.

The engine migrates to `social/v1/` and nowhere else, and refuses to run (`UNSUPPORTED_EPOCH`) in a build whose list prefix names another epoch. With one 0.x epoch and one transform step, the source is always `pub/pubky.app/`; discovering the epochs present and sourcing each object from the highest one is the follow-up the next epoch brings.

### The port

All I/O goes through the port, and every URL is a full `pubky://` URL:

```ts
interface MigrationPort {
  list(prefixUrl: string, cursor?: string): Promise<{ urls: string[]; next?: string }>;
  get(url: string): Promise<Uint8Array | null>;
  head(url: string): Promise<boolean>;
  putJson(url: string, object: unknown, options?: { ifAbsent?: boolean }): Promise<void>;
  putBytes(url: string, bytes: Uint8Array, options?: { ifAbsent?: boolean }): Promise<void>;
  delete(url: string): Promise<void>;
}
```

A LIST is deep and ascending: every URL under the prefix, spelled as the prefix is, after `cursor` when given, with `next` the cursor of the following page and absent on the last; a prefix with nothing under it is an empty page. A LIST answering another spelling stops the run with `IO_ERROR`. A missing object is `null` from `get` and `false` from `head`. The engine passes `ifAbsent: true` on every PUT of a copy: the adapter sends `If-None-Match: *` where the homeserver supports conditional PUT and a HEAD before the PUT where it does not, and throws `exists` when something is there. Every failure is a thrown `MigrationPortError(kind, message?, status?)`, and the engine branches on `kind`:

| homeserver answer | `kind` | what the run does |
|---|---|---|
| 507 | `quota` | pauses |
| 429 | `rate_limited` | waits 1 s, doubling up to 60 s, and calls again |
| 401, 403 | `unauthorized` | aborts |
| 404 | `not_found` | a GET counts `deleted_mid_run`; a DELETE ignores it |
| 412 on an `ifAbsent` PUT | `exists` | counts `already_present` |
| 400 or 405 for a `/priv/` path | `unsupported` | aborts on the probe |
| no answer at all, or a 5xx other than 507 | `network` | retries three times, then counts `io_error` |
| any other 4xx, 413 included | `rejected`, with `status` | on a PUT counts `put_rejected`; on a GET or HEAD counts `io_error` |

`rejected` is a definitive refusal and lands in the flag like a skip. A server that failed is not refusing: a 5xx must reach the engine as `network`, so the object counts `io_error`, the run ends `incomplete` without a flag, and the next run writes it. `refusal(status, message?)` builds the error for a status as this table maps it; an adapter throws what it returns, and has to decide `unsupported` itself, since a bare 400 or 405 does not say the root is missing. Anything else a port throws counts as `network`, a call that never got an answer. `MemoryPort` implements the port over a `Map` for tests: `new MemoryPort({ privSupported, intercept, pageSize })`, where `privSupported: false` plays a homeserver without `/priv/`, `intercept(op, url)` runs before every call to fail it or to change `store` under the run, and `pageSize` shortens LIST pages.

The retries wait through `sleep(ms, signal)`, a timer that ends early when `signal` aborts unless the host passes its own.

What the host implements: the adapter from its homeserver client to the port, the lock (without one the run is unlocked; in a browser, `navigator.locks` keeps it to one tab), and the UI over `onProgress` and the report. The rest of the migration is the app's too: the capability upgrade when a run returns `CAPS_MISSING`, importing its own `settings.json` and `last_read` into its private namespace, and keeping the last progress snapshot so it can show where an interrupted run stopped.

### Running it against a homeserver

The tests run the engine over the vectors in a `MemoryPort`. A live run against a local homeserver is manual for now, since it needs a Node port adapter:

1. Start a testnet: `cargo install pubky-testnet --locked` (a version whose homeserver serves `/priv/`), then `pubky-testnet`. The homeserver needs Postgres: point `TEST_PUBKY_CONNECTION_STRING` at one, or install with `--features embedded-postgres` to have it start one in Docker.
2. With the pubky SDK against the testnet, sign up a user granted `ENGINE_CAPS` and the 0.x tree (`MIGRATION_CAPS` covers both) and write a small 0.x tree under `/pub/pubky.app/`: a File object and its blob, a post referencing it, a tag, a follow, a profile.
3. Implement the port over that session following the table above, then `runMigration({ owner, port })` twice: the first run reports every object but the File objects `written`, the second returns `already_migrated`, and `rescan: true` reports what the first run wrote as `already_present` and writes only the flag.

### The transforms on their own

`runMigration` is built on two exports a host can also call directly: `createMigration(owner)` returns a run handle and `migrate(run, path, bytes)` migrates one stored object, by its owner-relative path (`pub/pubky.app/...`) or the full `pubky://` URL a LIST returns. It returns `{writes, dropped}` or `{skip}`. Each write is `{kind, object, meta}`: the object as `readObject` reads it (media as `{bytes}`) and `meta` as a builder gives it, so `validate(meta.url, object)` already holds and the PUT is `kind === "file" ? object.bytes : JSON.stringify(object)` at `meta.url`. A 0.x File object writes nothing: its name, blob and content type feed the run, so every File has to go through `migrate` before the posts, tags and profile that reference them. A reference the run cannot resolve stays as written, since the legacy URI keeps resolving.

`skip` is one of `skipReasons`, frozen data readable before `init()`. A `[DELETED]` post or profile skips as `tombstone`; `not_migrated` is a path with no 1.x counterpart, such as `last_read`, or another owner's path. A blob that skips leaves the references already rewritten to it dangling, so count every skip and report it.

The handle holds the run's memory in the wasm: call `free()` when the run ends (a handle that is garbage collected is freed too, through the glue's `FinalizationRegistry`). It works only with the entry that made it, since the ESM and CommonJS entries hold separate instances.

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

Releases are cut from a git tag, and a build that is not on npm yet can be installed from an `npm pack` tarball. Both are described in [Releasing](https://github.com/pubky/pubky-social-specs#releasing).

## License

MIT
