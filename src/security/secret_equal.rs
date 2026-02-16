use subtle::ConstantTimeEq;

/// Constant-time comparison of two secrets. Returns false if either is None/empty.
pub fn safe_equal_secret(provided: Option<&str>, expected: Option<&str>) -> bool {
    let (provided, expected) = match (provided, expected) {
        (Some(p), Some(e)) => (p, e),
        _ => return false,
    };
    if provided.len() != expected.len() {
        return false;
    }
    provided.as_bytes().ct_eq(expected.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_identical_secrets() {
        assert!(safe_equal_secret(Some("secret-token"), Some("secret-token")));
    }

    #[test]
    fn rejects_mismatched_secrets() {
        assert!(!safe_equal_secret(Some("secret-token"), Some("secret-tokEn")));
    }

    #[test]
    fn rejects_different_length_secrets() {
        assert!(!safe_equal_secret(Some("short"), Some("much-longer")));
    }

    #[test]
    fn rejects_missing_values() {
        assert!(!safe_equal_secret(None, Some("secret")));
        assert!(!safe_equal_secret(Some("secret"), None));
    }
}
