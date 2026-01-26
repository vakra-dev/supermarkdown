//! Markdown escaping utilities.

#![allow(dead_code)] // Utility functions available for extensibility

/// Escape special markdown characters in text.
///
/// Characters escaped: \ ` * _ { } [ ] ( ) # + - . ! |
pub fn escape_markdown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' | '`' | '*' | '_' | '{' | '}' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '.'
            | '!' | '|' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }
    result
}

/// Escape characters in link/image titles (within quotes).
///
/// Escapes: " and \
pub fn escape_title(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '"' | '\\' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }
    result
}

/// Escape characters in link URLs.
///
/// Escapes parentheses and spaces.
pub fn escape_url(url: &str) -> String {
    let mut result = String::with_capacity(url.len());
    for c in url.chars() {
        match c {
            '(' => result.push_str("%28"),
            ')' => result.push_str("%29"),
            ' ' => result.push_str("%20"),
            _ => result.push(c),
        }
    }
    result
}

/// Escape pipe characters for table cells.
pub fn escape_table_cell(text: &str) -> String {
    text.replace('|', "\\|")
}

/// Escape backticks in inline code.
///
/// Returns the appropriate number of backticks to use as delimiters.
pub fn calculate_code_backticks(code: &str) -> usize {
    let mut max_run = 0;
    let mut current_run = 0;

    for c in code.chars() {
        if c == '`' {
            current_run += 1;
            max_run = max_run.max(current_run);
        } else {
            current_run = 0;
        }
    }

    if max_run == 0 {
        1
    } else {
        max_run + 1
    }
}

/// Resolve a relative URL against a base URL.
pub fn resolve_url(base: &str, relative: &str) -> String {
    // If the URL is already absolute, return as-is
    if relative.starts_with("http://")
        || relative.starts_with("https://")
        || relative.starts_with("//")
        || relative.starts_with("mailto:")
        || relative.starts_with("tel:")
        || relative.starts_with("data:")
    {
        return relative.to_string();
    }

    if relative.starts_with('/') {
        // Absolute path - combine with base origin
        if let Some(protocol_end) = base.find("://") {
            let after_protocol = &base[protocol_end + 3..];
            let origin_end = after_protocol
                .find('/')
                .map(|j| protocol_end + 3 + j)
                .unwrap_or(base.len());
            format!("{}{}", &base[..origin_end], relative)
        } else {
            format!("{}{}", base, relative)
        }
    } else if relative.starts_with('#') || relative.starts_with('?') {
        // Fragment or query - append to base (without trailing slash)
        let base = base.trim_end_matches('/');
        format!("{}{}", base, relative)
    } else {
        // Relative path - combine with base directory
        // If base ends with /, it's a directory - append directly
        // Otherwise, find the last / and append to that directory
        if base.ends_with('/') {
            format!("{}{}", base, relative)
        } else if let Some(last_slash) = base.rfind('/') {
            format!("{}/{}", &base[..last_slash], relative)
        } else {
            format!("{}/{}", base, relative)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_markdown() {
        assert_eq!(escape_markdown("*bold*"), "\\*bold\\*");
        assert_eq!(escape_markdown("_italic_"), "\\_italic\\_");
        assert_eq!(escape_markdown("[link]"), "\\[link\\]");
        assert_eq!(escape_markdown("# heading"), "\\# heading");
    }

    #[test]
    fn test_escape_title() {
        assert_eq!(escape_title(r#"Hello "World""#), r#"Hello \"World\""#);
        assert_eq!(escape_title(r"path\to\file"), r"path\\to\\file");
    }

    #[test]
    fn test_escape_url() {
        assert_eq!(escape_url("url (with parens)"), "url%20%28with%20parens%29");
    }

    #[test]
    fn test_escape_table_cell() {
        assert_eq!(escape_table_cell("a | b"), "a \\| b");
    }

    #[test]
    fn test_calculate_code_backticks() {
        assert_eq!(calculate_code_backticks("no backticks"), 1);
        assert_eq!(calculate_code_backticks("one ` backtick"), 2);
        assert_eq!(calculate_code_backticks("two `` backticks"), 3);
        assert_eq!(calculate_code_backticks("```"), 4);
    }

    #[test]
    fn test_resolve_url() {
        // Absolute URLs unchanged
        assert_eq!(
            resolve_url("https://example.com", "https://other.com/page"),
            "https://other.com/page"
        );

        // Absolute paths
        assert_eq!(
            resolve_url("https://example.com/dir/page", "/other"),
            "https://example.com/other"
        );

        // Relative paths
        assert_eq!(
            resolve_url("https://example.com/dir/page", "other"),
            "https://example.com/dir/other"
        );

        // Fragments
        assert_eq!(
            resolve_url("https://example.com/page", "#section"),
            "https://example.com/page#section"
        );
    }
}
