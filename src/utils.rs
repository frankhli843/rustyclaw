use std::path::{Path, PathBuf};

/// Clamp a number between min and max.
pub fn clamp_number(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

/// Clamp an integer between min and max (floors the value first).
pub fn clamp_int(value: f64, min: i64, max: i64) -> i64 {
    (value.floor() as i64).max(min).min(max)
}

/// Escape special regex characters in a string.
pub fn escape_regexp(value: &str) -> String {
    let special = r".*+?^${}()|[]\";
    let mut result = String::with_capacity(value.len() * 2);
    for ch in value.chars() {
        if special.contains(ch) {
            result.push('\\');
        }
        result.push(ch);
    }
    result
}

/// Safely parse JSON, returning None on error.
pub fn safe_parse_json<T: serde::de::DeserializeOwned>(raw: &str) -> Option<T> {
    serde_json::from_str(raw).ok()
}

/// Check if a value is a "plain object" (JSON object, not array/null).
pub fn is_plain_object(value: &serde_json::Value) -> bool {
    value.is_object()
}

/// Normalize a path by ensuring it starts with '/'.
pub fn normalize_path(p: &str) -> String {
    if !p.starts_with('/') {
        format!("/{}", p)
    } else {
        p.to_string()
    }
}

/// Add whatsapp: prefix if not already present.
pub fn with_whatsapp_prefix(number: &str) -> String {
    if number.starts_with("whatsapp:") {
        number.to_string()
    } else {
        format!("whatsapp:{}", number)
    }
}

/// Normalize a phone number to E.164 format.
pub fn normalize_e164(number: &str) -> String {
    let without_prefix = number.trim_start_matches("whatsapp:").trim();
    let digits: String = without_prefix.chars().filter(|c| c.is_ascii_digit() || *c == '+').collect();
    if digits.starts_with('+') {
        format!("+{}", &digits[1..])
    } else {
        format!("+{}", digits)
    }
}

/// Convert a phone number to a WhatsApp JID.
pub fn to_whatsapp_jid(number: &str) -> String {
    let without_prefix = number.trim_start_matches("whatsapp:").trim();
    if without_prefix.contains('@') {
        return without_prefix.to_string();
    }
    let e164 = normalize_e164(without_prefix);
    let digits: String = e164.chars().filter(|c| c.is_ascii_digit()).collect();
    format!("{}@s.whatsapp.net", digits)
}

/// Check if "self-chat mode" is active (bot and owner are the same WhatsApp identity).
pub fn is_self_chat_mode(self_e164: Option<&str>, allow_from: Option<&[String]>) -> bool {
    let self_e164 = match self_e164 {
        Some(s) if !s.is_empty() => s,
        _ => return false,
    };
    let allow_from = match allow_from {
        Some(af) if !af.is_empty() => af,
        _ => return false,
    };
    let normalized_self = normalize_e164(self_e164);
    allow_from.iter().any(|n| {
        if n == "*" {
            return false;
        }
        normalize_e164(n) == normalized_self
    })
}

/// Convert a WhatsApp JID to E.164 format.
pub fn jid_to_e164(jid: &str) -> Option<String> {
    // Match standard JIDs: digits[:device]@s.whatsapp.net or @hosted
    let re = regex::Regex::new(r"^(\d+)(?::\d+)?@(s\.whatsapp\.net|hosted)$").unwrap();
    if let Some(caps) = re.captures(jid) {
        let digits = &caps[1];
        return Some(format!("+{}", digits));
    }
    None
}

/// Resolve the OpenClaw config directory.
pub fn resolve_config_dir() -> PathBuf {
    if let Ok(override_dir) = std::env::var("OPENCLAW_STATE_DIR") {
        let trimmed = override_dir.trim();
        if !trimmed.is_empty() {
            return resolve_user_path(trimmed);
        }
    }
    if let Ok(override_dir) = std::env::var("CLAWDBOT_STATE_DIR") {
        let trimmed = override_dir.trim();
        if !trimmed.is_empty() {
            return resolve_user_path(trimmed);
        }
    }
    let home = resolve_home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".openclaw")
}

/// Resolve the home directory, preferring OPENCLAW_HOME.
pub fn resolve_home_dir() -> Option<PathBuf> {
    if let Ok(openclaw_home) = std::env::var("OPENCLAW_HOME") {
        let trimmed = openclaw_home.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed).canonicalize().unwrap_or_else(|_| PathBuf::from(trimmed)));
        }
    }
    dirs::home_dir()
}

/// Resolve a user path, expanding ~ to home directory.
pub fn resolve_user_path(input: &str) -> PathBuf {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return PathBuf::new();
    }
    if trimmed.starts_with('~') {
        let home = resolve_home_dir().unwrap_or_else(|| PathBuf::from("."));
        let rest = trimmed.trim_start_matches('~').trim_start_matches('/');
        if rest.is_empty() {
            return home;
        }
        return home.join(rest);
    }
    let path = Path::new(trimmed);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(path)
    }
}

/// Shorten a path by replacing the home directory with ~.
pub fn shorten_home_path(input: &str) -> String {
    if input.is_empty() {
        return input.to_string();
    }
    if let Some(home) = resolve_home_dir() {
        let home_str = home.to_string_lossy();
        let prefix = if std::env::var("OPENCLAW_HOME").is_ok() {
            "$OPENCLAW_HOME"
        } else {
            "~"
        };
        if input == home_str.as_ref() {
            return prefix.to_string();
        }
        if input.starts_with(&format!("{}/", home_str)) || input.starts_with(&format!("{}\\", home_str)) {
            return format!("{}{}", prefix, &input[home_str.len()..]);
        }
    }
    input.to_string()
}

/// Replace all occurrences of the home directory in a string with ~ or $OPENCLAW_HOME.
pub fn shorten_home_in_string(input: &str) -> String {
    if input.is_empty() {
        return input.to_string();
    }
    if let Some(home) = resolve_home_dir() {
        let home_str = home.to_string_lossy().to_string();
        let prefix = if std::env::var("OPENCLAW_HOME").is_ok() {
            "$OPENCLAW_HOME"
        } else {
            "~"
        };
        return input.replace(&home_str, prefix);
    }
    input.to_string()
}

/// UTF-16 safe string slicing (avoids splitting surrogate pairs).
pub fn slice_utf16_safe(input: &str, start: usize, end: Option<usize>) -> String {
    let utf16: Vec<u16> = input.encode_utf16().collect();
    let len = utf16.len();
    let mut from = start.min(len);
    let mut to = end.unwrap_or(len).min(len);

    if to < from {
        std::mem::swap(&mut from, &mut to);
    }

    // Avoid splitting surrogate pairs
    if from > 0 && from < len {
        if is_low_surrogate(utf16[from]) && from > 0 && is_high_surrogate(utf16[from - 1]) {
            from += 1;
        }
    }
    if to > 0 && to < len {
        if is_high_surrogate(utf16[to - 1]) && is_low_surrogate(utf16[to]) {
            to -= 1;
        }
    }

    String::from_utf16_lossy(&utf16[from..to])
}

/// Truncate a string safely at UTF-16 boundaries.
pub fn truncate_utf16_safe(input: &str, max_len: usize) -> String {
    let utf16: Vec<u16> = input.encode_utf16().collect();
    if utf16.len() <= max_len {
        return input.to_string();
    }
    slice_utf16_safe(input, 0, Some(max_len))
}

fn is_high_surrogate(code_unit: u16) -> bool {
    (0xD800..=0xDBFF).contains(&code_unit)
}

fn is_low_surrogate(code_unit: u16) -> bool {
    (0xDC00..=0xDFFF).contains(&code_unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_path_adds_leading_slash() {
        assert_eq!(normalize_path("foo"), "/foo");
    }

    #[test]
    fn normalize_path_keeps_existing_slash() {
        assert_eq!(normalize_path("/bar"), "/bar");
    }

    #[test]
    fn with_whatsapp_prefix_adds_prefix() {
        assert_eq!(with_whatsapp_prefix("+1555"), "whatsapp:+1555");
    }

    #[test]
    fn with_whatsapp_prefix_leaves_prefixed_intact() {
        assert_eq!(with_whatsapp_prefix("whatsapp:+1555"), "whatsapp:+1555");
    }

    #[test]
    fn normalize_e164_strips_formatting_and_prefixes() {
        assert_eq!(normalize_e164("whatsapp:(555) 123-4567"), "+5551234567");
    }

    #[test]
    fn to_whatsapp_jid_converts_number() {
        assert_eq!(to_whatsapp_jid("whatsapp:+555 123 4567"), "5551234567@s.whatsapp.net");
    }

    #[test]
    fn to_whatsapp_jid_preserves_existing_jids() {
        assert_eq!(to_whatsapp_jid("123456789-987654321@g.us"), "123456789-987654321@g.us");
        assert_eq!(to_whatsapp_jid("whatsapp:123456789-987654321@g.us"), "123456789-987654321@g.us");
        assert_eq!(to_whatsapp_jid("1555123@s.whatsapp.net"), "1555123@s.whatsapp.net");
    }

    #[test]
    fn jid_to_e164_standard_jid() {
        assert_eq!(jid_to_e164("1555000:2@hosted"), Some("+1555000".to_string()));
        assert_eq!(jid_to_e164("12345@s.whatsapp.net"), Some("+12345".to_string()));
    }

    #[test]
    fn clamp_number_works() {
        assert_eq!(clamp_number(5.0, 0.0, 10.0), 5.0);
        assert_eq!(clamp_number(-1.0, 0.0, 10.0), 0.0);
        assert_eq!(clamp_number(15.0, 0.0, 10.0), 10.0);
    }

    #[test]
    fn clamp_int_works() {
        assert_eq!(clamp_int(5.7, 0, 10), 5);
        assert_eq!(clamp_int(-1.0, 0, 10), 0);
        assert_eq!(clamp_int(15.0, 0, 10), 10);
    }

    #[test]
    fn escape_regexp_works() {
        assert_eq!(escape_regexp("hello.world"), "hello\\.world");
        assert_eq!(escape_regexp("a+b"), "a\\+b");
    }

    #[test]
    fn safe_parse_json_works() {
        let result: Option<serde_json::Value> = safe_parse_json(r#"{"key": "value"}"#);
        assert!(result.is_some());
        let result: Option<serde_json::Value> = safe_parse_json("not json");
        assert!(result.is_none());
    }

    #[test]
    fn resolve_user_path_blank() {
        assert_eq!(resolve_user_path(""), PathBuf::new());
        assert_eq!(resolve_user_path("   "), PathBuf::new());
    }

    #[test]
    fn resolve_user_path_tilde() {
        let result = resolve_user_path("~");
        assert!(result.to_string_lossy().len() > 1);
    }

    #[test]
    fn resolve_user_path_tilde_subdir() {
        let result = resolve_user_path("~/openclaw");
        assert!(result.to_string_lossy().contains("openclaw"));
    }

    #[test]
    fn resolve_user_path_relative() {
        let result = resolve_user_path("tmp/dir");
        assert!(result.is_absolute());
    }

    #[test]
    fn is_self_chat_mode_works() {
        assert!(!is_self_chat_mode(None, None));
        assert!(!is_self_chat_mode(Some("+1555"), None));
        assert!(!is_self_chat_mode(Some("+1555"), Some(&[])));
        assert!(is_self_chat_mode(Some("+1555"), Some(&["+1555".to_string()])));
        assert!(!is_self_chat_mode(Some("+1555"), Some(&["*".to_string()])));
    }

    #[test]
    fn slice_utf16_safe_basic() {
        assert_eq!(slice_utf16_safe("hello", 0, Some(3)), "hel");
        assert_eq!(slice_utf16_safe("hello", 2, None), "llo");
    }

    #[test]
    fn truncate_utf16_safe_no_truncation() {
        assert_eq!(truncate_utf16_safe("hello", 10), "hello");
    }

    #[test]
    fn truncate_utf16_safe_truncates() {
        assert_eq!(truncate_utf16_safe("hello world", 5), "hello");
    }
}
