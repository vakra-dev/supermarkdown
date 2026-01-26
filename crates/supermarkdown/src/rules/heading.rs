//! Heading rule (h1-h6).

use once_cell::sync::Lazy;
use regex::Regex;
use scraper::ElementRef;

use crate::options::{HeadingStyle, Options};
use crate::precompute::MetadataMap;
use crate::rules::Rule;

/// Regex for normalizing whitespace in headings.
static WS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

pub struct HeadingRule;

impl Rule for HeadingRule {
    fn tags(&self) -> &'static [&'static str] {
        &["h1", "h2", "h3", "h4", "h5", "h6"]
    }

    fn convert(
        &self,
        element: ElementRef,
        metadata: &MetadataMap,
        options: &Options,
        convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
    ) -> String {
        let tag = element.value().name();
        let level: usize = tag[1..].parse().unwrap_or(1);

        let content = convert_children(element, metadata, options);
        let content = WS_RE.replace_all(content.trim(), " ");

        if content.is_empty() {
            return String::new();
        }

        match options.heading_style {
            HeadingStyle::Atx => {
                format!("\n\n{} {}\n\n", "#".repeat(level), content)
            }
            HeadingStyle::Setext if level <= 2 => {
                let underline = if level == 1 { "=" } else { "-" };
                // Use char count for proper unicode handling
                let len = content.chars().count();
                format!("\n\n{}\n{}\n\n", content, underline.repeat(len))
            }
            _ => format!("\n\n{} {}\n\n", "#".repeat(level), content),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    fn convert_test(html: &str, options: &Options) -> String {
        let dom = Html::parse_fragment(html);
        let element = dom.root_element().first_child().unwrap();
        let element = ElementRef::wrap(element).unwrap();
        let metadata = MetadataMap::default();

        HeadingRule.convert(element, &metadata, options, &|e, m, o| {
            e.text().collect::<Vec<_>>().join("")
        })
    }

    #[test]
    fn test_h1_atx() {
        let result = convert_test("<h1>Hello</h1>", &Options::default());
        assert!(result.contains("# Hello"));
    }

    #[test]
    fn test_h2_atx() {
        let result = convert_test("<h2>World</h2>", &Options::default());
        assert!(result.contains("## World"));
    }

    #[test]
    fn test_h1_setext() {
        let options = Options::new().heading_style(HeadingStyle::Setext);
        let result = convert_test("<h1>Title</h1>", &options);
        assert!(result.contains("Title\n====="));
    }

    #[test]
    fn test_h2_setext() {
        let options = Options::new().heading_style(HeadingStyle::Setext);
        let result = convert_test("<h2>Subtitle</h2>", &options);
        assert!(result.contains("Subtitle\n--------"));
    }

    #[test]
    fn test_empty_heading() {
        let result = convert_test("<h1></h1>", &Options::default());
        assert!(result.is_empty());
    }

    #[test]
    fn test_whitespace_normalization() {
        let result = convert_test("<h1>Hello   World</h1>", &Options::default());
        assert!(result.contains("# Hello World"));
    }
}
