use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ByteSizeError {
    #[error("invalid byte size (empty)")]
    Empty,
    #[error("invalid byte size: {0}")]
    Invalid(String),
    #[error("invalid byte size unit: {0}")]
    InvalidUnit(String),
}

/// Parse a byte size string (e.g., "10kb", "1mb", "2gb") into bytes.
pub fn parse_byte_size(raw: &str) -> Result<u64, ByteSizeError> {
    let trimmed = raw.trim().to_lowercase();
    if trimmed.is_empty() {
        return Err(ByteSizeError::Empty);
    }

    let re = Regex::new(r"^(\d+(?:\.\d+)?)([a-z]+)?$").unwrap();
    let caps = re.captures(&trimmed).ok_or_else(|| ByteSizeError::Invalid(raw.to_string()))?;

    let value: f64 = caps[1].parse().map_err(|_| ByteSizeError::Invalid(raw.to_string()))?;
    if !value.is_finite() || value < 0.0 {
        return Err(ByteSizeError::Invalid(raw.to_string()));
    }

    let unit = caps.get(2).map(|m| m.as_str()).unwrap_or("b");

    let multiplier: f64 = match unit {
        "b" => 1.0,
        "kb" | "k" => 1024.0,
        "mb" | "m" => 1024.0_f64.powi(2),
        "gb" | "g" => 1024.0_f64.powi(3),
        "tb" | "t" => 1024.0_f64.powi(4),
        _ => return Err(ByteSizeError::InvalidUnit(raw.to_string())),
    };

    let bytes = (value * multiplier).round() as u64;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bytes_with_units() {
        assert_eq!(parse_byte_size("10kb").unwrap(), 10 * 1024);
        assert_eq!(parse_byte_size("1mb").unwrap(), 1024 * 1024);
        assert_eq!(parse_byte_size("2gb").unwrap(), 2 * 1024 * 1024 * 1024);
    }

    #[test]
    fn parses_shorthand_units() {
        assert_eq!(parse_byte_size("5k").unwrap(), 5 * 1024);
        assert_eq!(parse_byte_size("1m").unwrap(), 1024 * 1024);
    }

    #[test]
    fn uses_default_unit_when_omitted() {
        assert_eq!(parse_byte_size("123").unwrap(), 123);
    }

    #[test]
    fn rejects_invalid_values() {
        assert!(parse_byte_size("").is_err());
        assert!(parse_byte_size("nope").is_err());
        assert!(parse_byte_size("-5kb").is_err());
    }
}
