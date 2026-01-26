//! Whitespace normalization utilities.

#![allow(dead_code)] // Utility functions available for extensibility

use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for collapsing multiple spaces/tabs (preserves newlines).
static INLINE_WS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[ \t]+").unwrap());

/// Regex for collapsing all whitespace including newlines.
static ALL_WS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

/// Normalize whitespace for inline elements.
///
/// Collapses multiple spaces/tabs to single space, preserves newlines.
pub fn normalize_inline_whitespace(text: &str) -> String {
    INLINE_WS_RE.replace_all(text, " ").into_owned()
}

/// Normalize whitespace for block elements.
///
/// Collapses all whitespace including newlines to single space.
pub fn normalize_block_whitespace(text: &str) -> String {
    ALL_WS_RE.replace_all(text, " ").into_owned()
}

/// Trim leading/trailing whitespace and return the trimmed content with
/// the leading and trailing whitespace preserved separately.
///
/// Useful for handling cases like `<em> text </em>` → preserving surrounding spaces.
pub fn trim_inline_content(text: &str) -> (&str, &str, &str) {
    let start_len = text.len() - text.trim_start().len();
    let end_len = text.len() - text.trim_end().len();
    let leading = &text[..start_len];
    let trailing = &text[text.len() - end_len..];
    let trimmed = text.trim();
    (leading, trimmed, trailing)
}

/// Check if a string contains only whitespace.
pub fn is_whitespace_only(text: &str) -> bool {
    text.chars().all(|c| c.is_whitespace())
}

/// Collapse consecutive newlines to a maximum of 2.
pub fn collapse_newlines(text: &str) -> String {
    static NEWLINES_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\n{3,}").unwrap());
    NEWLINES_RE.replace_all(text, "\n\n").into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_inline_whitespace() {
        assert_eq!(normalize_inline_whitespace("hello   world"), "hello world");
        assert_eq!(normalize_inline_whitespace("a\t\tb"), "a b");
        assert_eq!(normalize_inline_whitespace("  leading"), " leading");
        // Preserves newlines
        assert_eq!(normalize_inline_whitespace("line1\nline2"), "line1\nline2");
    }

    #[test]
    fn test_normalize_block_whitespace() {
        assert_eq!(normalize_block_whitespace("hello   world"), "hello world");
        assert_eq!(normalize_block_whitespace("line1\n\nline2"), "line1 line2");
        assert_eq!(
            normalize_block_whitespace("  spaced  out  "),
            " spaced out "
        );
    }

    #[test]
    fn test_trim_inline_content() {
        let (leading, trimmed, trailing) = trim_inline_content("  hello  ");
        assert_eq!(leading, "  ");
        assert_eq!(trimmed, "hello");
        assert_eq!(trailing, "  ");

        let (leading, trimmed, trailing) = trim_inline_content("no spaces");
        assert_eq!(leading, "");
        assert_eq!(trimmed, "no spaces");
        assert_eq!(trailing, "");
    }

    #[test]
    fn test_is_whitespace_only() {
        assert!(is_whitespace_only("   "));
        assert!(is_whitespace_only("\t\n"));
        assert!(is_whitespace_only(""));
        assert!(!is_whitespace_only("a"));
        assert!(!is_whitespace_only(" a "));
    }

    #[test]
    fn test_collapse_newlines() {
        assert_eq!(collapse_newlines("a\n\n\n\nb"), "a\n\nb");
        assert_eq!(collapse_newlines("a\n\nb"), "a\n\nb");
        assert_eq!(collapse_newlines("a\nb"), "a\nb");
    }
}
