//! Link rule.

use once_cell::sync::Lazy;
use regex::Regex;
use scraper::ElementRef;

use crate::escape::{escape_title, escape_url, resolve_url};
use crate::options::Options;
use crate::precompute::MetadataMap;
use crate::rules::Rule;

/// Regex for normalizing whitespace in link text.
static WS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

pub struct LinkRule;

impl Rule for LinkRule {
    fn tags(&self) -> &'static [&'static str] {
        &["a"]
    }

    fn convert(
        &self,
        element: ElementRef,
        metadata: &MetadataMap,
        options: &Options,
        convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
    ) -> String {
        let href = element.value().attr("href").unwrap_or("");
        let title = element.value().attr("title");

        let content = convert_children(element, metadata, options);
        let content = WS_RE.replace_all(content.trim(), " ");

        // Handle empty or fragment-only href
        if href.is_empty() || href == "#" {
            return content.to_string();
        }

        // Resolve relative URLs if base_url provided
        let href = if let Some(base) = &options.base_url {
            resolve_url(base, href)
        } else {
            href.to_string()
        };

        // Check for autolink: when link text equals URL or email
        // Email autolink: <a href="mailto:test@example.com">test@example.com</a> → <test@example.com>
        // URL autolink: <a href="https://example.com">https://example.com</a> → <https://example.com>
        if title.is_none() {
            // Check for email autolink
            if let Some(email) = href.strip_prefix("mailto:") {
                if content == email {
                    return format!("<{}>", email);
                }
            }
            // Check for URL autolink (link text matches href)
            if content == href {
                return format!("<{}>", href);
            }
        }

        let href = escape_url(&href);

        // All links output as inline format - referenced conversion happens in postprocess
        match title {
            Some(t) => {
                format!("[{}]({} \"{}\")", content, href, escape_title(t))
            }
            None => {
                format!("[{}]({})", content, href)
            }
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

        LinkRule.convert(element, &metadata, options, &|e, _, _| {
            e.text().collect::<Vec<_>>().join("")
        })
    }

    #[test]
    fn test_simple_link() {
        let result = convert_test(
            r#"<a href="https://example.com">Link</a>"#,
            &Options::default(),
        );
        assert_eq!(result, "[Link](https://example.com)");
    }

    #[test]
    fn test_link_with_title() {
        let result = convert_test(
            r#"<a href="https://example.com" title="Example">Link</a>"#,
            &Options::default(),
        );
        assert_eq!(result, r#"[Link](https://example.com "Example")"#);
    }

    #[test]
    fn test_empty_href() {
        let result = convert_test(r#"<a href="">Text</a>"#, &Options::default());
        assert_eq!(result, "Text");
    }

    #[test]
    fn test_fragment_only() {
        let result = convert_test(r##"<a href="#">Text</a>"##, &Options::default());
        assert_eq!(result, "Text");
    }

    #[test]
    fn test_relative_url_with_base() {
        let options = Options::new().base_url(Some("https://example.com/dir/".to_string()));
        let result = convert_test(r#"<a href="page.html">Link</a>"#, &options);
        assert!(result.contains("https://example.com/dir/page.html"));
    }

    #[test]
    fn test_url_with_spaces() {
        let result = convert_test(
            r#"<a href="https://example.com/path with spaces">Link</a>"#,
            &Options::default(),
        );
        assert!(result.contains("%20"));
    }

    #[test]
    fn test_autolink_url() {
        let result = convert_test(
            r#"<a href="https://example.com">https://example.com</a>"#,
            &Options::default(),
        );
        assert_eq!(result, "<https://example.com>");
    }

    #[test]
    fn test_autolink_email() {
        let result = convert_test(
            r#"<a href="mailto:test@example.com">test@example.com</a>"#,
            &Options::default(),
        );
        assert_eq!(result, "<test@example.com>");
    }

    #[test]
    fn test_no_autolink_when_text_differs() {
        let result = convert_test(
            r#"<a href="https://example.com">Example Site</a>"#,
            &Options::default(),
        );
        assert_eq!(result, "[Example Site](https://example.com)");
    }

    #[test]
    fn test_no_autolink_with_title() {
        let result = convert_test(
            r#"<a href="https://example.com" title="tip">https://example.com</a>"#,
            &Options::default(),
        );
        assert_eq!(
            result,
            r#"[https://example.com](https://example.com "tip")"#
        );
    }

    #[test]
    fn test_fragment_link_with_section() {
        // Fragment links like #section should be preserved, unlike bare #
        let result = convert_test(r##"<a href="#section">Section</a>"##, &Options::default());
        assert_eq!(result, "[Section](#section)");
    }

    #[test]
    fn test_fragment_link_with_complex_id() {
        let result = convert_test(
            r##"<a href="#user-guide-installation">Installation</a>"##,
            &Options::default(),
        );
        assert_eq!(result, "[Installation](#user-guide-installation)");
    }
}
