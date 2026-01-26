//! Node.js bindings for supermarkdown.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use supermarkdown::{HeadingStyle, LinkStyle, Options};

/// Options for HTML to Markdown conversion.
#[derive(Default)]
#[napi(object)]
pub struct ConvertOptions {
    /// Heading style: "atx" (default) or "setext"
    pub heading_style: Option<String>,
    /// Link style: "inline" (default) or "referenced"
    pub link_style: Option<String>,
    /// Code fence character: "`" (default) or "~"
    pub code_fence: Option<String>,
    /// Bullet marker for unordered lists: "-" (default), "*", or "+"
    pub bullet_marker: Option<String>,
    /// Base URL for resolving relative links
    pub base_url: Option<String>,
    /// CSS selectors for elements to exclude
    pub exclude_selectors: Option<Vec<String>>,
    /// CSS selectors for elements to force keep (overrides excludes)
    pub include_selectors: Option<Vec<String>>,
}

/// Convert ConvertOptions to internal Options.
fn to_internal_options(opts: Option<ConvertOptions>) -> Options {
    let opts = opts.unwrap_or_default();
    let mut options = Options::new();

    if let Some(style) = opts.heading_style {
        options = match style.to_lowercase().as_str() {
            "setext" => options.heading_style(HeadingStyle::Setext),
            _ => options.heading_style(HeadingStyle::Atx),
        };
    }

    if let Some(style) = opts.link_style {
        options = match style.to_lowercase().as_str() {
            "referenced" | "reference" => options.link_style(LinkStyle::Referenced),
            _ => options.link_style(LinkStyle::Inline),
        };
    }

    if let Some(fence) = opts.code_fence {
        let fence_char = fence.chars().next().unwrap_or('`');
        options = options.code_fence(fence_char);
    }

    if let Some(marker) = opts.bullet_marker {
        let marker_char = marker.chars().next().unwrap_or('-');
        options = options.bullet_marker(marker_char);
    }

    if let Some(url) = opts.base_url {
        options = options.base_url(Some(url));
    }

    if let Some(selectors) = opts.exclude_selectors {
        options = options.exclude_selectors(selectors);
    }

    if let Some(selectors) = opts.include_selectors {
        options = options.include_selectors(selectors);
    }

    options
}

/// Convert HTML to Markdown synchronously.
///
/// @param html - The HTML string to convert
/// @param options - Optional conversion options
/// @returns The converted Markdown string
#[napi]
pub fn convert(html: String, options: Option<ConvertOptions>) -> String {
    let opts = to_internal_options(options);
    supermarkdown::convert_with_options(&html, &opts)
}

/// Convert HTML to Markdown asynchronously.
///
/// This is useful for large documents to avoid blocking the main thread.
///
/// @param html - The HTML string to convert
/// @param options - Optional conversion options
/// @returns A promise that resolves to the converted Markdown string
#[napi]
pub async fn convert_async(html: String, options: Option<ConvertOptions>) -> Result<String> {
    let opts = to_internal_options(options);

    // Use tokio's spawn_blocking to run the CPU-intensive conversion
    // on a separate thread pool, avoiding blocking the Node.js event loop
    let result =
        tokio::task::spawn_blocking(move || supermarkdown::convert_with_options(&html, &opts))
            .await
            .map_err(|e| Error::from_reason(format!("Conversion failed: {}", e)))?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_basic() {
        let html = "<h1>Hello</h1><p>World</p>";
        let result = convert(html.to_string(), None);
        assert!(result.contains("# Hello"));
        assert!(result.contains("World"));
    }

    #[test]
    fn test_convert_with_options() {
        let html = "<h1>Title</h1>";
        let options = ConvertOptions {
            heading_style: Some("setext".to_string()),
            link_style: None,
            code_fence: None,
            bullet_marker: None,
            base_url: None,
            exclude_selectors: None,
            include_selectors: None,
        };
        let result = convert(html.to_string(), Some(options));
        assert!(result.contains("====="));
    }

    #[test]
    fn test_convert_with_exclude() {
        let html = "<div><nav>Skip</nav><p>Keep</p></div>";
        let options = ConvertOptions {
            heading_style: None,
            link_style: None,
            code_fence: None,
            bullet_marker: None,
            base_url: None,
            exclude_selectors: Some(vec!["nav".to_string()]),
            include_selectors: None,
        };
        let result = convert(html.to_string(), Some(options));
        assert!(!result.contains("Skip"));
        assert!(result.contains("Keep"));
    }
}
