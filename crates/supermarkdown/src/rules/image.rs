//! Image rule.

use scraper::ElementRef;

use crate::escape::{escape_title, escape_url, resolve_url};
use crate::options::Options;
use crate::precompute::MetadataMap;
use crate::rules::Rule;

pub struct ImageRule;

impl Rule for ImageRule {
    fn tags(&self) -> &'static [&'static str] {
        &["img"]
    }

    fn convert(
        &self,
        element: ElementRef,
        _metadata: &MetadataMap,
        options: &Options,
        _convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
    ) -> String {
        let src = element.value().attr("src").unwrap_or("");
        let alt = element.value().attr("alt").unwrap_or("");
        let title = element.value().attr("title");

        // Skip images without src
        if src.is_empty() {
            return String::new();
        }

        // Resolve relative URLs if base_url provided
        let src = if let Some(base) = &options.base_url {
            resolve_url(base, src)
        } else {
            src.to_string()
        };

        let src = escape_url(&src);

        match title {
            Some(t) => format!("![{}]({} \"{}\")", alt, src, escape_title(t)),
            None => format!("![{}]({})", alt, src),
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

        ImageRule.convert(element, &metadata, options, &|_, _, _| String::new())
    }

    #[test]
    fn test_simple_image() {
        let result = convert_test(
            r#"<img src="image.png" alt="An image">"#,
            &Options::default(),
        );
        assert_eq!(result, "![An image](image.png)");
    }

    #[test]
    fn test_image_with_title() {
        let result = convert_test(
            r#"<img src="image.png" alt="Alt" title="Title">"#,
            &Options::default(),
        );
        assert_eq!(result, r#"![Alt](image.png "Title")"#);
    }

    #[test]
    fn test_empty_src() {
        let result = convert_test(r#"<img src="" alt="No source">"#, &Options::default());
        assert!(result.is_empty());
    }

    #[test]
    fn test_no_alt() {
        let result = convert_test(r#"<img src="image.png">"#, &Options::default());
        assert_eq!(result, "![](image.png)");
    }

    #[test]
    fn test_relative_url_with_base() {
        let options = Options::new().base_url(Some("https://example.com/".to_string()));
        let result = convert_test(r#"<img src="images/photo.jpg" alt="Photo">"#, &options);
        assert!(result.contains("https://example.com/images/photo.jpg"));
    }
}
