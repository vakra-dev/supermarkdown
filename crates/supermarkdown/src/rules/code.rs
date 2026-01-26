//! Inline code rule.

use scraper::ElementRef;

use crate::escape::calculate_code_backticks;
use crate::options::Options;
use crate::precompute::MetadataMap;
use crate::rules::Rule;

pub struct CodeRule;

impl Rule for CodeRule {
    fn tags(&self) -> &'static [&'static str] {
        &["code"]
    }

    fn convert(
        &self,
        element: ElementRef,
        _metadata: &MetadataMap,
        _options: &Options,
        _convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
    ) -> String {
        // Check if this is inside a <pre> - if so, let PreRule handle it
        if let Some(parent) = element.parent() {
            if let Some(parent_el) = ElementRef::wrap(parent) {
                if parent_el.value().name() == "pre" {
                    // This is a code block, not inline code - just return text
                    return element.text().collect::<Vec<_>>().join("");
                }
            }
        }

        let code = element.text().collect::<Vec<_>>().join("");

        if code.is_empty() {
            return String::new();
        }

        // Calculate required number of backticks
        let backticks = calculate_code_backticks(&code);
        let delim = "`".repeat(backticks);

        // Add padding if code starts or ends with backtick
        let (prefix, suffix) = if code.starts_with('`') || code.ends_with('`') {
            (" ", " ")
        } else {
            ("", "")
        };

        format!("{}{}{}{}{}", delim, prefix, code, suffix, delim)
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

        CodeRule.convert(element, &metadata, &Options::default(), &|e, _, _| {
            e.text().collect::<Vec<_>>().join("")
        })
    }

    #[test]
    fn test_simple_code() {
        assert_eq!(convert_test("<code>hello</code>"), "`hello`");
    }

    #[test]
    fn test_code_with_backticks() {
        assert_eq!(convert_test("<code>use `this`</code>"), "`` use `this` ``");
    }

    #[test]
    fn test_code_starting_with_backtick() {
        assert_eq!(convert_test("<code>`start</code>"), "`` `start ``");
    }

    #[test]
    fn test_empty_code() {
        assert_eq!(convert_test("<code></code>"), "");
    }
}
