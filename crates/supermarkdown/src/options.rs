//! Configuration options for HTML to Markdown conversion.

/// Configuration options for HTML to Markdown conversion.
#[derive(Debug, Clone)]
pub struct Options {
    /// CSS selectors for elements to exclude from output.
    /// Default: []
    pub exclude_selectors: Vec<String>,

    /// CSS selectors for elements to always include (overrides exclude).
    /// Default: []
    pub include_selectors: Vec<String>,

    /// Heading style: ATX (###) or Setext (underline).
    /// Default: Atx
    pub heading_style: HeadingStyle,

    /// Character for code fences.
    /// Default: '`'
    pub code_fence: char,

    /// Link style: Inline `[text](url)` or Referenced `[text][1]`.
    /// Default: Inline
    pub link_style: LinkStyle,

    /// Bullet character for unordered lists.
    /// Default: '-'
    pub bullet_marker: char,

    /// Base URL for resolving relative links.
    /// Default: None
    pub base_url: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            exclude_selectors: vec![],
            include_selectors: vec![],
            heading_style: HeadingStyle::Atx,
            code_fence: '`',
            link_style: LinkStyle::Inline,
            bullet_marker: '-',
            base_url: None,
        }
    }
}

impl Options {
    /// Create a new Options with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set CSS selectors to exclude.
    pub fn exclude_selectors(mut self, selectors: Vec<String>) -> Self {
        self.exclude_selectors = selectors;
        self
    }

    /// Set CSS selectors to always include.
    pub fn include_selectors(mut self, selectors: Vec<String>) -> Self {
        self.include_selectors = selectors;
        self
    }

    /// Set heading style.
    pub fn heading_style(mut self, style: HeadingStyle) -> Self {
        self.heading_style = style;
        self
    }

    /// Set code fence character.
    pub fn code_fence(mut self, fence: char) -> Self {
        self.code_fence = fence;
        self
    }

    /// Set link style.
    pub fn link_style(mut self, style: LinkStyle) -> Self {
        self.link_style = style;
        self
    }

    /// Set bullet marker for unordered lists.
    pub fn bullet_marker(mut self, marker: char) -> Self {
        self.bullet_marker = marker;
        self
    }

    /// Set base URL for resolving relative links.
    pub fn base_url(mut self, url: Option<String>) -> Self {
        self.base_url = url;
        self
    }
}

/// Heading style for markdown output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HeadingStyle {
    /// ATX style: ### Heading
    #[default]
    Atx,
    /// Setext style: Heading\n=======
    Setext,
}

/// Link style for markdown output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LinkStyle {
    /// Inline style: `[text](url)`
    #[default]
    Inline,
    /// Referenced style: `[text][1] ... [1]: url`
    Referenced,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = Options::default();
        assert!(opts.exclude_selectors.is_empty());
        assert!(opts.include_selectors.is_empty());
        assert_eq!(opts.heading_style, HeadingStyle::Atx);
        assert_eq!(opts.code_fence, '`');
        assert_eq!(opts.link_style, LinkStyle::Inline);
        assert_eq!(opts.bullet_marker, '-');
        assert!(opts.base_url.is_none());
    }

    #[test]
    fn test_builder_pattern() {
        let opts = Options::new()
            .heading_style(HeadingStyle::Setext)
            .link_style(LinkStyle::Referenced)
            .bullet_marker('*')
            .code_fence('~')
            .base_url(Some("https://example.com".to_string()));

        assert_eq!(opts.heading_style, HeadingStyle::Setext);
        assert_eq!(opts.link_style, LinkStyle::Referenced);
        assert_eq!(opts.bullet_marker, '*');
        assert_eq!(opts.code_fence, '~');
        assert_eq!(opts.base_url, Some("https://example.com".to_string()));
    }
}
