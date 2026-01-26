//! Figure/figcaption rule.

use once_cell::sync::Lazy;
use regex::Regex;
use scraper::ElementRef;

use crate::options::Options;
use crate::precompute::MetadataMap;
use crate::rules::Rule;

/// Regex for normalizing whitespace.
static WS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

pub struct FigureRule;

impl Rule for FigureRule {
    fn tags(&self) -> &'static [&'static str] {
        &["figure"]
    }

    fn convert(
        &self,
        element: ElementRef,
        metadata: &MetadataMap,
        options: &Options,
        convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
    ) -> String {
        let mut image_md = String::new();
        let mut caption = String::new();

        for child in element.children() {
            if let Some(el) = ElementRef::wrap(child) {
                let tag = el.value().name();
                match tag {
                    "img" => {
                        // Convert img directly
                        let src = el.value().attr("src").unwrap_or("");
                        let alt = el.value().attr("alt").unwrap_or("");

                        if !src.is_empty() {
                            image_md = format!("![{}]({})", alt, src);
                        }
                    }
                    "figcaption" => {
                        let c = convert_children(el, metadata, options);
                        caption = WS_RE.replace_all(c.trim(), " ").to_string();
                    }
                    "picture" => {
                        // Handle <picture> element - find the img inside
                        for pic_child in el.children() {
                            if let Some(pic_el) = ElementRef::wrap(pic_child) {
                                if pic_el.value().name() == "img" {
                                    let src = pic_el.value().attr("src").unwrap_or("");
                                    let alt = pic_el.value().attr("alt").unwrap_or("");
                                    if !src.is_empty() {
                                        image_md = format!("![{}]({})", alt, src);
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        // Handle other nested elements that might contain images
                        let nested = convert_children(el, metadata, options);
                        if image_md.is_empty() && nested.contains("![") {
                            image_md = nested;
                        }
                    }
                }
            }
        }

        if image_md.is_empty() {
            return String::new();
        }

        let mut result = format!("\n\n{}", image_md.trim());
        if !caption.is_empty() {
            result.push_str(&format!("\n*{}*", caption));
        }
        result.push_str("\n\n");
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

        FigureRule.convert(element, &metadata, &Options::default(), &|e, _, _| {
            e.text().collect::<Vec<_>>().join("")
        })
    }

    #[test]
    fn test_figure_with_caption() {
        let result = convert_test(
            r#"<figure>
                <img src="photo.jpg" alt="A photo">
                <figcaption>This is the caption</figcaption>
            </figure>"#,
        );
        assert!(result.contains("![A photo](photo.jpg)"));
        assert!(result.contains("*This is the caption*"));
    }

    #[test]
    fn test_figure_without_caption() {
        let result = convert_test(
            r#"<figure>
                <img src="photo.jpg" alt="A photo">
            </figure>"#,
        );
        assert!(result.contains("![A photo](photo.jpg)"));
        assert!(!result.contains("*"));
    }

    #[test]
    fn test_figure_with_picture() {
        let result = convert_test(
            r#"<figure>
                <picture>
                    <source srcset="photo.webp" type="image/webp">
                    <img src="photo.jpg" alt="A photo">
                </picture>
            </figure>"#,
        );
        assert!(result.contains("![A photo](photo.jpg)"));
    }

    #[test]
    fn test_empty_figure() {
        let result = convert_test("<figure></figure>");
        assert!(result.is_empty());
    }
}
