use regex::Regex;

/// Convert standard Markdown formatting to WhatsApp-compatible markup.
///
/// WhatsApp uses its own formatting syntax:
///   bold:          *text*
///   italic:        _text_
///   strikethrough: ~text~
///   monospace:     ```text```
///
/// The conversion preserves fenced code blocks and inline code,
/// then converts bold and strikethrough markers.
pub fn markdown_to_whatsapp(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    const FENCE_PLACEHOLDER: &str = "\x00FENCE";
    const INLINE_CODE_PLACEHOLDER: &str = "\x00CODE";

    // 1. Extract and protect fenced code blocks
    let mut fences: Vec<String> = Vec::new();
    let fence_re = Regex::new(r"(?s)```.*?```").unwrap();
    let result = fence_re.replace_all(text, |caps: &regex::Captures| {
        fences.push(caps[0].to_string());
        format!("{}{}", FENCE_PLACEHOLDER, fences.len() - 1)
    }).to_string();

    // 2. Extract and protect inline code
    let mut inline_codes: Vec<String> = Vec::new();
    let inline_re = Regex::new(r"`[^`\n]+`").unwrap();
    let result = inline_re.replace_all(&result, |caps: &regex::Captures| {
        inline_codes.push(caps[0].to_string());
        format!("{}{}", INLINE_CODE_PLACEHOLDER, inline_codes.len() - 1)
    }).to_string();

    // 3. Convert markdown headers (## Header) → *Header* (WhatsApp bold)
    let header_re = Regex::new(r"(?m)^#{1,6}\s+(.+)$").unwrap();
    let result = header_re.replace_all(&result, "*$1*").to_string();

    // 4. Convert **bold** → *bold* and __bold__ → *bold*
    let bold_star_re = Regex::new(r"\*\*(.+?)\*\*").unwrap();
    let result = bold_star_re.replace_all(&result, "*$1*").to_string();
    let bold_under_re = Regex::new(r"__(.+?)__").unwrap();
    let result = bold_under_re.replace_all(&result, "*$1*").to_string();

    // 5. Convert ~~strikethrough~~ → ~strikethrough~
    let strike_re = Regex::new(r"~~(.+?)~~").unwrap();
    let result = strike_re.replace_all(&result, "~$1~").to_string();

    // 6. Restore inline code
    let mut result = result;
    for (i, code) in inline_codes.iter().enumerate() {
        result = result.replace(&format!("{}{}", INLINE_CODE_PLACEHOLDER, i), code);
    }

    // 7. Restore fenced code blocks
    for (i, fence) in fences.iter().enumerate() {
        result = result.replace(&format!("{}{}", FENCE_PLACEHOLDER, i), fence);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_bold_star_to_whatsapp_bold() {
        assert_eq!(markdown_to_whatsapp("**SOD Blast:**"), "*SOD Blast:*");
    }

    #[test]
    fn converts_bold_underscore_to_whatsapp_bold() {
        assert_eq!(markdown_to_whatsapp("__important__"), "*important*");
    }

    #[test]
    fn converts_strikethrough() {
        assert_eq!(markdown_to_whatsapp("~~deleted~~"), "~deleted~");
    }

    #[test]
    fn leaves_single_star_italic_unchanged() {
        assert_eq!(markdown_to_whatsapp("*text*"), "*text*");
    }

    #[test]
    fn leaves_underscore_italic_unchanged() {
        assert_eq!(markdown_to_whatsapp("_text_"), "_text_");
    }

    #[test]
    fn preserves_fenced_code_blocks() {
        let input = "```\nconst x = **bold**;\n```";
        assert_eq!(markdown_to_whatsapp(input), input);
    }

    #[test]
    fn preserves_inline_code() {
        assert_eq!(
            markdown_to_whatsapp("Use `**not bold**` here"),
            "Use `**not bold**` here"
        );
    }

    #[test]
    fn handles_mixed_formatting() {
        assert_eq!(
            markdown_to_whatsapp("**bold** and ~~strike~~ and _italic_"),
            "*bold* and ~strike~ and _italic_"
        );
    }

    #[test]
    fn handles_multiple_bold_segments() {
        assert_eq!(markdown_to_whatsapp("**one** then **two**"), "*one* then *two*");
    }

    #[test]
    fn returns_empty_for_empty_input() {
        assert_eq!(markdown_to_whatsapp(""), "");
    }

    #[test]
    fn returns_plain_text_unchanged() {
        assert_eq!(markdown_to_whatsapp("no formatting here"), "no formatting here");
    }

    #[test]
    fn handles_bold_inside_sentence() {
        assert_eq!(
            markdown_to_whatsapp("This is **very** important"),
            "This is *very* important"
        );
    }

    #[test]
    fn preserves_code_block_with_formatting_inside() {
        let input = "Before ```**bold** and ~~strike~~``` after **real bold**";
        assert_eq!(
            markdown_to_whatsapp(input),
            "Before ```**bold** and ~~strike~~``` after *real bold*"
        );
    }
}
