# Unicode String Length Handling

## Overview

This document explains how string length validation works in `pubky-app-specs` and the important differences between JavaScript's native string length and Rust's character counting.

## The Problem

JavaScript and Rust count string length differently for certain Unicode characters:

| Character | Type | Rust `.chars().count()` | JS `.length` |
|-----------|------|-------------------------|--------------|
| `"Hello"` | ASCII | 5 | 5 |
| `"中文"` | Chinese | 2 | 2 |
| `"café"` | Accented | 4 | 4 |
| `"🔥"` | Emoji | **1** | **2** |
| `"𒅃"` | Cuneiform | **1** | **2** |
| `"𓀀"` | Hieroglyph | **1** | **2** |

### Why the Difference?

- **JavaScript** uses **UTF-16** encoding internally. The `.length` property counts **UTF-16 code units**.
- **Rust** `.chars().count()` counts **Unicode code points** (scalar values).

Characters in the **Basic Multilingual Plane (BMP)** (U+0000 to U+FFFF) use 1 UTF-16 code unit.
Characters **outside the BMP** (U+10000 and above) require a **surrogate pair** (2 UTF-16 code units).

### Characters Outside BMP (Affected by This Difference)

| Category | Examples | UTF-16 Units per Char |
|----------|----------|----------------------|
| Emoji | 🔥 🚀 😀 👋 🌍 | 2 |
| Cuneiform (Sumerian) | 𒅃 𒀀 𒁀 | 2 |
| Egyptian Hieroglyphs | 𓀀 𓆉 𓍄 | 2 |
| Musical Symbols | 𝄞 𝄢 | 2 |
| Mathematical Alphanumeric | 𝔸 𝕏 | 2 |
| Historic Scripts | Various | 2 |

### Characters in BMP (No Difference)

| Category | Examples | UTF-16 Units per Char |
|----------|----------|----------------------|
| ASCII/Latin | A-Z, a-z, 0-9 | 1 |
| Latin Extended | á, ñ, ü, ø | 1 |
| Chinese | 中文字 | 1 |
| Japanese (Hiragana/Katakana/Kanji) | 日本語 | 1 |
| Korean (Hangul) | 한글 | 1 |
| Arabic | العربية | 1 |
| Hebrew | עברית | 1 |
| Cyrillic | русский | 1 |
| Greek | ελληνικά | 1 |
| Thai | ไทย | 1 |

## Our Solution: WASM-Based Validation

All validation in `pubky-app-specs` happens **inside the WASM module** (Rust), not in JavaScript.

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    JavaScript Client                     │
│                                                         │
│   const user = PubkyAppUser.fromJson({                  │
│       name: "Alice🔥",                                  │
│       bio: "Hello 𓀀"                                   │
│   });                                                   │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│                    WASM Module (Rust)                    │
│                                                         │
│   1. Deserialize JSON                                   │
│   2. Sanitize (trim whitespace, normalize)              │
│   3. Validate (using .chars().count())  ◄── Single      │
│   4. Return Result                           Source     │
│                                              of Truth   │
└─────────────────────────────────────────────────────────┘
```

### Why This Works

1. **Single Source of Truth**: All validation uses Rust's `.chars().count()`, which counts Unicode code points.
2. **No JS Validation**: JavaScript never validates string lengths directly—it delegates to WASM.
3. **Consistent Behavior**: Whether the user types emoji, Chinese, or cuneiform, the validation is consistent.

### Example: Username Validation

```rust
// In Rust (WASM)
const MAX_USERNAME_LENGTH: usize = 50;

fn validate(&self, _id: Option<&str>) -> Result<(), String> {
    let name_length = self.name.chars().count();  // Unicode code points
    if name_length > MAX_USERNAME_LENGTH {
        return Err("Validation Error: Invalid name length".into());
    }
    Ok(())
}
```

| Input | `.chars().count()` | Valid? (max 50) |
|-------|-------------------|-----------------|
| `"Alice"` | 5 | ✅ |
| `"🔥".repeat(50)` | 50 | ✅ |
| `"🔥".repeat(51)` | 51 | ❌ |
| `"𓀀".repeat(50)` | 50 | ✅ |

## Important: Don't Validate in JavaScript

If you need client-side validation (for UX feedback), you **must** match Rust's behavior or trust in pubk-app-specs WASM module.

```javascript
// ❌ WRONG - will reject valid input
if (username.length > 50) {
    showError("Username too long");
}

// ✅ CORRECT - matches Rust's .chars().count()
if ([...username].length > 50) {
    showError("Username too long");
}

// ✅ ALSO CORRECT - using Array.from
if (Array.from(username).length > 50) {
    showError("Username too long");
}
```

### JavaScript Length Methods Comparison

```javascript
const str = "Hi🔥";

str.length                    // 4 (UTF-16 code units) ❌
[...str].length              // 3 (Unicode code points) ✅
Array.from(str).length       // 3 (Unicode code points) ✅
```

## Edge Cases: Grapheme Clusters

Note: Even `.chars().count()` in Rust doesn't handle **grapheme clusters** perfectly:

| String | Visual | Code Points | Graphemes |
|--------|--------|-------------|-----------|
| `"👨‍👩‍👧‍👦"` | 1 family emoji | 7 | 1 |
| `"🇺🇸"` | 1 flag | 2 | 1 |
| `"é"` (composed) | 1 character | 1 | 1 |
| `"é"` (decomposed: e + ◌́) | 1 character | 2 | 1 |

For most use cases (usernames, tags, bios), counting code points is sufficient. True grapheme cluster counting would require additional dependencies.

## Summary

| Aspect | Approach |
|--------|----------|
| **Validation Location** | WASM (Rust) only |
| **Length Method** | `.chars().count()` (Unicode code points) |
| **JS Client** | Use `[...str].length` if local validation needed |
| **Affected Characters** | Emoji, ancient scripts, musical symbols |
| **Unaffected Characters** | ASCII, Chinese, Japanese, Arabic, etc. |

## References

- [Unicode Standard](https://unicode.org/)
- [UTF-16 on Wikipedia](https://en.wikipedia.org/wiki/UTF-16)
- [JavaScript String length](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/length)
- [Rust chars() documentation](https://doc.rust-lang.org/std/primitive.str.html#method.chars)

