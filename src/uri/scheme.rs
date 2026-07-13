use crate::PROTOCOL;

/// Returns true when `scheme` is `pubky` (case-insensitive).
pub fn is_pubky_scheme(scheme: &str) -> bool {
    scheme.eq_ignore_ascii_case(PROTOCOL.trim_end_matches("://"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_pubky_scheme() {
        assert!(is_pubky_scheme("pubky"));
        assert!(is_pubky_scheme("PUBKY"));
        assert!(is_pubky_scheme("Pubky"));
        assert!(!is_pubky_scheme("https"));
    }
}
