//! Declared MIME type in, path extension out. The extension is path-only and never enters the
//! hash, so this table cannot fork an identity.

use crate::common::ascii_fold;

/// The essence char class: ^[a-z0-9!#$&^_.+-]+/[a-z0-9!#$&^_.+-]+$,
/// hand-rolled (a regex dependency is not JS-mirrorable logic; this is).
fn is_essence_char(c: char) -> bool {
    matches!(c,
        'a'..='z' | '0'..='9' |
        '!' | '#' | '$' | '&' | '^' | '_' | '.' | '+' | '-')
}

/// Substring before the first ';', ASCII-folded, NO trimming (a leading space is malformed on
/// purpose). `None` = malformed. A second '/' cannot slip through: it is outside the char
/// class, so it fails the subtype scan.
pub fn essence(declared: &str) -> Option<String> {
    let before = match declared.find(';') {
        Some(i) => &declared[..i],
        None => declared,
    };
    let folded = ascii_fold(before);
    let (ty, sub) = folded.split_once('/')?;
    if ty.is_empty()
        || sub.is_empty()
        || !ty.chars().all(is_essence_char)
        || !sub.chars().all(is_essence_char)
    {
        return None;
    }
    Some(folded)
}

/// FROZEN single-valued map; the extension is path-only and never enters the hash, so a map
/// edit can never fork identity. Additions are crate-minor; rows are never changed or removed.
pub const MIME_TO_EXT: &[(&str, &str); 19] = &[
    ("image/jpeg", "jpg"),
    ("image/png", "png"),
    ("image/gif", "gif"),
    ("image/webp", "webp"),
    ("image/svg+xml", "svg"),
    ("text/csv", "csv"),
    ("video/mp4", "mp4"),
    ("video/mpeg", "mpeg"),
    ("audio/mpeg", "mp3"),
    ("audio/wav", "wav"),
    ("application/pdf", "pdf"),
    ("application/json", "json"),
    ("application/xml", "xml"),
    ("text/xml", "xml"),
    ("application/zip", "zip"),
    ("application/javascript", "js"),
    ("text/css", "css"),
    ("text/html", "html"),
    ("text/plain", "txt"),
];

/// Full declared header value in, extension out. Malformed or unmapped (empty,
/// application/octet-stream, multipart/form-data, ...) -> "bin", which mime_guess maps back to
/// octet-stream: correct for unknown content.
pub fn mime_to_ext(declared: &str) -> String {
    essence(declared)
        .and_then(|e| {
            MIME_TO_EXT
                .iter()
                .find(|(m, _)| *m == e)
                .map(|(_, x)| x.to_string())
        })
        .unwrap_or_else(|| "bin".to_string())
}

/// The parser's strip set: the closed inverse of the map plus "bin". 18 distinct map values
/// (xml appears twice) + bin = 19. Case-sensitive.
pub const STRIP_SET: &[&str; 19] = &[
    "bin", "css", "csv", "gif", "html", "jpg", "js", "json", "mp3", "mp4", "mpeg", "pdf", "png",
    "svg", "txt", "wav", "webp", "xml", "zip",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VALID_MIME_TYPES;

    #[test]
    fn essence_vectors() {
        let cases: &[(&str, Option<&str>)] = &[
            ("image/jpeg", Some("image/jpeg")),
            ("IMAGE/JPEG; charset=x", Some("image/jpeg")),
            ("image/svg+xml", Some("image/svg+xml")),
            ("text/plain;charset=utf-8", Some("text/plain")),
            // No trim, so a leading space is malformed
            (" image/jpeg", None),
            ("image/jpeg ", None),
            // Accepted by the mime crate, rejected here
            ("image/", None),
            ("", None),
            ("noslash", None),
            ("a/b/c", None),
            ("/png", None),
            (";image/png", None),
            ("imagé/png", None),
        ];
        for (input, expected) in cases {
            assert_eq!(essence(input).as_deref(), *expected, "{input}");
        }
    }

    #[test]
    fn mime_to_ext_vectors() {
        for (declared, ext) in MIME_TO_EXT {
            assert_eq!(&mime_to_ext(declared), ext, "{declared}");
        }
        assert_eq!(mime_to_ext("text/csv"), "csv");
        assert_eq!(mime_to_ext("IMAGE/PNG; charset=x"), "png");
        for unmapped in [
            "application/octet-stream",
            "multipart/form-data",
            "application/x-www-form-urlencoded",
            "",
            "garbage",
            " image/png",
        ] {
            assert_eq!(mime_to_ext(unmapped), "bin", "{unmapped}");
        }
    }

    #[test]
    fn every_advisory_type_maps() {
        let bins = VALID_MIME_TYPES
            .iter()
            .filter(|m| mime_to_ext(m) == "bin")
            .count();
        assert_eq!(bins, 3, "octet-stream, form-data and form-urlencoded");
        for declared in VALID_MIME_TYPES {
            assert!(
                STRIP_SET.contains(&mime_to_ext(declared).as_str()),
                "{declared}"
            );
        }
    }

    #[test]
    fn strip_set_is_the_closed_inverse_of_the_map() {
        let mut inverse: Vec<&str> = MIME_TO_EXT.iter().map(|(_, ext)| *ext).collect();
        inverse.push("bin");
        inverse.sort_unstable();
        inverse.dedup();
        assert_eq!(inverse, STRIP_SET.to_vec());
    }
}
