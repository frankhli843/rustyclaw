/// The current version of rustyclaw, read from Cargo.toml at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_not_empty() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn version_has_semver_format() {
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert!(parts.len() >= 2, "Version should have at least major.minor");
        for part in &parts {
            assert!(part.parse::<u32>().is_ok(), "Version part '{}' should be numeric", part);
        }
    }
}
