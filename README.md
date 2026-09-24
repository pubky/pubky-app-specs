# pubky-social-specs

[![crates.io](https://img.shields.io/crates/v/pubky-social-specs)](https://crates.io/crates/pubky-social-specs)
[![docs.rs](https://img.shields.io/docsrs/pubky-social-specs)](https://docs.rs/pubky-social-specs)
[![npm](https://img.shields.io/npm/v/pubky-social-specs)](https://www.npmjs.com/package/pubky-social-specs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Rust types, builders, and validation for Pubky social data models. The builders trim text and fold tokens; reading an object back never rewrites it. Use this crate to build JSON that matches what [Pubky indexers](https://github.com/pubky/pubky-nexus) expect.

## Installation

**Rust** ([crates.io](https://crates.io/crates/pubky-social-specs)):

```bash
cargo add pubky-social-specs
```

**JavaScript / TypeScript** ([npm](https://www.npmjs.com/package/pubky-social-specs)): see [`pkg/README.md`](https://github.com/pubky/pubky-social-specs/blob/main/pkg/README.md).

## Rust quick start

```rust
use pubky_social_specs::{
    traits::{HasPath, Validatable},
    PubkySocialUser,
};
use serde_json::to_vec;

// Create a user profile
let user = PubkySocialUser::new("Alice".into(), None, None, None, None);
let path = PubkySocialUser::create_path(); // /pub/social/v1/profile.json
let json = to_vec(&user).unwrap();

// Parse and validate JSON from storage
let profile = PubkySocialUser::try_from(&json, "", &PUB_CTX).unwrap();
```

For a full homeserver flow, see [`examples/create_user.rs`](https://github.com/pubky/pubky-social-specs/blob/main/examples/create_user.rs).

## Why use this crate

- **Validation consistency:** same validation rules as Pubky indexers, and the same canonical forms on the wire.
- **Auto IDs and paths:** generates IDs, paths, and URLs according to Pubky standards.
- **Single source of truth:** the Rust models drive native consumers and the WASM bindings.
- **No silent rewrites:** builders canonicalize (an attachment name is trimmed there), and after that a value is stored as written and counted as written, so reading never repairs what a writer stored.

## Features

| Feature   | Purpose                        |
| --------- | ------------------------------ |
| `openapi` | OpenAPI schemas via `utoipa`   |

```toml
pubky-social-specs = { version = "1.0.0-alpha.3", features = ["openapi"] }
```

- **MSRV:** 1.89 (see `rust-version` in `Cargo.toml`)
- **API docs:** [docs.rs/pubky-social-specs](https://docs.rs/pubky-social-specs)

## Models

| Rust type           | Purpose                                  |
| ------------------- | ---------------------------------------- |
| `PubkySocialUser`      | User profile information                 |
| `PubkySocialFile`      | Media bytes                              |
| `PubkySocialPost`      | Posts, replies, embeds, and collections  |
| `PubkySocialTag`       | Tags applied to Pubky URIs               |
| `PubkySocialBookmark`  | Private bookmarks, target in the filename |
| `PubkySocialFollow`    | Follow relationships                     |
| `PubkySocialFeed`      | Feed configurations                      |
| `PubkySocialMute`      | Muted users                              |

## Reading 0.x data

`legacy_v0` is the 0.x reader, frozen at the 0.8.0 pin. It carries that release's parser, read models and validation copied unchanged, so an object 0.x accepted or rejected keeps the same answer forever, and none of it is edited to match the 1.x rules. Hand its object enum a stored URI and the bytes and it answers what 0.8.0 answered.

`stable_id` keys a stored path the same under either epoch, so a migrated object indexes in place rather than twice:

```rust
use pubky_social_specs::stable_id;

assert_eq!(
    stable_id("pub/pubky.app/posts/0RDX5H0000000"),
    stable_id("pub/social/v1/posts/0RDX5H0000000/0RDX5J0000002.json"),
);
```

## Specification

The 1.x design is in [`docs/rfc-v1-social-specs.md`](https://github.com/pubky/pubky-social-specs/blob/main/docs/rfc-v1-social-specs.md). The legacy 0.x layout is in [`docs/SPEC_V0.md`](https://github.com/pubky/pubky-social-specs/blob/main/docs/SPEC_V0.md), for reading un-migrated data.

## License

MIT
