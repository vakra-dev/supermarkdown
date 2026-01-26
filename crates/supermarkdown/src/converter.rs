//! Main conversion orchestrator.

use scraper::{ElementRef, Html};

use crate::entities::decode_entities;
use crate::options::Options;
use crate::postprocess::postprocess;
use crate::precompute::{precompute_metadata, CompiledSelectors, MetadataMap};
use crate::rules::{default_rules, find_rule, Rule};
use crate::whitespace::normalize_block_whitespace;

/// The main HTML to Markdown converter.
pub struct Converter {
    rules: Vec<Box<dyn Rule>>,
}

impl Converter {
    /// Create a new converter with default rules.
    pub fn new() -> Self {
        Self {
            rules: default_rules(),
        }
    }

    /// Convert HTML to Markdown.
    pub fn convert(&self, html: &str, options: &Options) -> String {
        if html.is_empty() {
            return String::new();
        }

        // 1. Parse HTML (html5ever handles malformed HTML gracefully)
        let dom = Html::parse_document(html);

        // 2. Compile selectors once
        let selectors = CompiledSelectors::new(options);

        // 3. Pre-compute metadata (single O(n) traversal)
        let metadata = precompute_metadata(&dom, &selectors, options);

        // 4. Convert to markdown (single O(n) traversal)
        let markdown = self.convert_element(dom.root_element(), &metadata, options);

        // 5. Post-process
        postprocess(markdown, options)
    }

    /// Convert an element and its children to markdown.
    fn convert_element(
        &self,
        element: ElementRef,
        metadata: &MetadataMap,
        options: &Options,
    ) -> String {
        self.convert_node_internal(element, metadata, options)
    }

    /// Internal conversion function.
    fn convert_node_internal(
        &self,
        element: ElementRef,
        metadata: &MetadataMap,
        options: &Options,
    ) -> String {
        // Check skip/force_keep from metadata
        if let Some(meta) = metadata.get(&element.id()) {
            if meta.skip && !meta.force_keep {
                return String::new();
            }
        }

        let tag = element.value().name();

        // Find matching rule
        if let Some(rule) = find_rule(&self.rules, tag) {
            return rule.convert(element, metadata, options, &|e, m, o| {
                self.convert_children(e, m, o)
            });
        }

        // Default: just convert children
        self.convert_children(element, metadata, options)
    }

    /// Convert all children of an element.
    fn convert_children(
        &self,
        element: ElementRef,
        metadata: &MetadataMap,
        options: &Options,
    ) -> String {
        let mut result = String::new();

        for child in element.children() {
            match child.value() {
                scraper::Node::Text(text) => {
                    // Decode HTML entities and normalize whitespace in text nodes
                    // (collapses multiple spaces/tabs/newlines to single space)
                    let decoded = decode_entities(text);
                    let normalized = normalize_block_whitespace(&decoded);
                    result.push_str(&normalized);
                }
                scraper::Node::Element(_) => {
                    if let Some(child_element) = ElementRef::wrap(child) {
                        result.push_str(&self.convert_node_internal(child_element, metadata, options));
                    }
                }
                _ => {}
            }
        }

        result
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{HeadingStyle, LinkStyle};

    fn convert(html: &str) -> String {
        Converter::new().convert(html, &Options::default())
    }

    fn convert_with(html: &str, options: &Options) -> String {
        Converter::new().convert(html, options)
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(convert(""), "");
    }

    #[test]
    fn test_heading() {
        let result = convert("<h1>Title</h1>");
        assert!(result.contains("# Title"));
    }

    #[test]
    fn test_paragraph() {
        let result = convert("<p>Hello World</p>");
        assert!(result.contains("Hello World"));
    }

    #[test]
    fn test_link() {
        let result = convert(r#"<a href="https://example.com">Link</a>"#);
        assert!(result.contains("[Link](https://example.com)"));
    }

    #[test]
    fn test_image() {
        let result = convert(r#"<img src="image.png" alt="Alt text">"#);
        assert!(result.contains("![Alt text](image.png)"));
    }

    #[test]
    fn test_emphasis() {
        let result = convert("<em>italic</em>");
        assert!(result.contains("*italic*"));
    }

    #[test]
    fn test_strong() {
        let result = convert("<strong>bold</strong>");
        assert!(result.contains("**bold**"));
    }

    #[test]
    fn test_code() {
        let result = convert("<code>inline</code>");
        assert!(result.contains("`inline`"));
    }

    #[test]
    fn test_pre() {
        let result = convert("<pre>code block</pre>");
        assert!(result.contains("```"));
        assert!(result.contains("code block"));
    }

    #[test]
    fn test_list() {
        let result = convert("<ul><li>One</li><li>Two</li></ul>");
        assert!(result.contains("- One"));
        assert!(result.contains("- Two"));
    }

    #[test]
    fn test_ordered_list() {
        let result = convert("<ol><li>First</li><li>Second</li></ol>");
        assert!(result.contains("1. First"));
        assert!(result.contains("2. Second"));
    }

    #[test]
    fn test_blockquote() {
        let result = convert("<blockquote>Quote</blockquote>");
        assert!(result.contains("> Quote"));
    }

    #[test]
    fn test_nested_blockquotes() {
        let result = convert("<blockquote>Outer<blockquote>Inner</blockquote></blockquote>");
        assert!(result.contains("> Outer"));
        assert!(result.contains("> > Inner"));
    }

    #[test]
    fn test_deeply_nested_blockquotes() {
        let result = convert(
            "<blockquote>Level 1<blockquote>Level 2<blockquote>Level 3</blockquote></blockquote></blockquote>",
        );
        assert!(result.contains("> Level 1"));
        assert!(result.contains("> > Level 2"));
        assert!(result.contains("> > > Level 3"));
    }

    #[test]
    fn test_hr() {
        let result = convert("<hr>");
        assert!(result.contains("---"));
    }

    #[test]
    fn test_entity_decoding() {
        let result = convert("<p>&lt;html&gt; &amp; more</p>");
        assert!(result.contains("<html> & more"));
    }

    #[test]
    fn test_nested_elements() {
        let result = convert("<p>This is <strong>bold and <em>italic</em></strong> text.</p>");
        assert!(result.contains("**bold and *italic***"));
    }

    #[test]
    fn test_setext_headings() {
        let options = Options::new().heading_style(HeadingStyle::Setext);
        let result = convert_with("<h1>Title</h1>", &options);
        assert!(result.contains("====="));
    }

    #[test]
    fn test_referenced_links() {
        let options = Options::new().link_style(LinkStyle::Referenced);
        let result = convert_with(
            r#"<p><a href="https://a.com">A</a> and <a href="https://b.com">B</a></p>"#,
            &options,
        );
        assert!(result.contains("[A][1]"));
        assert!(result.contains("[B][2]"));
        assert!(result.contains("[1]: https://a.com"));
    }

    #[test]
    fn test_exclude_selector() {
        let options = Options::new().exclude_selectors(vec!["nav".to_string()]);
        let result = convert_with("<div><nav>Skip this</nav><p>Keep this</p></div>", &options);
        assert!(!result.contains("Skip this"));
        assert!(result.contains("Keep this"));
    }

    #[test]
    fn test_whitespace_normalization() {
        // Multiple spaces should collapse to single space
        let result = convert("<p>Hello    world</p>");
        assert!(result.contains("Hello world"));

        // Newlines in inline context should become spaces
        let result = convert("<p>Hello\n\n\nworld</p>");
        assert!(result.contains("Hello world"));

        // Tabs should collapse to single space
        let result = convert("<p>Hello\t\tworld</p>");
        assert!(result.contains("Hello world"));

        // Mixed whitespace
        let result = convert("<p>Hello  \n\t  world</p>");
        assert!(result.contains("Hello world"));
    }

    #[test]
    fn test_complex_document() {
        let html = r#"
            <article>
                <h1>Article Title</h1>
                <p>First paragraph with <strong>bold</strong> and <em>italic</em>.</p>
                <h2>Section</h2>
                <p>A <a href="https://example.com">link</a> here.</p>
                <ul>
                    <li>Item 1</li>
                    <li>Item 2</li>
                </ul>
                <pre><code class="language-rust">fn main() {}</code></pre>
            </article>
        "#;
        let result = convert(html);

        assert!(result.contains("# Article Title"));
        assert!(result.contains("## Section"));
        assert!(result.contains("**bold**"));
        assert!(result.contains("*italic*"));
        assert!(result.contains("[link](https://example.com)"));
        assert!(result.contains("- Item 1"));
        assert!(result.contains("```rust"));
        assert!(result.contains("fn main()"));
    }
}
