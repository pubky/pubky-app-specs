#[cfg(target_arch = "wasm32")]
use js_sys::Date;

use std::sync::atomic::{AtomicI64, Ordering};
use url::Url;

/// Returns the current timestamp in microseconds since the UNIX epoch.
#[cfg(target_arch = "wasm32")]
pub fn timestamp() -> i64 {
    let ms = Date::now() as i64;
    ms * 1_000
}

#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(not(target_arch = "wasm32"))]
pub fn timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as i64
}

/// Trims whitespace and normalizes a URL if valid and invalid URLs are preserved
/// (not discarded) so validation can catch them
pub fn sanitize_url(input: &str) -> String {
    let trimmed = input.trim();
    match Url::parse(trimmed) {
        Ok(parsed_url) => parsed_url.to_string(),
        Err(_) => trimmed.to_string(),
    }
}

/// The 25 code points with White_Space=Yes at Unicode 15.1. Frozen: never
/// regenerate from a newer Unicode table, because content-addressed ids
/// depend on trim. Engine notions of whitespace (`str::trim`, `\s`,
/// `String.prototype.trim`) stay off the validation surface. U+200B and
/// U+FEFF are deliberately absent (not White_Space). "ASCII control" in this
/// crate means exactly U+0000..U+001F and U+007F (`char::is_ascii_control`).
pub const FROZEN_WHITESPACE: [char; 25] = [
    '\u{0009}', '\u{000A}', '\u{000B}', '\u{000C}', '\u{000D}', '\u{0020}', '\u{0085}', '\u{00A0}',
    '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}',
    '\u{2007}', '\u{2008}', '\u{2009}', '\u{200A}', '\u{2028}', '\u{2029}', '\u{202F}', '\u{205F}',
    '\u{3000}',
];

pub fn is_frozen_whitespace(c: char) -> bool {
    FROZEN_WHITESPACE.contains(&c)
}

/// Trims leading and trailing frozen-whitespace code points. The one trim
/// allowed anywhere on the validation surface.
pub fn frozen_trim(s: &str) -> &str {
    s.trim_matches(is_frozen_whitespace)
}

/// ASCII-only lowercase fold (tag labels, MIME essences). Full-Unicode
/// lowercasing changes with Unicode versions, so it stays off this surface.
pub fn ascii_fold(s: &str) -> String {
    s.to_ascii_lowercase()
}

/// Length in Unicode code points; equals JS `[...s].length`.
pub fn code_point_len(s: &str) -> usize {
    s.chars().count()
}

const CROCKFORD_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// An id is valid iff re-encoding its decoded bytes reproduces the input
/// byte-for-byte: every char from the canonical uppercase alphabet, and
/// every dangling (pad) bit zero. Alias spellings (`O` for `0`, lowercase)
/// would otherwise name the same object under a different homeserver key.
fn canonical_crockford_value(id: &str, expected_chars: usize) -> Result<u128, String> {
    let bytes = id.as_bytes();
    if bytes.len() != expected_chars {
        return Err(format!(
            "Validation Error: Invalid ID length: must be {expected_chars} characters"
        ));
    }
    let mut acc: u128 = 0;
    for &b in bytes {
        let val = CROCKFORD_ALPHABET
            .iter()
            .position(|&c| c == b)
            .ok_or("Validation Error: non-canonical Crockford character")?;
        acc = (acc << 5) | val as u128;
    }
    Ok(acc)
}

/// TimestampId: 13 chars, 65 bits, one dangling bit that must be zero
/// (equivalently, the final char is one of `0 2 4 6 8 A C E G J M P R T W Y`).
/// Returns the decoded microseconds. Canonicality only; time bounds are the
/// caller's concern.
pub fn validate_timestamp_id_format(id: &str) -> Result<i64, String> {
    let acc = canonical_crockford_value(id, 13)?;
    if acc & 1 != 0 {
        return Err("Validation Error: non-canonical ID (dangling bit set)".into());
    }
    Ok((acc >> 1) as u64 as i64)
}

/// HashId: 26 chars, 130 bits, two dangling bits that must be zero
/// (equivalently, the final char is one of `0 4 8 C G M R W`).
pub fn validate_hash_id_format(id: &str) -> Result<(), String> {
    let acc = canonical_crockford_value(id, 26)?;
    if acc & 0b11 != 0 {
        return Err("Validation Error: non-canonical ID (dangling bits set)".into());
    }
    Ok(())
}

static LAST_MINTED_MICROS: AtomicI64 = AtomicI64::new(0);

/// Strictly increasing microsecond mint for TimestampId creation. If the
/// clock has not advanced past the last issued value, issues last + 1. A
/// burst may run a few microseconds ahead of the wall clock, well inside
/// the now + 2h validity bound. `timestamp()` stays the raw clock for
/// `created_at` fields.
pub fn mint_timestamp_micros() -> i64 {
    let now = timestamp();
    let prev = LAST_MINTED_MICROS
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |last| {
            Some(if now > last { now } else { last + 1 })
        })
        .expect("closure always returns Some");
    if now > prev {
        now
    } else {
        prev + 1
    }
}

/// 2^53 - 1: the largest integer JSON round-trips identically through a JS
/// caller. Every i64 wire integer must satisfy |v| <= this.
pub const MAX_SAFE_JSON_INT: i64 = 9_007_199_254_740_991;

pub fn validate_safe_json_int(v: i64) -> Result<(), String> {
    if !(-MAX_SAFE_JSON_INT..=MAX_SAFE_JSON_INT).contains(&v) {
        return Err(format!(
            "Validation Error: integer {v} outside the JSON-safe range"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use base32::{decode, encode, Alphabet};

    const TS_LOWER_BOUND: i64 = 1727740800000000;

    fn crockford_oracle(id: &str) -> bool {
        decode(Alphabet::Crockford, id).map(|b| encode(Alphabet::Crockford, &b))
            == Some(id.to_string())
    }

    #[test]
    fn timestamp_id_kats() {
        assert_eq!(validate_timestamp_id_format("FZZZZZZZZZZZY"), Ok(i64::MAX));
        assert_eq!(
            encode(Alphabet::Crockford, &i64::MAX.to_be_bytes()),
            "FZZZZZZZZZZZY"
        );
        assert_eq!(
            encode(Alphabet::Crockford, &TS_LOWER_BOUND.to_be_bytes()),
            "00326QR0MQG00"
        );
        assert_eq!(
            validate_timestamp_id_format("00326QR0MQG00"),
            Ok(TS_LOWER_BOUND)
        );
        // The O alias still decodes under the crate's decoder; canonical validation rejects it.
        assert!(decode(Alphabet::Crockford, "O0326QR0MQG00").is_some());
        assert!(validate_timestamp_id_format("O0326QR0MQG00").is_err());
    }

    #[test]
    fn timestamp_id_validation_equals_the_reencode_oracle() {
        for id in [
            "0032SSN7Q4EVG",
            "0034A0X7NJ52G",
            "00326QR0MQG00",
            "FZZZZZZZZZZZY",
            "0032ssn7q4evg",
            "O032SSN7Q4EVG",
            "I032SSN7Q4EVG",
            "L032SSN7Q4EVG",
            "U032SSN7Q4EVG",
            "0000000000001",
            "0000000000002",
            "000000000000",
            "00000000000000",
            "",
            "0032SSN7Q4EV",
            "0032SSN7Q4EVG0",
        ] {
            let expected = id.len() == 13 && crockford_oracle(id);
            assert_eq!(validate_timestamp_id_format(id).is_ok(), expected, "{id:?}");
        }
    }

    #[test]
    fn hash_id_validation_equals_the_reencode_oracle() {
        for id in [
            "PZBQ010FF079VVZPQG1RNFN6DR",
            "8Z8CWH8NVYQY39ZEBFGKQWWEKG",
            "00000000000000000000000000",
            "0000000000000000000000000Z",
            "0000000000000000000000001",
            "000000000000000000000000000",
            "pzbq010ff079vvzpqg1rnfn6dr",
            "",
        ] {
            let expected = id.len() == 26 && crockford_oracle(id);
            assert_eq!(validate_hash_id_format(id).is_ok(), expected, "{id:?}");
        }
    }

    #[test]
    fn frozen_trim_strips_exactly_the_table() {
        for c in FROZEN_WHITESPACE {
            assert_eq!(frozen_trim(&format!("{c}a{c}")), "a", "{:?}", c as u32);
        }
        assert_eq!(frozen_trim("\u{200B}a\u{FEFF}"), "\u{200B}a\u{FEFF}");
        assert_eq!(frozen_trim("\u{3000}a\u{00A0}"), "a");
        assert_eq!(frozen_trim(" a b "), "a b");
        assert_eq!(FROZEN_WHITESPACE.len(), 25);
        let mut sorted = FROZEN_WHITESPACE;
        sorted.sort_unstable();
        assert_eq!(sorted, FROZEN_WHITESPACE, "table is ascending");
    }

    #[test]
    fn text_ops_are_ascii_and_code_point_based() {
        assert_eq!(ascii_fold("İX"), "İx");
        assert_eq!(ascii_fold("RUST"), "rust");
        assert_eq!(code_point_len("a👍"), 2);
    }

    #[test]
    fn safe_json_int_bounds() {
        assert!(validate_safe_json_int(MAX_SAFE_JSON_INT).is_ok());
        assert!(validate_safe_json_int(-MAX_SAFE_JSON_INT).is_ok());
        assert!(validate_safe_json_int(MAX_SAFE_JSON_INT + 1).is_err());
        assert!(validate_safe_json_int(i64::MIN).is_err());
    }

    #[test]
    fn mint_is_strictly_increasing() {
        let mut last = 0;
        for _ in 0..10_000 {
            let t = mint_timestamp_micros();
            assert!(t > last);
            last = t;
        }
    }
}
