//! Post-processing for markdown output.

use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::FxHashMap;

use crate::options::{LinkStyle, Options};

/// Regex for collapsing excessive newlines.
static EXCESSIVE_NEWLINES_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\n{3,}").unwrap());

/// Regex for matching inline links (not images).
/// Matches [text](url) or [text](url "title") but not ![alt](src)
/// Uses a capture group to detect if preceded by ! (for images)
static INLINE_LINK_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(^|[^!])\[([^\]]+)\]\(([^)\s]+)(?:\s+"([^"]*)")?\)"#).unwrap()
});

/// Post-process the markdown output.
pub fn postprocess(markdown: String, options: &Options) -> String {
    let mut result = markdown;

    // 1. Escape newlines in link text [text\nmore](url) → [text\\nmore](url)
    result = escape_link_newlines(&result);

    // 2. Convert to referenced links if requested
    if matches!(options.link_style, LinkStyle::Referenced) {
        result = convert_to_referenced_links(&result);
    }

    // 3. Collapse 3+ newlines to 2
    result = EXCESSIVE_NEWLINES_RE
        .replace_all(&result, "\n\n")
        .into_owned();

    // 4. Trim trailing whitespace per line
    result = trim_trailing_whitespace(&result);

    // 5. Trim document
    result.trim().to_string()
}

/// Convert inline links to referenced style.
/// [text](url) → [text][1] with [1]: url at document end
fn convert_to_referenced_links(markdown: &str) -> String {
    let mut url_to_ref: FxHashMap<String, usize> = FxHashMap::default();
    let mut references: Vec<(usize, String, Option<String>)> = Vec::new();
    let mut ref_counter = 0;

    // Replace inline links with reference-style
    // Capture groups: 1=prefix (empty or non-!), 2=text, 3=url, 4=title
    let result = INLINE_LINK_RE.replace_all(markdown, |caps: &regex::Captures| {
        let prefix = &caps[1]; // Character before [ (or empty at start)
        let text = &caps[2];
        let url = &caps[3];
        let title = caps.get(4).map(|m| m.as_str().to_string());

        // Check if we've seen this URL before (deduplicate)
        let ref_num = if let Some(&existing_ref) = url_to_ref.get(url) {
            existing_ref
        } else {
            ref_counter += 1;
            url_to_ref.insert(url.to_string(), ref_counter);
            references.push((ref_counter, url.to_string(), title));
            ref_counter
        };

        format!("{}[{}][{}]", prefix, text, ref_num)
    });

    // If no links found, return as-is
    if references.is_empty() {
        return markdown.to_string();
    }

    // Append reference definitions at end
    let mut output = result.into_owned();
    output.push_str("\n\n");

    for (num, url, title) in references {
        match title {
            Some(t) => output.push_str(&format!("[{}]: {} \"{}\"\n", num, url, t)),
            None => output.push_str(&format!("[{}]: {}\n", num, url)),
        }
    }

    output
}

/// Escape newlines inside link text, handling escaped brackets correctly.
fn escape_link_newlines(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut bracket_depth: i32 = 0;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Check for escaped bracket
        if c == '\\' && i + 1 < chars.len() {
            let next = chars[i + 1];
            if next == '[' || next == ']' {
                // Escaped bracket - don't change depth, output both chars
                result.push(c);
                result.push(next);
                i += 2;
                continue;
            }
        }

        match c {
            '[' => {
                bracket_depth += 1;
                result.push(c);
            }
            ']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                result.push(c);
            }
            '\n' if bracket_depth > 0 => {
                result.push_str("\\n");
            }
            _ => result.push(c),
        }
        i += 1;
    }

    result
}

/// Trim trailing whitespace from each line.
fn trim_trailing_whitespace(text: &str) -> String {
    text.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_link_newlines() {
        let input = "[text\nwith newline](url)";
        let result = escape_link_newlines(input);
        assert_eq!(result, "[text\\nwith newline](url)");
    }

    #[test]
    fn test_escape_link_newlines_escaped_bracket() {
        let input = r"text with \[ escaped bracket";
        let result = escape_link_newlines(input);
        assert_eq!(result, r"text with \[ escaped bracket");
    }

    #[test]
    fn test_escape_link_newlines_no_change() {
        let input = "normal text\nwith newline outside brackets";
        let result = escape_link_newlines(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_convert_to_referenced_links() {
        let input = "Check [this](https://a.com) and [that](https://b.com).";
        let result = convert_to_referenced_links(input);
        assert!(result.contains("[this][1]"));
        assert!(result.contains("[that][2]"));
        assert!(result.contains("[1]: https://a.com"));
        assert!(result.contains("[2]: https://b.com"));
    }

    #[test]
    fn test_convert_to_referenced_links_dedup() {
        let input = "[a](https://x.com) and [b](https://x.com)";
        let result = convert_to_referenced_links(input);
        assert!(result.contains("[a][1]"));
        assert!(result.contains("[b][1]")); // Same reference
        // Should only have one reference
        assert_eq!(result.matches("[1]:").count(), 1);
    }

    #[test]
    fn test_convert_to_referenced_links_with_title() {
        let input = r#"[link](https://a.com "Title")"#;
        let result = convert_to_referenced_links(input);
        assert!(result.contains("[link][1]"));
        assert!(result.contains(r#"[1]: https://a.com "Title""#));
    }

    #[test]
    fn test_convert_to_referenced_links_no_images() {
        let input = "![image](img.png) and [link](url)";
        let result = convert_to_referenced_links(input);
        // Image should NOT be converted
        assert!(result.contains("![image](img.png)"));
        // Link should be converted
        assert!(result.contains("[link][1]"));
    }

    #[test]
    fn test_trim_trailing_whitespace() {
        let input = "line 1   \nline 2  \nline 3";
        let result = trim_trailing_whitespace(input);
        assert_eq!(result, "line 1\nline 2\nline 3");
    }

    #[test]
    fn test_collapse_newlines() {
        let input = "a\n\n\n\nb";
        let result = EXCESSIVE_NEWLINES_RE.replace_all(input, "\n\n");
        assert_eq!(result, "a\n\nb");
    }

    #[test]
    fn test_postprocess_full() {
        let input = "# Title\n\n\n\nParagraph   \n";
        let result = postprocess(input.to_string(), &Options::default());
        assert_eq!(result, "# Title\n\nParagraph");
    }
}
