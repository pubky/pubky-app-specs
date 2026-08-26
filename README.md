# pubky-social-specs

[![crates.io](https://img.shields.io/crates/v/pubky-social-specs)](https://crates.io/crates/pubky-social-specs)
[![docs.rs](https://img.shields.io/docsrs/pubky-social-specs)](https://docs.rs/pubky-social-specs)
[![npm](https://img.shields.io/npm/v/pubky-social-specs)](https://www.npmjs.com/package/pubky-social-specs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Rust types, sanitization, and validation for Pubky social data models. Use this crate to build JSON that matches what [Pubky indexers](https://github.com/pubky/pubky-nexus) expect.

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
let path = PubkySocialUser::create_path(); // /pub/pubky.app/profile.json
let json = to_vec(&user).unwrap();

// Parse and validate JSON from storage
let profile = PubkySocialUser::try_from(&json, "").unwrap();
```

For a full homeserver flow, see [`examples/create_user.rs`](https://github.com/pubky/pubky-social-specs/blob/main/examples/create_user.rs).

## Why use this crate

- **Validation consistency:** same sanitization and validation rules as Pubky indexers.
- **Auto IDs and paths:** generates IDs, paths, and URLs according to Pubky standards.
- **Single source of truth:** Rust models drive native apps, WASM bindings, and this spec.

## Features

| Feature   | Purpose                        |
| --------- | ------------------------------ |
| `openapi` | OpenAPI schemas via `utoipa`   |

```toml
pubky-social-specs = { version = "1.0.0-alpha.1", features = ["openapi"] }
```

- **MSRV:** 1.89 (see `rust-version` in `Cargo.toml`)
- **API docs:** [docs.rs/pubky-social-specs](https://docs.rs/pubky-social-specs)

## Models

| Rust type           | Purpose                                  |
| ------------------- | ---------------------------------------- |
| `PubkySocialUser`      | User profile information                 |
| `PubkySocialFile`      | Uploaded file metadata                   |
| `PubkySocialPost`      | Posts, replies, embeds, and collections  |
| `PubkySocialTag`       | Tags applied to Pubky URIs               |
| `PubkySocialBookmark`  | Bookmarks for Pubky URIs                 |
| `PubkySocialFollow`    | Follow relationships                     |
| `PubkySocialFeed`      | Feed configurations                      |
| `PubkySocialMute`      | Muted users                              |
| `PubkySocialBlob`      | Raw binary file data                     |
| `PubkySocialLastRead`  | Last-read notification timestamp         |

## Specification

The legacy 0.x layout is documented in [`docs/SPEC_V0.md`](https://github.com/pubky/pubky-social-specs/blob/v1/docs/SPEC_V0.md), for reading un-migrated data.

## License

MIT
