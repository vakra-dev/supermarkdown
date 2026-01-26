//! Emphasis rules (em/i and strong/b).

use scraper::ElementRef;

use crate::options::Options;
use crate::precompute::MetadataMap;
use crate::rules::Rule;

/// Strong/bold rule (** or __).
pub struct StrongRule;

impl Rule for StrongRule {
    fn tags(&self) -> &'static [&'static str] {
        &["strong", "b"]
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

        format!("**{}**", content)
    }
}

/// Emphasis/italic rule (* or _).
pub struct EmphasisRule;

impl Rule for EmphasisRule {
    fn tags(&self) -> &'static [&'static str] {
        &["em", "i"]
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

        format!("*{}*", content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    fn convert_strong(html: &str) -> String {
        let dom = Html::parse_fragment(html);
        let element = dom.root_element().first_child().unwrap();
        let element = ElementRef::wrap(element).unwrap();
        let metadata = MetadataMap::default();

        StrongRule.convert(element, &metadata, &Options::default(), &|e, _, _| {
            e.text().collect::<Vec<_>>().join("")
        })
    }

    fn convert_em(html: &str) -> String {
        let dom = Html::parse_fragment(html);
        let element = dom.root_element().first_child().unwrap();
        let element = ElementRef::wrap(element).unwrap();
        let metadata = MetadataMap::default();

        EmphasisRule.convert(element, &metadata, &Options::default(), &|e, _, _| {
            e.text().collect::<Vec<_>>().join("")
        })
    }

    #[test]
    fn test_strong() {
        assert_eq!(convert_strong("<strong>bold</strong>"), "**bold**");
        assert_eq!(convert_strong("<b>bold</b>"), "**bold**");
    }

    #[test]
    fn test_emphasis() {
        assert_eq!(convert_em("<em>italic</em>"), "*italic*");
        assert_eq!(convert_em("<i>italic</i>"), "*italic*");
    }

    #[test]
    fn test_empty() {
        assert_eq!(convert_strong("<strong></strong>"), "");
        assert_eq!(convert_em("<em></em>"), "");
    }
}
