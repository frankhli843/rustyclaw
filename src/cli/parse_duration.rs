use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DurationError {
    #[error("invalid duration (empty)")]
    Empty,
    #[error("invalid duration: {0}")]
    Invalid(String),
}

/// Default unit when none is specified.
#[derive(Debug, Clone, Copy)]
pub enum DurationUnit {
    Ms,
    S,
    M,
    H,
    D,
}

/// Parse a duration string (e.g., "10s", "1m", "2h", "500ms", "2d") into milliseconds.
pub fn parse_duration_ms(raw: &str, default_unit: Option<DurationUnit>) -> Result<u64, DurationError> {
    let trimmed = raw.trim().to_lowercase();
    if trimmed.is_empty() {
        return Err(DurationError::Empty);
    }

    let re = Regex::new(r"^(\d+(?:\.\d+)?)(ms|s|m|h|d)?$").unwrap();
    let caps = re.captures(&trimmed).ok_or_else(|| DurationError::Invalid(raw.to_string()))?;

    let value: f64 = caps[1].parse().map_err(|_| DurationError::Invalid(raw.to_string()))?;
    if !value.is_finite() || value < 0.0 {
        return Err(DurationError::Invalid(raw.to_string()));
    }

    let unit = caps.get(2).map(|m| m.as_str()).unwrap_or(match default_unit.unwrap_or(DurationUnit::Ms) {
        DurationUnit::Ms => "ms",
        DurationUnit::S => "s",
        DurationUnit::M => "m",
        DurationUnit::H => "h",
        DurationUnit::D => "d",
    });

    let multiplier: f64 = match unit {
        "ms" => 1.0,
        "s" => 1_000.0,
        "m" => 60_000.0,
        "h" => 3_600_000.0,
        "d" => 86_400_000.0,
        _ => return Err(DurationError::Invalid(raw.to_string())),
    };

    let ms = (value * multiplier).round() as u64;
    Ok(ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bare_ms() {
        assert_eq!(parse_duration_ms("10000", None).unwrap(), 10_000);
    }

    #[test]
    fn parses_seconds_suffix() {
        assert_eq!(parse_duration_ms("10s", None).unwrap(), 10_000);
    }

    #[test]
    fn parses_minutes_suffix() {
        assert_eq!(parse_duration_ms("1m", None).unwrap(), 60_000);
    }

    #[test]
    fn parses_hours_suffix() {
        assert_eq!(parse_duration_ms("2h", None).unwrap(), 7_200_000);
    }

    #[test]
    fn parses_days_suffix() {
        assert_eq!(parse_duration_ms("2d", None).unwrap(), 172_800_000);
    }

    #[test]
    fn supports_decimals() {
        assert_eq!(parse_duration_ms("0.5s", None).unwrap(), 500);
    }
}
