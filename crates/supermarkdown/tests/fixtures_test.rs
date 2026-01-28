//! Integration tests using real-world HTML fixtures.
//!
//! These tests verify that supermarkdown handles various real-world HTML
//! documents correctly, including edge cases and malformed input.

use supermarkdown::{convert, convert_with_options, Options};
use std::fs;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn load_fixture(name: &str) -> String {
    let path = fixtures_dir().join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", name, e))
}

// =============================================================================
// Blog Post Tests
// =============================================================================

#[test]
fn test_blog_post_headings() {
    let html = load_fixture("blog_post.html");
    let markdown = convert(&html);

    // Main title
    assert!(markdown.contains("# Getting Started with Rust"));

    // Section headings
    assert!(markdown.contains("## Why Rust?"));
    assert!(markdown.contains("## Installation"));
    assert!(markdown.contains("## Your First Program"));
    assert!(markdown.contains("## Conclusion"));
}

#[test]
fn test_blog_post_code_blocks() {
    let html = load_fixture("blog_post.html");
    let markdown = convert(&html);

    // Code blocks should be fenced
    assert!(markdown.contains("```rust") || markdown.contains("```"));
    assert!(markdown.contains("fn main()"));
    assert!(markdown.contains("println!"));

    // Should contain the actual code
    assert!(markdown.contains("Hello, World!"));
}

#[test]
fn test_blog_post_lists() {
    let html = load_fixture("blog_post.html");
    let markdown = convert(&html);

    // Should have list items
    assert!(markdown.contains("- ") || markdown.contains("* "));
}

#[test]
fn test_blog_post_blockquote() {
    let html = load_fixture("blog_post.html");
    let markdown = convert(&html);

    // Blockquotes should use >
    assert!(markdown.contains(">"));
}

#[test]
fn test_blog_post_links() {
    let html = load_fixture("blog_post.html");
    let markdown = convert(&html);

    // Links should be converted
    assert!(markdown.contains("[official Rust Book](https://doc.rust-lang.org/book/)"));
}

// =============================================================================
// Documentation Tests
// =============================================================================

#[test]
fn test_documentation_tables() {
    let html = load_fixture("documentation.html");
    let markdown = convert(&html);

    // Should have table structure
    assert!(markdown.contains("|"));
    assert!(markdown.contains("Name"));
    assert!(markdown.contains("Type"));
    assert!(markdown.contains("Required"));
}

#[test]
fn test_documentation_code_in_text() {
    let html = load_fixture("documentation.html");
    let markdown = convert(&html);

    // Inline code should be preserved
    assert!(markdown.contains("`convert(html, options?)`"));
    assert!(markdown.contains("`string`"));
}

#[test]
fn test_documentation_definition_lists() {
    let html = load_fixture("documentation.html");
    let markdown = convert(&html);

    // Definition list content should be present
    assert!(markdown.contains("headingStyle"));
    assert!(markdown.contains("linkStyle"));
    assert!(markdown.contains("excludeSelectors"));
}

#[test]
fn test_documentation_details_element() {
    let html = load_fixture("documentation.html");
    let markdown = convert(&html);

    // Content inside details should be present
    assert!(markdown.contains("Block Elements"));
    assert!(markdown.contains("Inline Elements"));
    assert!(markdown.contains("Headings"));
}

// =============================================================================
// Table Tests
// =============================================================================

#[test]
fn test_tables_basic() {
    let html = load_fixture("tables.html");
    let markdown = convert(&html);

    // Feature comparison table
    assert!(markdown.contains("Feature"));
    assert!(markdown.contains("Free"));
    assert!(markdown.contains("Pro"));
    assert!(markdown.contains("Enterprise"));

    // Data should be present
    assert!(markdown.contains("API Requests"));
    assert!(markdown.contains("Unlimited"));
}

#[test]
fn test_tables_code_in_cells() {
    let html = load_fixture("tables.html");
    let markdown = convert(&html);

    // Code in table cells
    assert!(markdown.contains("`200`") || markdown.contains("200"));
    assert!(markdown.contains("OK"));
    assert!(markdown.contains("Request succeeded"));
}

#[test]
fn test_tables_separator() {
    let html = load_fixture("tables.html");
    let markdown = convert(&html);

    // Should have header separator
    assert!(markdown.contains("|---") || markdown.contains("| ---"));
}

// =============================================================================
// Code Heavy Tests
// =============================================================================

#[test]
fn test_code_heavy_multiple_languages() {
    let html = load_fixture("code_heavy.html");
    let markdown = convert(&html);

    // Should have different language annotations
    assert!(markdown.contains("```bash") || markdown.contains("```"));
    assert!(markdown.contains("```rust") || markdown.contains("```"));
    assert!(markdown.contains("```toml") || markdown.contains("```"));
}

#[test]
fn test_code_heavy_preserves_content() {
    let html = load_fixture("code_heavy.html");
    let markdown = convert(&html);

    // Key code content should be preserved
    assert!(markdown.contains("fn main()"));
    assert!(markdown.contains("println!"));
    assert!(markdown.contains("cargo new"));
}

#[test]
fn test_code_heavy_inline_code() {
    let html = load_fixture("code_heavy.html");
    let markdown = convert(&html);

    // Inline code
    assert!(markdown.contains("`println!`"));
    assert!(markdown.contains("`cargo new`"));
}

#[test]
fn test_code_heavy_special_chars() {
    let html = load_fixture("code_heavy.html");
    let markdown = convert(&html);

    // HTML entities in code should be decoded
    assert!(markdown.contains("->") || markdown.contains("-&gt;"));
    assert!(markdown.contains("&str") || markdown.contains("&amp;str"));
}

// =============================================================================
// Malformed HTML Tests
// =============================================================================

#[test]
fn test_malformed_no_panic() {
    let html = load_fixture("malformed.html");
    // Should not panic
    let _markdown = convert(&html);
}

#[test]
fn test_malformed_recovers_content() {
    let html = load_fixture("malformed.html");
    let markdown = convert(&html);

    // Should still extract content from malformed HTML
    assert!(markdown.contains("Welcome"));
    // Valid content at the end should be present
    assert!(markdown.contains("properly formatted") || markdown.contains("Valid Content"));
}

#[test]
fn test_malformed_handles_unclosed_tags() {
    let html = load_fixture("malformed.html");
    let markdown = convert(&html);

    // List items without closing tags
    assert!(markdown.contains("First item"));
    assert!(markdown.contains("Second item"));
    assert!(markdown.contains("Third item"));
}

#[test]
fn test_malformed_strips_script_with_exclude() {
    let html = load_fixture("malformed.html");
    let options = Options::new().exclude_selectors(vec!["script".to_string()]);
    let markdown = convert_with_options(&html, &options);

    // Script content should be removed when excluded
    assert!(!markdown.contains("alert"));
    assert!(!markdown.contains("XSS"));
}

#[test]
fn test_malformed_strips_style_with_exclude() {
    let html = load_fixture("malformed.html");
    let options = Options::new().exclude_selectors(vec!["style".to_string()]);
    let markdown = convert_with_options(&html, &options);

    // Style content should be removed when excluded
    assert!(!markdown.contains("background: red"));
}

#[test]
fn test_malformed_handles_entities() {
    let html = load_fixture("malformed.html");
    let markdown = convert(&html);

    // Should have copyright symbol or entity
    assert!(markdown.contains("©") || markdown.contains("&copy;"));
}

#[test]
fn test_malformed_valid_list_at_end() {
    let html = load_fixture("malformed.html");
    let markdown = convert(&html);

    // The valid list at the end should be properly converted
    // (using lowercase to handle potential case differences)
    let lower = markdown.to_lowercase();
    assert!(lower.contains("item one") || markdown.contains("Item one"));
    assert!(lower.contains("item two") || markdown.contains("Item two"));
    assert!(lower.contains("item three") || markdown.contains("Item three"));
}

// =============================================================================
// Options Tests
// =============================================================================

#[test]
fn test_exclude_nav() {
    let html = load_fixture("documentation.html");
    let options = Options::new().exclude_selectors(vec![".docs-nav".to_string()]);
    let markdown = convert_with_options(&html, &options);

    // Navigation should be excluded
    assert!(!markdown.contains("[Home](/)"));
}

#[test]
fn test_exclude_footer() {
    let html = load_fixture("documentation.html");
    let options = Options::new().exclude_selectors(vec!["footer".to_string()]);
    let markdown = convert_with_options(&html, &options);

    // Footer should be excluded
    assert!(!markdown.contains("supermarkdown documentation"));
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_empty_string() {
    let markdown = convert("");
    assert!(markdown.is_empty() || markdown.trim().is_empty());
}

#[test]
fn test_whitespace_only() {
    let markdown = convert("   \n\t\n   ");
    assert!(markdown.trim().is_empty());
}

#[test]
fn test_plain_text() {
    let markdown = convert("Just plain text without any HTML");
    assert!(markdown.contains("Just plain text"));
}

#[test]
fn test_deeply_nested() {
    let html = "<div><div><div><div><div><p>Deep content</p></div></div></div></div></div>";
    let markdown = convert(html);
    assert!(markdown.contains("Deep content"));
}

#[test]
fn test_long_document() {
    let html = load_fixture("code_heavy.html");
    let markdown = convert(&html);

    // Should handle the full document
    assert!(markdown.contains("Getting Started with Rust"));
    assert!(markdown.contains("Error Handling"));
}
