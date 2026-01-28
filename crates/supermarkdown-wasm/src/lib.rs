use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use supermarkdown::{HeadingStyle, LinkStyle, Options};

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertOptions {
    pub heading_style: Option<String>,
    pub link_style: Option<String>,
    pub code_fence: Option<String>,
    pub bullet_marker: Option<String>,
    pub base_url: Option<String>,
    pub exclude_selectors: Option<Vec<String>>,
    pub include_selectors: Option<Vec<String>>,
}

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
        options = options.code_fence(fence.chars().next().unwrap_or('`'));
    }

    if let Some(marker) = opts.bullet_marker {
        options = options.bullet_marker(marker.chars().next().unwrap_or('-'));
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

/// Convert HTML to Markdown with default options
#[wasm_bindgen]
pub fn convert(html: &str) -> String {
    supermarkdown::convert(html)
}

/// Convert HTML to Markdown with custom options
#[wasm_bindgen(js_name = convertWithOptions)]
pub fn convert_with_options(html: &str, options: JsValue) -> Result<String, JsError> {
    let opts: Option<ConvertOptions> = if options.is_undefined() || options.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(options)?)
    };

    let internal_opts = to_internal_options(opts);
    Ok(supermarkdown::convert_with_options(html, &internal_opts))
}
