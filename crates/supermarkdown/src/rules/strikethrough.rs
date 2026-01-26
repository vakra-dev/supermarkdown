//! Strikethrough rule (del, s, strike).

use scraper::ElementRef;

use crate::options::Options;
use crate::precompute::MetadataMap;
use crate::rules::Rule;

pub struct StrikethroughRule;

impl Rule for StrikethroughRule {
    fn tags(&self) -> &'static [&'static str] {
        &["del", "s", "strike"]
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

        format!("~~{}~~", content)
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

        StrikethroughRule.convert(element, &metadata, &Options::default(), &|e, _, _| {
            e.text().collect::<Vec<_>>().join("")
        })
    }

    #[test]
    fn test_del() {
        assert_eq!(convert_test("<del>deleted</del>"), "~~deleted~~");
    }

    #[test]
    fn test_s() {
        assert_eq!(convert_test("<s>strikethrough</s>"), "~~strikethrough~~");
    }

    #[test]
    fn test_strike() {
        assert_eq!(convert_test("<strike>old</strike>"), "~~old~~");
    }

    #[test]
    fn test_empty() {
        assert_eq!(convert_test("<del></del>"), "");
    }
}
