# RFC: pubky-social-specs v1 (first stable, first breaking release)

> Status: DRAFT, for review. This document is the complete v1 design and rollout plan: the
> design model by model (v0 shape, v1 shape, why), then migration, then the rollout. Comment
> inline on the line you disagree with. The companion `v0-vs-v1.md` expands every model change
> with its full reasoning.

The first stable and first breaking release of the shared social-data layer. Renames the crate
`pubky-app-specs` to `pubky-social-specs` (`1.0.0`) and moves all data from the single hard-coded
app path `/pub/pubky.app/<res>` to a versioned, app-neutral epoch `/{pub|priv}/social/v1/<res>`.
One coordinated break spends the whole budget on what cannot be added additively later; a
permanent forward-compat contract is designed so v1.x grows additively. A further epoch stays
reserved for the changes no contract can make additive: re-pinning a text or id function (which
re-ids data), changing an existing resource's root or per-kind content semantics, or breaking
the path grammar.

# Part A: Why break now

- `pub/pubky.app/` hard-codes one app's domain as the home of shared data and the parser rejects
  every other app. A shared spec must classify foreign data, not error on it.
- The path is the only version signal that survives the `/events/` feed, LIST, and anonymous GET.
  v0 has none, so v1.x could not evolve without breaking old clients.
- No privacy tier, no file extensions, GET-per-file bookmarks, and `url::Url` validation that
  normalizes junk into acceptance while rejecting valid short-form URIs.

# Part B: Design, model by model

## B0. Cross-cutting (applies to every model)

- **Namespace + epoch: `pub/pubky.app/<res>` becomes `{pub|priv}/social/v1/<res>`.** App-neutral
  (`pubky.app` wrongly signals one app owns shared data), versioned (the path is the only channel
  that survives events + LIST + anonymous GET), and a bare word with no dot (a dotted directory
  name like `pubky.app/` reads as an application bundle on macOS when a tree is exported to disk).
- **Folder ownership (the composition law).** The specification that defines an object
  determines its storage namespace, never the application that writes it. An app writing social
  objects writes them under `social/vN`; its own objects live under its own namespace. Apps
  therefore compose spec packages, each bringing its capability scopes and path builders (request
  `/pub/social/v1/:rw` alongside your app scopes; use each package's builders for its paths).
  Consequence: tags are social objects, so the one canonical v1 write location is
  `pub/social/v1/tags/`; indexers reading `tags/` directories inside other app namespaces is a
  legacy read rule, not a v1 write model.
- **App namespaces SHOULD carry an epoch too** (`pub/<app>/v1/...`). Nothing can be mandated for
  foreign namespaces (no enforcement point exists), but the recommendation is free and buys an
  app the same migration mechanics this spec built for itself: old and new data coexist in
  disjoint subtrees, and the path is the only version signal that survives the events feed,
  LIST, and anonymous GET. The parser already anticipates this: `Foreign` classification
  extracts the version whenever the segment after the namespace matches `v[0-9]+`, so indexers
  version-route conforming app data with zero extra work.
- **Namespace governance.** `social/vN` is owned by this repo: a resource type exists exactly
  when the released crate parses it, and additions land as ordinary crate-minor PRs here (parser
  arm + model + data assets + vectors in one change). Reserved: epoch segments `v[0-9]+`, the `_`
  filename prefix, every current resource segment, and the `ext` member name; unknown segments
  under `social/vN` parse as a handled `Unknown`, so additions never break deployed readers. A
  new epoch (`social/v2`) is reserved for changes impossible additively (re-pinned text/id
  functions, changed root or content semantics, grammar breaks). App namespaces are self-assigned
  (domain-style names recommended); the parser classifies them foreign, never invalid.
- **`.json` on every JSON leaf.** The homeserver derives the served type from magic bytes then the
  path extension; extensionless JSON serves as octet-stream/plaintext.
- **A privacy tier `/priv/` (owner-only, excluded from `/events/`).** For state whose only reader
  is the owner's own client. The placement test is the actual reader set, not aspiration.
- **Forward-compat contract (permanent).** All wire enums are plain string enums (unit
  variants, `rename_all` lowercase/snake_case), so `#[serde(other)] Unknown` plus
  `#[non_exhaustive]` is well-defined; no model uses `deny_unknown_fields`; every future field is
  optional + defaulted + skip-if-none. Degradation semantics are per-position: an `Unknown` in an
  object's primary enum (`post.kind`) fails validation and readers skip the object; an `Unknown`
  in a secondary enum (`feed.content`) degrades to "no constraint"; deserialization never crashes.
  Unknown members are tolerated on read AND preserved on rewrite: every wire model carries an
  opaque flattened catch-all map, so a client rewriting an object round-trips members it does not
  understand instead of destroying another client's data (tolerating without preserving would let
  any older client drop every field added after it shipped). Conformance vectors cover both the
  unknown-value and the preservation behavior. Preservation fixes typed-rewrite data loss, not
  concurrency: concurrent writers still clobber whole-file under last-write-wins where that is
  the documented rule.
- **Canonical-encoding id rule.** An id is valid iff re-encoding its decoded bytes reproduces the
  input (closed-form final-char check). v0 accepted dozens of alias spellings per id (lowercase,
  `O`->`0`, dangling bits), each a distinct homeserver key; that leniency is removed.
- **Engine-free validation.** `url::Url` (normalizes junk into acceptance), the `mime` crate, and
  full-Unicode case/trim are replaced by pinned rules (a strict raw-string canonicalizer, a
  frozen whitespace table, ASCII-only label folding, code-point lengths) that a Rust and a
  hand-written JS implementation reproduce byte-for-byte.
- **No silent sanitize-rewrites.** v0 rewrote `[DELETED]` names to "anonymous" and
  truncated-then-blanked over-long inputs; v1 makes invalid input a validation error.

## B1. User (profile)
v0 `pub/pubky.app/profile.json` -> v1 `pub/social/v1/profile.json`.
- `image` accepts pubky/http/https via one shared image validator (cap 300). pubky-app avatars are
  pubky file URIs; an http-only rule would reject every real avatar.
- The `[DELETED]` magic string dies entirely: v0's silent `[DELETED]` -> "anonymous" rewrite is
  removed with NO replacement rule; `[DELETED]` is an ordinary legal name. **Required nexus
  upgrade (gates v1 indexing):** the indexer currently keys deletion on that literal; it must key
  on a real flag (`deleted` on the indexed row / UserView) before indexing any v1 data. How a
  deleted account is displayed then becomes pure client presentation.
- Fields and caps otherwise unchanged; profile stays public (identity must be readable).

## B2. Post
v0 `pub/pubky.app/posts/{id}` (one flat file, overwritten on edit) -> v1
`{pub|priv}/social/v1/posts/{id}/{editId}.json`, referenced versionlessly as `posts/{id}`;
`{id}` and `{editId}` are both canonical TimestampIds (Appendix A3, A4).
- **Per-edit path versioning.** Each edit is a new file; the first version reuses the post id;
  references use the versionless form so edits never orphan replies or re-key tags. A counter was
  rejected (races across devices on a substrate with no compare-and-swap). Now-or-never: a
  `posts/{id}` file and a `posts/{id}/` directory are mutually exclusive on the homeserver.
- **Kinds renamed** `short`->`note`, `long`->`article` (nature, not length); all seven kinds kept.
- **`embed`** flattened from `{uri, kind}` to a plain URI string (kind is derivable from the target).
- **`attachments`** become `Vec<{uri, alt?, name?}>`, always `[]` never null. Objects, not strings,
  so per-item metadata (alt text now; hash/blurhash later) is additive; ships two committed fields.
- **`lock`** kept: the value is the lock-FILE URI (`pub/locks.app/<lock_id>.json`), and presence
  means "locked content" regardless of kind (matches the Locks feature's resolved design).
- **Dual-root:** posts may live under `/priv/` (drafts, private notes, private collections).
- The `[DELETED]` content sentinel dies with no replacement (same flag rule as B1); absence is
  the tombstone, synthesized by the indexer from real deletion state, never from content strings.

## B3. ArticleContent (new)
v0: pubky-app hand-rolls `{title, body}` JSON inside `long` posts, unspecified, cover smuggled as
`attachments[0]`. v1: a typed `{title, body, cover_image?}` envelope in `content` when
`kind == article`. Per-kind content shape is now-or-never; the cover moves into the envelope.
Articles may carry parent/embed/attachments (an article can be a reply or carry media).

## B4. CollectionContent
Shape unchanged (`{name, description?, items[], cover_image?}`). `items` now accept any
reference-tier pubky URI (any resource, any app), not just `posts/` under `pubky.app` (a curated
list may include foreign resources). Private collections come free from the dual-root post family.

## B5. Tag
`tags/{id}.json`. Id hashed over the canonicalized target plus a frozen-trimmed, ASCII-folded
label (v0 used engine `to_lowercase` and `url::Url` normalization, neither reproducible across
implementations; content-addressed ids must freeze their input functions). Injectivity holds only
because the label rejects `:`. One write location, any target: every app writes tags at the
author's `pub/social/v1/tags/`, and the target may be any public resource (social objects, other
apps' objects, external URIs). A logical tag therefore has exactly one possible address:
re-tagging self-overwrites idempotently, apps on the same account converge on the same file, and
the indexer reads one namespace with no writing-app dimension (reading `tags/` directories in
other app namespaces survives only as a legacy rule).

## B6. Bookmark
v0 `pub/pubky.app/bookmarks/{HashId(uri)}` (public, one-way filename, GET-per-file) -> v1
`priv/social/v1/bookmarks/{filename}.json`.
- **Private** (reader set is the owner).
- **Target in the filename, reversibly.** Primary form: `base64url(canonical target)` for
  targets up to 187 bytes. The math: 187 bytes encode to 250 base64url characters, and 250 +
  `.json` (5) = 255, the homeserver's per-segment maximum, so 187 is the largest cap ANY
  reversible encoding allows (base64url is the densest standard encoding that is also `/`- and
  `%`-free and natively JS-decodable). Longer targets use the overflow form `~ + HashId(target)`
  with the target kept in content. Listing costs zero GETs for primary-form entries (the
  overwhelming majority); each overflow entry costs one GET to recover its target.
- Content shrinks to `{created_at}` (plus `target` only in overflow).

## B7. Follow
`follows/{followeePk}.json`, `{created_at}`. Public (the social graph). Only the cross-cutting
changes apply; the filename-is-target pattern (one LIST answers "who do I follow") is kept.

## B8. Mute
`priv/social/v1/mutes/{muteePk}.json`. Moves to `/priv/`; who you muted is sensitive and its only
reader is the owner (the indexer has zero mute consumers). Shape unchanged.

## B9. LastRead
`priv/social/v1/last_read.json`, microseconds (was the lone milliseconds outlier). Private.

## B10. File (media), the v0 File + Blob pair collapsed
v0: two objects, `files/{id}` metadata + `blobs/{hash}` bytes -> v1: ONE content-addressed media
object `files/{hash}.{ext}`, the raw bytes.
- `name` relocates to the attachment object (per-reference, so shared bytes can carry different
  names). The declared MIME is consumed once at upload to pick the extension and is never stored;
  size and served type come from the bytes and headers. Authoring time is the referencing post's id.
- Deletes the `src` indirection, the two-PUT dance, and the metadata-per-blob ambiguity. Dual-root.
- The extension comes from a frozen MIME-to-ext map (`.bin` fallback), path-only, never hashed.

## B11. Feed
v0 `pub/pubky.app/feeds/{HashId(serde-json config)}` (public) -> v1
`{priv|pub}/social/v1/feeds/{id}.json`.
- **Private by default, published by choice** (copy the same file to `/pub/`). Reader set is the
  owner today; publishing is a deliberate act.
- **Content-addressed id over a pinned config string** (not serde output, which is field-order
  fragile and not JS-reproducible). Two users publishing the same config share an id, so future
  cross-homeserver popularity ranking is additive indexer work.
- All three enums gain `Unknown` (v0 hard-crashes old clients on any new reach/layout/sort value);
  `name` gains a cap.

## B12. Settings (new)
v0: pubky-app's `pub/pubky.app/settings.json` was the only unspec'd homeserver artifact, world-readable
while exposing the user's privacy posture (`require_pin`, `sign_out_inactive`) to a reader set of
one. v1: `PubkySocialSettings` at `priv/social/v1/settings.json`, all sections optional, whole-file
last-write-wins on a microsecond `updated_at`, the dead per-file `version` field dropped. Rewrites
preserve unknown members via the catch-all map (B0), so an older client editing one field cannot
destroy a section a newer client wrote; only the concurrent-edit race is lost, by LWW design.

## B13. Parser and `Resource`
v0 `url::Url`-based, hard-rejects any non-`pubky.app` app path, silently accepts userinfo/`..`/query
-> v1 one closed grammar: `Foreign` and `UnsupportedVersion` are first-class handled categories
(never errors), failed id validation yields `Unknown` (never a panic or error), wrong-root is
unrepresentable. A future `social/v2` reads as "upgrade me," not garbage. The normative grammar
is Appendix A; the reference crate and its committed conformance vectors are the executable form.

## B14. IDs
TimestampId (post and edit ids), HashId (tag/media/feed files, 128-bit), PubkyId (host/follow/mute), all
under the canonical-encoding rule (B0). TimestampId gains a per-session monotonic mint guard (the
JS runtime mints at ms resolution, so same-ms writes would clobber on path). No id function
content-addresses a serialized struct, so the Rust/JS byte-identity surface is pure string/byte
functions.

# Part C: Migration

- **Client-side, opt-in, resumable from the homeserver tree alone, permanent multi-epoch.** A
  dormant user may migrate years later in one pass; the indexer dual-reads every epoch forever
  (the permanent v0 parser stays, since the v1 parser classifies `pubky.app` as foreign).
- **STRICTLY non-destructive.** Migration never deletes any legacy-epoch data. Privacy lost before
  v1 is already lost; future activity is private. Private-tier legacy public copies are left inert
  (the indexer stops surfacing them).
- **Deterministic and total** over real v0 data: each record sources from its highest present
  epoch, transforms compose in memory writing only the latest, resume is by destination existence,
  ids/hashes are re-derived not minted.
- **Deletion** is user-initiated only: deleting a public object (post, file) removes every copy
  across epochs and both roots. Cross-epoch copies exist because migration copies and never
  deletes: a migrated post lives at BOTH `pub/pubky.app/posts/{id}` and
  `pub/social/v1/posts/{id}/...`, and a delete that missed the legacy copy would resurrect the
  post on a from-scratch reindex;
  private-tier deletes touch only the `/priv/` file. Absence is the tombstone: on a dumb blob
  store the owner's files are the only durable state, so restoring an old backup republishes its
  contents, accepted by design. Durable per-object tombstone files were considered and rejected
  (the substrate cannot enforce them against the owner's own writes, and public tombstones would
  leak deletion metadata forever).
- The indexer contract: dedup on `(author, resource_type, stable_id)`, tag id-sets for cross-epoch
  un-tagging, intrinsic-time ranking (decode the post id), tombstone only when no publicly visible
  copy survives.

# Part D: Rollout and subtasks

Spec crate on a long-lived `v1` branch, one PR per task, CI green on every commit, version
`1.0.0-alpha.N` until release. Each task is independently mergeable in this order.

**Spine (serialized):**
- [ ] **S1** rename crate + types (wire-invariant).
- [ ] **S2** retire the wasm surface; native rlib; single-sourced DATA assets (limits, enum names).
- [ ] **S3** forward-compat contract (`Unknown` on every wire enum).
- [ ] **S4** validation core: limits table, canonical id validators, frozen text ops, mint guard.
- [ ] **S5** path epoch + canonicalizers + parser (atomic).
- [ ] **S6a** post wire shapes (kinds, embed, attachments, Article envelope, reserved literal).
- [ ] **S6b** post storage, roots, lifecycle (versioned builders, root rule, publish/unpublish/delete).
- [ ] **S7** tag + collection (canonical id inputs, reference-tier items).
- [ ] **S8** media collapse (single bytes object, MIME map, parser ext-strip).
- [ ] **S9** feed dual-root + user gates.
- [ ] **S10** private tier + bookmarks + settings.
- [ ] **S11** legacy_v0 module + cross-epoch normalization (`stable_id`/`resolve_deref`).

**Pure-JS + gate:**
- [ ] **J1** Rust conformance-vector generator (byte-identity + verdict tiers, fuzzed).
- [ ] **J2** hand-written pure-JS package (ids, paths, canonicalizers, validation, builders).
- [ ] **J3** merge-blocking differential CI gate.

**Migrator:**
- [ ] **M1** transform registry + v0 reader (Rust reference emits semantic vectors).
- [ ] **M2** engine (epoch discovery, resume, source re-check, abort-if-no-`/priv/`).
- [ ] **M3** pubky.app `/migrate` route (caps, upgrade flow, live counts, settings import).

**Cross-repo:**
- [ ] **pubky-nexus:** per-epoch classify/adapt/normalize, permanent v0 parser, tag id-sets,
  intrinsic-time ranking, multi-epoch tombstones, bookmark-feature retirement, mixed-epoch resync.
- [ ] **pubky-app:** v1 adoption (new caps, kind strings, own-tree legacy read union so
  un-migrated users lose nothing, publish UI, media type threading, deletion engine).
  - [ ] **moderation:** mixed epoch support by both nexus and homeserver syncronization services as well as by checkstep-request services

**Gates:**
- [ ] **IN-PRIV** verify the target homeserver runs the `/priv/` tier and permits the write
  paths. Prerequisite for M2 onward and for any private-tier go-live, not a late release check.
- [ ] **REL** publish `1.0.0` (crate + npm), merge `v1` to main after nexus dual-read is live.

**Dependencies** (beyond the serialized spine order above): S11 needs S5. J1 needs S5 and
regenerates as later S tasks land; J2 needs J1; J3 needs J2 and is merge-blocking from then on.
M1 needs S11; M2 needs M1 and IN-PRIV; M3 needs M2 and J2. Nexus dual-read must be live before
any client writes v1 data; the pubky-app track needs J2 and that nexus gate. REL is last.

**Acceptance, every task:** its listed artifact lands with the full CI bar green (fmt, clippy
with warnings denied, tests, doctests, feature checks, regenerate-and-diff on committed data
assets); from J3 on, additionally the Rust/JS differential gate.

# Part E: Folds in and supersedes

Supersedes the v1 roadmap #12. Resolves: #47 (json extensions), #48 (attachment type array),
#55 (content hash, deferred to v1.x as a flat optional sibling with the shape pinned here), #120
(short-form URIs), #141 (reject non-canonical URIs). Mention-prefix cleanup (`pk:`) is handled
client-side and already shipped.

No open design decisions remain; the destructive migration carve-out was considered and rejected
in favor of strictly non-destructive migration (Part C).

# Appendix A: the v1 URI grammar (normative)

The reference crate's parser and its committed conformance vectors are the executable
definition; this is the same grammar in human-readable form.

## A1. URI forms and canonicalization

Accepted input forms: `pubky://<host>[/<path>]` and the SDK short form `pubky<host>[/<path>]`
(scheme prefix case-sensitive; `Pubky://` is invalid). The canonical output form is always
`pubky://<host>[/<path>]`.

Canonicalization is raw string work, byte-level, with no engine URL parser anywhere:

- **Host:** exactly 52 z-base32 characters (lowercase alphabet `ybndrfg8ejkmcpqxot1uwisza345h769`),
  final character `y` or `o` (the canonical-encoding rule: the trailing 4 pad bits must be zero).
  A `@` or `:` anywhere in the host is invalid (no userinfo, no port).
- **Path segments** (split on `/`): reject an empty segment (this covers trailing slashes and
  `//`), `.`, `..`, and any segment containing `%`, `?`, `#`, an ASCII control character, or a
  frozen-whitespace code point. There is NO percent-decoding, NO case folding, and NO segment
  normalization: what is stored is what was written.
- A bare host (`pubky://<host>`) is valid and canonical.

Failures here, plus the root check in A2 step 1, are the only HARD errors (in both cases no
resource with a visibility can be constructed); every other failure is a handled classification,
never an error and never a panic.

## A2. Classification

After canonicalization, split the path into segments and classify:

1. An empty path (a bare host) classifies as **User** (public): the URI references the user
   themselves. Otherwise segment 0 must be `pub` or `priv` (it becomes the resource's
   visibility); anything else is a hard error.
2. Segment 1 not `social`: **Foreign** `{namespace, version?, rest}`. Foreign data is valid,
   classified, and skipped by social readers; it is never an error.
3. Segment 1 `social`, segment 2 matching `v[0-9]+` but not the supported epoch:
   **UnsupportedVersion** (a reader's "upgrade me" signal).
4. Segment 2 not an epoch segment: **Unknown**.
5. Otherwise dispatch on the remaining segments per the table below. Any non-match, any failed
   id validation, any wrong-root spelling of a single-root resource, and any `_`-prefixed leaf
   (reserved client-private names) is **Unknown**.

## A3. Resource dispatch (owner-relative paths under `{root}/social/v1/`)

| Resource | Path remainder | Roots | Leaf rule |
|---|---|---|---|
| User | `profile.json` | pub | none |
| Post (reference) | `posts/{id}` | both | `{id}` = canonical TimestampId |
| Post (version) | `posts/{id}/{editId}.json` | both | both canonical TimestampIds |
| File (media) | `files/{hash}.{ext}` | both | strip exactly one extension, case-sensitively, ONLY if it is in the frozen extension set; remainder = canonical HashId; unknown or absent extension is Unknown |
| Tag | `tags/{id}.json` | pub | `{id}` = canonical HashId |
| Follow | `follows/{pk}.json` | pub | `{pk}` = canonical host key (as A1) |
| Mute | `mutes/{pk}.json` | priv | same |
| Bookmark | `bookmarks/{filename}.json` | priv | FORM check only: every character in the base64url alphabet, or `~` followed by 26 canonical Crockford characters; full round-trip validation is the reader's job |
| Feed | `feeds/{id}.json` | both | `{id}` = canonical HashId |
| Settings | `settings.json` | priv | none |
| LastRead | `last_read.json` | priv | none |

`.json` stripping is exact and single: `follows/<pk>.json.json` leaves `<pk>.json`, which fails
key validation and classifies Unknown.

## A4. Canonical id encodings

An id is valid iff re-encoding its decoded bytes reproduces the input. Closed form:

- **TimestampId** (post/edit ids): 13 chars of uppercase Crockford base32
  (`0123456789ABCDEFGHJKMNPQRSTVWXYZ`); final char in `{0,2,4,6,8,A,C,E,G,J,M,P,R,T,W,Y}`
  (one trailing pad bit, must be zero). Lowercase and the alias letters `O/I/L/U` are invalid.
- **HashId** (tag/media/feed ids): 26 chars, same alphabet; final char in `{0,4,8,C,G,M,R,W}`
  (two pad bits).
- **Host key**: as A1 (52 z-base32, final `y`/`o`).

Time bounds are never checked at parse time; only canonicality is.

## A5. Representative vectors

`<pk>` is any valid host key, `TS` a canonical TimestampId, `H26` a canonical HashId.

| Input | Result |
|---|---|
| `pubky<pk>/pub/social/v1/posts/TS` | Public Post reference (short form accepted) |
| `pubky://<pk>/priv/social/v1/posts/TS/TS.json` | Private Post version |
| `pubky://<pk>` | User (bare host) |
| `pubky://<pk>/pub/social/v1/files/H26.svg` | Public File, id `H26` |
| `pubky://<pk>/pub/social/v1/files/H26.JPG` | Unknown (extension set is case-sensitive) |
| `pubky://<pk>/pub/social/v1/posts/ts-lowercase` | Unknown (non-canonical id) |
| `pubky://<pk>/pub/social/v2/posts/TS` | UnsupportedVersion |
| `pubky://<pk>/pub/pubky.app/posts/TS` | Foreign |
| `pubky://<pk>/pub/social/v1/mutes/<pk>.json` | Unknown (mutes are priv-rooted) |
| `Pubky://<pk>/pub/social/v1/profile.json` | hard error (scheme case) |
| `pubky://user@<pk>/pub/social/v1/profile.json` | hard error (userinfo) |
| `pubky://<pk>/pub/social/v1/posts/../profile.json` | hard error (dot-dot) |
| `pubky://<pk>/dav/social/v1/profile.json` | hard error (unknown root) |
