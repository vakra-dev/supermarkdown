//! Details/summary rule.

use once_cell::sync::Lazy;
use regex::Regex;
use scraper::ElementRef;

use crate::options::Options;
use crate::precompute::MetadataMap;
use crate::rules::Rule;

/// Regex for normalizing whitespace.
static WS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

pub struct DetailsRule;

impl Rule for DetailsRule {
    fn tags(&self) -> &'static [&'static str] {
        &["details"]
    }

    fn convert(
        &self,
        element: ElementRef,
        metadata: &MetadataMap,
        options: &Options,
        convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
    ) -> String {
        let mut summary = String::new();
        let mut content = String::new();

        for child in element.children() {
            if let Some(el) = ElementRef::wrap(child) {
                if el.value().name() == "summary" {
                    let s = convert_children(el, metadata, options);
                    summary = WS_RE.replace_all(s.trim(), " ").to_string();
                } else {
                    content.push_str(&convert_children(el, metadata, options));
                }
            } else if let Some(text) = child.value().as_text() {
                content.push_str(text);
            }
        }

        let content = content.trim();

        if summary.is_empty() && content.is_empty() {
            return String::new();
        }

        // Format as blockquote with summary as bold header
        let mut result = String::from("\n\n");
        if !summary.is_empty() {
            result.push_str(&format!("> **{}**\n>\n", summary));
        }
        for line in content.lines() {
            if line.is_empty() {
                result.push_str(">\n");
            } else {
                result.push_str(&format!("> {}\n", line));
            }
        }
        result.push('\n');
        result
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

        DetailsRule.convert(element, &metadata, &Options::default(), &|e, _, _| {
            e.text().collect::<Vec<_>>().join("")
        })
    }

    #[test]
    fn test_details_with_summary() {
        let result = convert_test(
            r#"<details>
                <summary>Click to expand</summary>
                <p>Hidden content</p>
            </details>"#,
        );
        assert!(result.contains("> **Click to expand**"));
        assert!(result.contains("> Hidden content"));
    }

    #[test]
    fn test_details_without_summary() {
        let result = convert_test("<details><p>Just content</p></details>");
        assert!(result.contains("> Just content"));
        assert!(!result.contains("**"));
    }

    #[test]
    fn test_empty_details() {
        let result = convert_test("<details></details>");
        assert!(result.is_empty());
    }
}
