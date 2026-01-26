//! # Supermarkdown
//!
//! High-performance HTML to Markdown conversion library for LLMs.
//!
//! ## Quick Start
//!
//! ```rust
//! use supermarkdown::convert;
//!
//! let html = "<h1>Hello</h1><p>World</p>";
//! let markdown = convert(html);
//! assert!(markdown.contains("# Hello"));
//! ```
//!
//! ## With Options
//!
//! ```rust
//! use supermarkdown::{convert_with_options, Options, HeadingStyle};
//!
//! let html = "<h1>Hello</h1>";
//! let options = Options::new()
//!     .heading_style(HeadingStyle::Setext)
//!     .exclude_selectors(vec![".ad".to_string(), "#sidebar".to_string()]);
//!
//! let markdown = convert_with_options(html, &options);
//! ```

mod converter;
mod entities;
mod escape;
mod options;
mod postprocess;
mod precompute;
mod whitespace;

pub mod rules;

pub use converter::Converter;
pub use options::{HeadingStyle, LinkStyle, Options};

/// Convert HTML to Markdown with default options.
///
/// Returns markdown string; malformed HTML is handled gracefully.
///
/// # Example
///
/// ```rust
/// use supermarkdown::convert;
///
/// let markdown = convert("<h1>Title</h1><p>Content</p>");
/// assert!(markdown.contains("# Title"));
/// ```
pub fn convert(html: &str) -> String {
    convert_with_options(html, &Options::default())
}

/// Convert HTML to Markdown with custom options.
///
/// Returns markdown string; never panics on invalid input.
///
/// # Example
///
/// ```rust
/// use supermarkdown::{convert_with_options, Options};
///
/// let options = Options::new()
///     .exclude_selectors(vec![".navigation".to_string()]);
///
/// let markdown = convert_with_options("<div class='navigation'>Nav</div><p>Content</p>", &options);
/// assert!(!markdown.contains("Nav"));
/// ```
pub fn convert_with_options(html: &str, options: &Options) -> String {
    let converter = Converter::new();
    converter.convert(html, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_conversion() {
        let html = "<h1>Hello World</h1>";
        let markdown = convert(html);
        assert!(markdown.contains("# Hello World"));
    }

    #[test]
    fn test_empty_input() {
        let markdown = convert("");
        assert!(markdown.is_empty());
    }

    #[test]
    fn test_paragraph() {
        let html = "<p>This is a paragraph.</p>";
        let markdown = convert(html);
        assert!(markdown.contains("This is a paragraph."));
    }

    #[test]
    fn test_multiple_elements() {
        let html = "<h1>Title</h1><p>First paragraph.</p><p>Second paragraph.</p>";
        let markdown = convert(html);
        assert!(markdown.contains("# Title"));
        assert!(markdown.contains("First paragraph."));
        assert!(markdown.contains("Second paragraph."));
    }
}
