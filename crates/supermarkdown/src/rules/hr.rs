//! Horizontal rule.

use scraper::ElementRef;

use crate::options::Options;
use crate::precompute::MetadataMap;
use crate::rules::Rule;

pub struct HorizontalRule;

impl Rule for HorizontalRule {
    fn tags(&self) -> &'static [&'static str] {
        &["hr"]
    }

    fn convert(
        &self,
        _element: ElementRef,
        _metadata: &MetadataMap,
        _options: &Options,
        _convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
    ) -> String {
        "\n\n---\n\n".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn test_hr() {
        let dom = Html::parse_fragment("<hr>");
        let element = dom.root_element().first_child().unwrap();
        let element = ElementRef::wrap(element).unwrap();

        let result = HorizontalRule.convert(
            element,
            &MetadataMap::default(),
            &Options::default(),
            &|_, _, _| String::new(),
        );
        assert_eq!(result, "\n\n---\n\n");
    }
}
