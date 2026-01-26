//! Definition list rule (dl/dt/dd).

use scraper::ElementRef;

use crate::options::Options;
use crate::precompute::MetadataMap;
use crate::rules::Rule;

/// Rule for definition list container `<dl>`.
pub struct DefListRule;

impl Rule for DefListRule {
    fn tags(&self) -> &'static [&'static str] {
        &["dl"]
    }

    fn convert(
        &self,
        element: ElementRef,
        metadata: &MetadataMap,
        options: &Options,
        convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
    ) -> String {
        let mut result = String::from("\n\n");
        let mut last_was_dt = false;

        for child in element.children() {
            if let Some(el) = ElementRef::wrap(child) {
                match el.value().name() {
                    "dt" => {
                        let content = convert_children(el, metadata, options);
                        let content = content.trim();
                        if !content.is_empty() {
                            if !last_was_dt && !result.trim().is_empty() {
                                result.push('\n');
                            }
                            result.push_str(content);
                            result.push('\n');
                            last_was_dt = true;
                        }
                    }
                    "dd" => {
                        let content = convert_children(el, metadata, options);
                        let content = content.trim();
                        if !content.is_empty() {
                            // Indent multi-line definitions
                            let lines: Vec<&str> = content.lines().collect();
                            for (i, line) in lines.iter().enumerate() {
                                if i == 0 {
                                    result.push_str(": ");
                                    result.push_str(line);
                                } else {
                                    result.push_str("\n  ");
                                    result.push_str(line);
                                }
                            }
                            result.push('\n');
                            last_was_dt = false;
                        }
                    }
                    _ => {}
                }
            }
        }

        result.push('\n');
        result
    }
}

/// Rule for definition term `<dt>` (handled by DefListRule).
pub struct DefTermRule;

impl Rule for DefTermRule {
    fn tags(&self) -> &'static [&'static str] {
        &["dt"]
    }

    fn convert(
        &self,
        element: ElementRef,
        metadata: &MetadataMap,
        options: &Options,
        convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
    ) -> String {
        // When standalone (not inside dl), just return the text
        let content = convert_children(element, metadata, options);
        content.trim().to_string()
    }
}

/// Rule for definition description `<dd>` (handled by DefListRule).
pub struct DefDescRule;

impl Rule for DefDescRule {
    fn tags(&self) -> &'static [&'static str] {
        &["dd"]
    }

    fn convert(
        &self,
        element: ElementRef,
        metadata: &MetadataMap,
        options: &Options,
        convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
    ) -> String {
        // When standalone (not inside dl), just return the text with colon prefix
        let content = convert_children(element, metadata, options);
        let content = content.trim();
        if content.is_empty() {
            String::new()
        } else {
            format!(": {}", content)
        }
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

        DefListRule.convert(element, &metadata, &Options::default(), &|e, _, _| {
            e.text().collect::<Vec<_>>().join("")
        })
    }

    #[test]
    fn test_simple_deflist() {
        let result = convert_test(
            r#"<dl>
                <dt>Term</dt>
                <dd>Definition</dd>
            </dl>"#,
        );
        assert!(result.contains("Term"));
        assert!(result.contains(": Definition"));
    }

    #[test]
    fn test_multiple_definitions() {
        let result = convert_test(
            r#"<dl>
                <dt>Term 1</dt>
                <dd>Definition 1</dd>
                <dt>Term 2</dt>
                <dd>Definition 2</dd>
            </dl>"#,
        );
        assert!(result.contains("Term 1"));
        assert!(result.contains(": Definition 1"));
        assert!(result.contains("Term 2"));
        assert!(result.contains(": Definition 2"));
    }

    #[test]
    fn test_multiple_dd_per_dt() {
        let result = convert_test(
            r#"<dl>
                <dt>Term</dt>
                <dd>First definition</dd>
                <dd>Second definition</dd>
            </dl>"#,
        );
        assert!(result.contains("Term"));
        assert!(result.contains(": First definition"));
        assert!(result.contains(": Second definition"));
    }

    #[test]
    fn test_empty_deflist() {
        let result = convert_test("<dl></dl>");
        assert!(result.trim().is_empty());
    }
}
