//! List rules (ul, ol, li).

use scraper::ElementRef;

use crate::options::Options;
use crate::precompute::MetadataMap;
use crate::rules::Rule;

/// Rule for ul and ol elements - delegates to children.
pub struct ListRule;

impl Rule for ListRule {
    fn tags(&self) -> &'static [&'static str] {
        &["ul", "ol"]
    }

    fn convert(
        &self,
        element: ElementRef,
        metadata: &MetadataMap,
        options: &Options,
        convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
    ) -> String {
        let content = convert_children(element, metadata, options);
        let content = content.trim_end();

        if content.is_empty() {
            return String::new();
        }

        format!("\n\n{}\n\n", content)
    }
}

/// Rule for li elements - uses pre-computed prefix and indent.
pub struct ListItemRule;

impl Rule for ListItemRule {
    fn tags(&self) -> &'static [&'static str] {
        &["li"]
    }

    fn convert(
        &self,
        element: ElementRef,
        metadata: &MetadataMap,
        options: &Options,
        convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
    ) -> String {
        let content = convert_children(element, metadata, options);
        let content = content.trim();

        if content.is_empty() {
            return String::new();
        }

        // O(1) lookup from pre-computed metadata
        let (prefix, indent): (String, usize) = if let Some(meta) = metadata.get(&element.id()) {
            (
                meta.list_prefix.clone().unwrap_or_else(|| "- ".to_string()),
                meta.ancestor_indent,
            )
        } else {
            // Fallback if not found
            (format!("{} ", options.bullet_marker), 0)
        };

        // Indent continuation lines
        let indented = indent_continuation(content, prefix.len() + indent);

        format!("{}{}{}\n", " ".repeat(indent), prefix, indented)
    }
}

/// Indent continuation lines of multi-line content.
fn indent_continuation(text: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    let mut lines = text.lines();

    let first = lines.next().unwrap_or("");
    let rest: String = lines
        .map(|line| {
            if line.is_empty() {
                "\n".to_string()
            } else {
                format!("\n{}{}", indent, line)
            }
        })
        .collect();

    format!("{}{}", first, rest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::precompute::{precompute_metadata, CompiledSelectors};
    use scraper::Html;

    #[test]
    fn test_unordered_list() {
        let html = "<ul><li>First</li><li>Second</li></ul>";
        let dom = Html::parse_document(html);
        let options = Options::default();
        let selectors = CompiledSelectors::new(&options);
        let metadata = precompute_metadata(&dom, &selectors, &options);

        // Find the ul element
        let ul = dom
            .select(&scraper::Selector::parse("ul").unwrap())
            .next()
            .unwrap();

        let result = ListRule.convert(ul, &metadata, &options, &|e, m, o| {
            let mut s = String::new();
            for child in e.children() {
                if let Some(el) = ElementRef::wrap(child) {
                    if el.value().name() == "li" {
                        s.push_str(&ListItemRule.convert(el, m, o, &|e, _, _| {
                            e.text().collect::<Vec<_>>().join("")
                        }));
                    }
                }
            }
            s
        });

        assert!(result.contains("- First"));
        assert!(result.contains("- Second"));
    }

    #[test]
    fn test_ordered_list() {
        let html = "<ol><li>One</li><li>Two</li><li>Three</li></ol>";
        let dom = Html::parse_document(html);
        let options = Options::default();
        let selectors = CompiledSelectors::new(&options);
        let metadata = precompute_metadata(&dom, &selectors, &options);

        let ol = dom
            .select(&scraper::Selector::parse("ol").unwrap())
            .next()
            .unwrap();

        let result = ListRule.convert(ol, &metadata, &options, &|e, m, o| {
            let mut s = String::new();
            for child in e.children() {
                if let Some(el) = ElementRef::wrap(child) {
                    if el.value().name() == "li" {
                        s.push_str(&ListItemRule.convert(el, m, o, &|e, _, _| {
                            e.text().collect::<Vec<_>>().join("")
                        }));
                    }
                }
            }
            s
        });

        assert!(result.contains("1. One"));
        assert!(result.contains("2. Two"));
        assert!(result.contains("3. Three"));
    }

    #[test]
    fn test_custom_bullet() {
        let html = "<ul><li>Item</li></ul>";
        let dom = Html::parse_document(html);
        let options = Options::new().bullet_marker('*');
        let selectors = CompiledSelectors::new(&options);
        let metadata = precompute_metadata(&dom, &selectors, &options);

        let li = dom
            .select(&scraper::Selector::parse("li").unwrap())
            .next()
            .unwrap();

        let result = ListItemRule.convert(li, &metadata, &options, &|e, _, _| {
            e.text().collect::<Vec<_>>().join("")
        });

        assert!(result.contains("* Item"));
    }

    #[test]
    fn test_indent_continuation() {
        let text = "Line 1\nLine 2\nLine 3";
        let result = indent_continuation(text, 4);
        assert_eq!(result, "Line 1\n    Line 2\n    Line 3");
    }

    #[test]
    fn test_ordered_list_with_start() {
        let html = r#"<ol start="5"><li>Fifth</li><li>Sixth</li><li>Seventh</li></ol>"#;
        let dom = Html::parse_document(html);
        let options = Options::default();
        let selectors = CompiledSelectors::new(&options);
        let metadata = precompute_metadata(&dom, &selectors, &options);

        let ol = dom
            .select(&scraper::Selector::parse("ol").unwrap())
            .next()
            .unwrap();

        let result = ListRule.convert(ol, &metadata, &options, &|e, m, o| {
            let mut s = String::new();
            for child in e.children() {
                if let Some(el) = ElementRef::wrap(child) {
                    if el.value().name() == "li" {
                        s.push_str(&ListItemRule.convert(el, m, o, &|e, _, _| {
                            e.text().collect::<Vec<_>>().join("")
                        }));
                    }
                }
            }
            s
        });

        assert!(result.contains("5. Fifth"));
        assert!(result.contains("6. Sixth"));
        assert!(result.contains("7. Seventh"));
    }

    #[test]
    fn test_nested_list() {
        let html = r#"<ul><li>Level 1<ul><li>Level 2</li></ul></li></ul>"#;
        let dom = Html::parse_document(html);
        let options = Options::default();
        let selectors = CompiledSelectors::new(&options);
        let metadata = precompute_metadata(&dom, &selectors, &options);

        // Verify nested list has indentation
        let li_metadata: Vec<_> = metadata
            .values()
            .filter(|m| m.list_prefix.is_some())
            .collect();

        // Should have 2 list items
        assert_eq!(li_metadata.len(), 2);

        // One should have ancestor_indent > 0
        let has_nested = li_metadata.iter().any(|m| m.ancestor_indent > 0);
        assert!(has_nested);
    }

    #[test]
    fn test_empty_list_items_skipped() {
        let html = "<ul><li>Item 1</li><li></li><li>Item 3</li></ul>";
        let dom = Html::parse_document(html);
        let options = Options::default();
        let selectors = CompiledSelectors::new(&options);
        let metadata = precompute_metadata(&dom, &selectors, &options);

        let ul = dom
            .select(&scraper::Selector::parse("ul").unwrap())
            .next()
            .unwrap();

        let result = ListRule.convert(ul, &metadata, &options, &|e, m, o| {
            let mut s = String::new();
            for child in e.children() {
                if let Some(el) = ElementRef::wrap(child) {
                    if el.value().name() == "li" {
                        s.push_str(&ListItemRule.convert(el, m, o, &|e, _, _| {
                            e.text().collect::<Vec<_>>().join("")
                        }));
                    }
                }
            }
            s
        });

        // Empty list items should be skipped
        assert!(result.contains("- Item 1"));
        assert!(result.contains("- Item 3"));
        // Should not have extra blank markers
        let marker_count = result.matches("- ").count();
        assert_eq!(marker_count, 2);
    }

    #[test]
    fn test_plus_bullet_marker() {
        let html = "<ul><li>Item</li></ul>";
        let dom = Html::parse_document(html);
        let options = Options::new().bullet_marker('+');
        let selectors = CompiledSelectors::new(&options);
        let metadata = precompute_metadata(&dom, &selectors, &options);

        let li = dom
            .select(&scraper::Selector::parse("li").unwrap())
            .next()
            .unwrap();

        let result = ListItemRule.convert(li, &metadata, &options, &|e, _, _| {
            e.text().collect::<Vec<_>>().join("")
        });

        assert!(result.contains("+ Item"));
    }
}
