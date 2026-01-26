//! Superscript rule.

use scraper::ElementRef;

use crate::options::Options;
use crate::precompute::MetadataMap;
use crate::rules::Rule;

pub struct SuperscriptRule;

impl Rule for SuperscriptRule {
    fn tags(&self) -> &'static [&'static str] {
        &["sup"]
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

        // Use HTML tag for compatibility (not all markdown parsers support ^)
        format!("<sup>{}</sup>", content)
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

        SuperscriptRule.convert(element, &metadata, &Options::default(), &|e, _, _| {
            e.text().collect::<Vec<_>>().join("")
        })
    }

    #[test]
    fn test_superscript() {
        assert_eq!(convert_test("<sup>2</sup>"), "<sup>2</sup>");
    }

    #[test]
    fn test_empty() {
        assert_eq!(convert_test("<sup></sup>"), "");
    }
}
