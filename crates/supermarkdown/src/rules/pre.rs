//! Pre/code block rule.

use once_cell::sync::Lazy;
use regex::Regex;
use scraper::ElementRef;

use crate::options::Options;
use crate::precompute::MetadataMap;
use crate::rules::Rule;

/// Cached regex for counting backtick runs.
static BACKTICK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"`+").unwrap());
static TILDE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"~+").unwrap());

pub struct PreRule;

impl Rule for PreRule {
    fn tags(&self) -> &'static [&'static str] {
        &["pre"]
    }

    fn convert(
        &self,
        element: ElementRef,
        _metadata: &MetadataMap,
        options: &Options,
        _convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
    ) -> String {
        // Detect language from <code class="language-xxx">
        let lang = detect_language(&element).unwrap_or_default();

        // Collect text, skipping line number gutters
        let code = collect_code_text(&element);
        let code = code.trim_end_matches('\n');

        if code.is_empty() {
            return String::new();
        }

        // Dynamic fence calculation (handles nested backticks)
        let fence = calculate_fence(code, options.code_fence);

        format!("\n\n{}{}\n{}\n{}\n\n", fence, lang, code, fence)
    }
}

/// Detect language from pre or child code element class.
fn detect_language(pre: &ElementRef) -> Option<String> {
    // Check <pre class="language-xxx">
    if let Some(class) = pre.value().attr("class") {
        if let Some(lang) = extract_language_from_class(class) {
            return Some(lang);
        }
    }

    // Check child <code class="language-xxx">
    for child in pre.children() {
        if let Some(element) = ElementRef::wrap(child) {
            if element.value().name() == "code" {
                if let Some(class) = element.value().attr("class") {
                    if let Some(lang) = extract_language_from_class(class) {
                        return Some(lang);
                    }
                }
            }
        }
    }

    None
}

/// Known programming language identifiers for bare class fallback.
static KNOWN_LANGUAGES: &[&str] = &[
    "bash", "c", "cpp", "csharp", "css", "dart", "diff", "go", "graphql", "html",
    "java", "javascript", "js", "json", "kotlin", "lua", "makefile", "markdown",
    "objectivec", "perl", "php", "plaintext", "python", "r", "ruby", "rust",
    "scala", "shell", "sql", "swift", "typescript", "ts", "xml", "yaml", "yml",
];

/// Extract language from class attribute.
fn extract_language_from_class(class: &str) -> Option<String> {
    // First pass: check for prefixed patterns (higher priority)
    for part in class.split_whitespace() {
        // language-{lang} (standard)
        if let Some(lang) = part.strip_prefix("language-") {
            return Some(lang.to_string());
        }
        // lang-{lang} (common)
        if let Some(lang) = part.strip_prefix("lang-") {
            return Some(lang.to_string());
        }
        // highlight-{lang} (some highlighters)
        if let Some(lang) = part.strip_prefix("highlight-") {
            return Some(lang.to_string());
        }
        // hljs-{lang} (highlight.js)
        if let Some(lang) = part.strip_prefix("hljs-") {
            // Skip non-language classes like hljs-keyword
            if !["keyword", "string", "number", "comment", "function", "class", "built_in"]
                .contains(&lang)
            {
                return Some(lang.to_string());
            }
        }
    }

    // Second pass: check for bare language names (fallback)
    for part in class.split_whitespace() {
        let lower = part.to_lowercase();
        if KNOWN_LANGUAGES.contains(&lower.as_str()) {
            return Some(lower);
        }
    }

    None
}

/// Calculate the fence string needed for code that may contain backticks/tildes.
fn calculate_fence(code: &str, preferred: char) -> String {
    let re = match preferred {
        '~' => &*TILDE_RE,
        _ => &*BACKTICK_RE,
    };

    let max_run = re
        .find_iter(code)
        .map(|m| m.as_str().len())
        .max()
        .unwrap_or(0);

    let fence_len = std::cmp::max(3, max_run + 1);
    std::iter::repeat_n(preferred, fence_len).collect()
}

/// Collect text from pre element, skipping gutter elements.
fn collect_code_text(pre: &ElementRef) -> String {
    let mut text = String::new();

    fn collect_recursive(node: ego_tree::NodeRef<scraper::Node>, text: &mut String, skip: bool) {
        if let Some(element) = ElementRef::wrap(node) {
            // Skip gutter/line-number elements
            if let Some(class) = element.value().attr("class") {
                if class.contains("gutter")
                    || class.contains("line-number")
                    || class.contains("line-numbers")
                    || class.contains("lineno")
                    || class.contains("linenumber")
                {
                    return;
                }
            }
        }

        match node.value() {
            scraper::Node::Text(t) => {
                if !skip {
                    text.push_str(t);
                }
            }
            scraper::Node::Element(_) => {
                for child in node.children() {
                    collect_recursive(child, text, false);
                }
            }
            _ => {}
        }
    }

    for child in pre.children() {
        collect_recursive(child, &mut text, false);
    }

    text
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

        PreRule.convert(element, &metadata, options, &|_, _, _| String::new())
    }

    #[test]
    fn test_simple_pre() {
        let result = convert_test("<pre>code here</pre>", &Options::default());
        assert!(result.contains("```"));
        assert!(result.contains("code here"));
    }

    #[test]
    fn test_pre_with_language() {
        let result = convert_test(
            r#"<pre><code class="language-rust">fn main() {}</code></pre>"#,
            &Options::default(),
        );
        assert!(result.contains("```rust"));
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_pre_with_backticks() {
        let result = convert_test("<pre>use ``` here</pre>", &Options::default());
        // Should use more than 3 backticks
        assert!(result.contains("````"));
    }

    #[test]
    fn test_tilde_fence() {
        let options = Options::new().code_fence('~');
        let result = convert_test("<pre>code</pre>", &options);
        assert!(result.contains("~~~"));
    }

    #[test]
    fn test_empty_pre() {
        let result = convert_test("<pre></pre>", &Options::default());
        assert!(result.is_empty());
    }

    #[test]
    fn test_highlight_prefix() {
        let result = convert_test(
            r#"<pre><code class="highlight-python">print("hello")</code></pre>"#,
            &Options::default(),
        );
        assert!(result.contains("```python"));
    }

    #[test]
    fn test_bare_language_fallback() {
        let result = convert_test(
            r#"<pre><code class="javascript">console.log()</code></pre>"#,
            &Options::default(),
        );
        assert!(result.contains("```javascript"));
    }

    #[test]
    fn test_lang_prefix() {
        let result = convert_test(
            r#"<pre><code class="lang-go">func main()</code></pre>"#,
            &Options::default(),
        );
        assert!(result.contains("```go"));
    }

    #[test]
    fn test_hljs_language() {
        let result = convert_test(
            r#"<pre><code class="hljs-typescript">const x: string = "test";</code></pre>"#,
            &Options::default(),
        );
        assert!(result.contains("```typescript"));
    }

    #[test]
    fn test_hljs_token_class_ignored() {
        // hljs-keyword and other token classes should not be detected as language
        let result = convert_test(
            r#"<pre><code class="hljs-keyword">const</code></pre>"#,
            &Options::default(),
        );
        // Should produce code block without language (keyword is not a language)
        assert!(result.contains("```\n"));
    }

    #[test]
    fn test_multiple_backticks_in_code() {
        // Code containing ``` needs to use ```` as fence
        let result = convert_test(
            "<pre>Here is some code:\n```\nmore code\n```</pre>",
            &Options::default(),
        );
        // Should use at least 4 backticks
        assert!(result.contains("````"));
    }

    #[test]
    fn test_gutter_classes_skipped() {
        // Line numbers should be stripped
        let result = convert_test(
            r#"<pre><code><span class="line-numbers">1</span>first line
<span class="lineno">2</span>second line</code></pre>"#,
            &Options::default(),
        );
        // Should not contain the line numbers
        assert!(!result.contains("1first"));
        assert!(!result.contains("2second"));
        assert!(result.contains("first line"));
        assert!(result.contains("second line"));
    }

    #[test]
    fn test_pre_on_element_class() {
        // Language class on <pre> element instead of <code>
        let result = convert_test(
            r#"<pre class="language-sql">SELECT * FROM users;</pre>"#,
            &Options::default(),
        );
        assert!(result.contains("```sql"));
    }

    #[test]
    fn test_multiline_code() {
        let result = convert_test(
            r#"<pre><code>line 1
line 2
line 3</code></pre>"#,
            &Options::default(),
        );
        assert!(result.contains("line 1\nline 2\nline 3"));
    }

    #[test]
    fn test_whitespace_preserved() {
        // Whitespace in pre blocks should be preserved (it's preformatted)
        let result = convert_test("<pre>    indented\n        more</pre>", &Options::default());
        assert!(result.contains("    indented"));
        assert!(result.contains("        more"));
    }
}
