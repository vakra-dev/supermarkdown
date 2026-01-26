//! Paragraph rule.

use scraper::ElementRef;

use crate::options::Options;
use crate::precompute::MetadataMap;
use crate::rules::Rule;

pub struct ParagraphRule;

impl Rule for ParagraphRule {
    fn tags(&self) -> &'static [&'static str] {
        &["p"]
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

        format!("\n\n{}\n\n", content)
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

        ParagraphRule.convert(element, &metadata, &Options::default(), &|e, _, _| {
            e.text().collect::<Vec<_>>().join("")
        })
    }

    #[test]
    fn test_paragraph() {
        let result = convert_test("<p>Hello World</p>");
        assert!(result.contains("Hello World"));
        assert!(result.starts_with("\n\n"));
        assert!(result.ends_with("\n\n"));
    }

    #[test]
    fn test_empty_paragraph() {
        let result = convert_test("<p></p>");
        assert!(result.is_empty());
    }

    #[test]
    fn test_whitespace_paragraph() {
        let result = convert_test("<p>   </p>");
        assert!(result.is_empty());
    }
}
