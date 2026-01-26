//! Blockquote rule.

use scraper::ElementRef;

use crate::options::Options;
use crate::precompute::MetadataMap;
use crate::rules::Rule;

pub struct BlockquoteRule;

impl Rule for BlockquoteRule {
    fn tags(&self) -> &'static [&'static str] {
        &["blockquote"]
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

        // Prefix each line with "> "
        let quoted: String = content
            .lines()
            .map(|line| {
                if line.is_empty() {
                    ">".to_string()
                } else {
                    format!("> {}", line)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!("\n\n{}\n\n", quoted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    fn convert_test(html: &str) -> String {
        let dom = Html::parse_fragment(html);
        let element = dom.root_element().first_child().unwrap();
        let element = ElementRef::wrap(element).unwrap();
        let metadata = MetadataMap::default();

        BlockquoteRule.convert(element, &metadata, &Options::default(), &|e, _, _| {
            e.text().collect::<Vec<_>>().join("")
        })
    }

    #[test]
    fn test_simple_blockquote() {
        let result = convert_test("<blockquote>Quote</blockquote>");
        assert!(result.contains("> Quote"));
    }

    #[test]
    fn test_multiline_blockquote() {
        let result = convert_test("<blockquote>Line 1\nLine 2</blockquote>");
        assert!(result.contains("> Line 1"));
        assert!(result.contains("> Line 2"));
    }

    #[test]
    fn test_empty_blockquote() {
        let result = convert_test("<blockquote></blockquote>");
        assert!(result.is_empty());
    }
}
