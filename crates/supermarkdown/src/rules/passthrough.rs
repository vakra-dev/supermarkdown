//! Passthrough rule for HTML elements without Markdown equivalents.
//!
//! Elements like <kbd>, <mark>, <abbr>, <samp>, <var> are passed through
//! as raw HTML since Markdown supports inline HTML.

use scraper::ElementRef;

use crate::options::Options;
use crate::precompute::MetadataMap;
use crate::rules::Rule;

/// Rule for keyboard input `<kbd>`.
pub struct KbdRule;

impl Rule for KbdRule {
    fn tags(&self) -> &'static [&'static str] {
        &["kbd"]
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
            String::new()
        } else {
            format!("<kbd>{}</kbd>", content)
        }
    }
}

/// Rule for marked/highlighted text `<mark>`.
pub struct MarkRule;

impl Rule for MarkRule {
    fn tags(&self) -> &'static [&'static str] {
        &["mark"]
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
            String::new()
        } else {
            format!("<mark>{}</mark>", content)
        }
    }
}

/// Rule for abbreviations `<abbr>`.
pub struct AbbrRule;

impl Rule for AbbrRule {
    fn tags(&self) -> &'static [&'static str] {
        &["abbr"]
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

        // Preserve title attribute if present
        if let Some(title) = element.value().attr("title") {
            format!("<abbr title=\"{}\">{}</abbr>", escape_attr(title), content)
        } else {
            format!("<abbr>{}</abbr>", content)
        }
    }
}

/// Rule for sample output `<samp>`.
pub struct SampRule;

impl Rule for SampRule {
    fn tags(&self) -> &'static [&'static str] {
        &["samp"]
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
            String::new()
        } else {
            format!("<samp>{}</samp>", content)
        }
    }
}

/// Rule for variables `<var>`.
pub struct VarRule;

impl Rule for VarRule {
    fn tags(&self) -> &'static [&'static str] {
        &["var"]
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
            String::new()
        } else {
            format!("<var>{}</var>", content)
        }
    }
}

/// Escape special characters in HTML attribute values.
fn escape_attr(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    fn convert_test<R: Rule>(rule: &R, html: &str) -> String {
        let dom = Html::parse_fragment(html);
        let element = dom.root_element().first_child().unwrap();
        let element = ElementRef::wrap(element).unwrap();
        let metadata = MetadataMap::default();

        rule.convert(element, &metadata, &Options::default(), &|e, _, _| {
            e.text().collect::<Vec<_>>().join("")
        })
    }

    #[test]
    fn test_kbd() {
        let result = convert_test(&KbdRule, "<kbd>Ctrl+C</kbd>");
        assert_eq!(result, "<kbd>Ctrl+C</kbd>");
    }

    #[test]
    fn test_mark() {
        let result = convert_test(&MarkRule, "<mark>highlighted</mark>");
        assert_eq!(result, "<mark>highlighted</mark>");
    }

    #[test]
    fn test_abbr() {
        let result = convert_test(&AbbrRule, "<abbr>HTML</abbr>");
        assert_eq!(result, "<abbr>HTML</abbr>");
    }

    #[test]
    fn test_abbr_with_title() {
        let result = convert_test(
            &AbbrRule,
            r#"<abbr title="HyperText Markup Language">HTML</abbr>"#,
        );
        assert_eq!(
            result,
            "<abbr title=\"HyperText Markup Language\">HTML</abbr>"
        );
    }

    #[test]
    fn test_samp() {
        let result = convert_test(&SampRule, "<samp>output</samp>");
        assert_eq!(result, "<samp>output</samp>");
    }

    #[test]
    fn test_var() {
        let result = convert_test(&VarRule, "<var>x</var>");
        assert_eq!(result, "<var>x</var>");
    }

    #[test]
    fn test_empty_elements() {
        assert!(convert_test(&KbdRule, "<kbd></kbd>").is_empty());
        assert!(convert_test(&MarkRule, "<mark></mark>").is_empty());
        assert!(convert_test(&AbbrRule, "<abbr></abbr>").is_empty());
        assert!(convert_test(&SampRule, "<samp></samp>").is_empty());
        assert!(convert_test(&VarRule, "<var></var>").is_empty());
    }
}
