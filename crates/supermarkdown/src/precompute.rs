//! Pre-computation module for O(n) metadata extraction.
//!
//! This module performs a single DFS traversal to compute all node metadata,
//! avoiding O(n²) traversals for nested lists and selector matching.

use ego_tree::NodeId;
use rustc_hash::FxHashMap;
use scraper::{ElementRef, Html, Selector};

use crate::options::Options;

/// Pre-computed metadata for O(1) access during conversion.
#[derive(Debug, Default, Clone)]
pub struct NodeMetadata {
    /// For `<li>`: the prefix string ("- ", "1. ", "2. ", etc.)
    pub list_prefix: Option<String>,

    /// For `<li>`: total indentation from all ancestor lists (in spaces)
    pub ancestor_indent: usize,

    /// Should skip this node and its subtree (matches exclude selector)
    pub skip: bool,

    /// Force keep this node (matches include selector, overrides parent skip)
    pub force_keep: bool,
}

/// Type alias for the metadata map.
pub type MetadataMap = FxHashMap<NodeId, NodeMetadata>;

/// Compiled CSS selectors for efficient matching.
pub struct CompiledSelectors {
    pub exclude: Vec<Selector>,
    pub include: Vec<Selector>,
}

impl CompiledSelectors {
    /// Compile selectors from options.
    pub fn new(options: &Options) -> Self {
        Self {
            exclude: options
                .exclude_selectors
                .iter()
                .filter_map(|s| compile_selector(s))
                .collect(),
            include: options
                .include_selectors
                .iter()
                .filter_map(|s| compile_selector(s))
                .collect(),
        }
    }

    /// Check if an element matches any exclude selector.
    pub fn matches_exclude(&self, element: &ElementRef) -> bool {
        self.exclude.iter().any(|sel| sel.matches(element))
    }

    /// Check if an element matches any include selector.
    pub fn matches_include(&self, element: &ElementRef) -> bool {
        self.include.iter().any(|sel| sel.matches(element))
    }
}

/// Compile a CSS selector string, returning None on error.
fn compile_selector(selector: &str) -> Option<Selector> {
    #[cfg(feature = "logging")]
    if let Err(e) = Selector::parse(selector) {
        log::warn!("Invalid selector '{}': {:?}", selector, e);
        return None;
    }
    Selector::parse(selector).ok()
}

/// Context for tracking list state during traversal.
struct ListContext {
    /// Whether this is an ordered list.
    ordered: bool,
    /// Current item index (1-based).
    index: usize,
    /// Indentation from ancestor lists.
    indent: usize,
    /// Length of the prefix (e.g., "- " is 2, "10. " is 4).
    prefix_len: usize,
}

/// Single O(n) traversal to compute all node metadata.
pub fn precompute_metadata(
    dom: &Html,
    selectors: &CompiledSelectors,
    options: &Options,
) -> MetadataMap {
    let mut metadata = FxHashMap::default();
    let mut list_stack: Vec<ListContext> = Vec::with_capacity(8);
    let mut skip_depth: Option<usize> = None;
    let mut depth: usize = 0;

    // Use scraper's select to traverse all elements
    // We'll use a manual traversal for proper edge handling
    let root = dom.root_element();

    fn traverse(
        node: ego_tree::NodeRef<scraper::Node>,
        metadata: &mut MetadataMap,
        list_stack: &mut Vec<ListContext>,
        skip_depth: &mut Option<usize>,
        depth: &mut usize,
        selectors: &CompiledSelectors,
        options: &Options,
    ) {
        *depth += 1;

        if let Some(element) = ElementRef::wrap(node) {
            let tag = element.value().name();

            // Track list context
            if tag == "ul" || tag == "ol" {
                let current_indent = list_stack
                    .last()
                    .map(|ctx| ctx.indent + ctx.prefix_len)
                    .unwrap_or(0);

                // Check for start attribute on ordered lists
                let start_index = if tag == "ol" {
                    element
                        .value()
                        .attr("start")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(1)
                        .saturating_sub(1) // Subtract 1 because we increment before use
                } else {
                    0
                };

                list_stack.push(ListContext {
                    ordered: tag == "ol",
                    index: start_index,
                    indent: current_indent,
                    prefix_len: 2, // Will be updated when processing li
                });
            }

            // Compute list item metadata
            if tag == "li" {
                if let Some(ctx) = list_stack.last_mut() {
                    ctx.index += 1;

                    let prefix = if ctx.ordered {
                        format!("{}. ", ctx.index)
                    } else {
                        format!("{} ", options.bullet_marker)
                    };

                    ctx.prefix_len = prefix.len();

                    let meta = metadata.entry(node.id()).or_default();
                    meta.list_prefix = Some(prefix);
                    meta.ancestor_indent = ctx.indent;
                }
            }

            // Check include selectors first (force_keep)
            let force_keep = selectors.matches_include(&element);

            // Check exclude selectors
            let matches_exclude = selectors.matches_exclude(&element);

            // Determine skip state
            let inherited_skip = skip_depth.is_some();
            let skip = if force_keep {
                false // force_keep overrides everything
            } else if matches_exclude {
                if skip_depth.is_none() {
                    *skip_depth = Some(*depth);
                }
                true
            } else {
                inherited_skip
            };

            if skip || force_keep {
                let meta = metadata.entry(node.id()).or_default();
                meta.skip = skip;
                meta.force_keep = force_keep;
            }
        }

        // Recurse into children
        for child in node.children() {
            traverse(
                child, metadata, list_stack, skip_depth, depth, selectors, options,
            );
        }

        // Handle exit
        if let Some(element) = ElementRef::wrap(node) {
            let tag = element.value().name();
            if tag == "ul" || tag == "ol" {
                list_stack.pop();
            }
        }

        // Reset skip_depth when leaving the element that started the skip
        if *skip_depth == Some(*depth) {
            *skip_depth = None;
        }
        *depth -= 1;
    }

    // Get the underlying node reference from the root element
    for child in root.children() {
        traverse(
            child,
            &mut metadata,
            &mut list_stack,
            &mut skip_depth,
            &mut depth,
            selectors,
            options,
        );
    }

    metadata
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_metadata() {
        let html = r#"<ul><li>First</li><li>Second</li></ul>"#;
        let dom = Html::parse_document(html);
        let options = Options::default();
        let selectors = CompiledSelectors::new(&options);
        let metadata = precompute_metadata(&dom, &selectors, &options);

        // Should have metadata for the two li elements
        let li_metadata: Vec<_> = metadata
            .values()
            .filter(|m| m.list_prefix.is_some())
            .collect();
        assert_eq!(li_metadata.len(), 2);
    }

    #[test]
    fn test_nested_list_indentation() {
        let html = r#"<ul><li>First<ul><li>Nested</li></ul></li></ul>"#;
        let dom = Html::parse_document(html);
        let options = Options::default();
        let selectors = CompiledSelectors::new(&options);
        let metadata = precompute_metadata(&dom, &selectors, &options);

        let li_metadata: Vec<_> = metadata
            .values()
            .filter(|m| m.list_prefix.is_some())
            .collect();

        // Check that nested item has indentation
        let has_nested = li_metadata.iter().any(|m| m.ancestor_indent > 0);
        assert!(has_nested);
    }

    #[test]
    fn test_ordered_list() {
        let html = r#"<ol><li>First</li><li>Second</li><li>Third</li></ol>"#;
        let dom = Html::parse_document(html);
        let options = Options::default();
        let selectors = CompiledSelectors::new(&options);
        let metadata = precompute_metadata(&dom, &selectors, &options);

        let prefixes: Vec<_> = metadata
            .values()
            .filter_map(|m| m.list_prefix.as_ref())
            .collect();

        assert!(prefixes.contains(&&"1. ".to_string()));
        assert!(prefixes.contains(&&"2. ".to_string()));
        assert!(prefixes.contains(&&"3. ".to_string()));
    }

    #[test]
    fn test_ordered_list_with_start() {
        let html = r#"<ol start="5"><li>Fifth</li><li>Sixth</li><li>Seventh</li></ol>"#;
        let dom = Html::parse_document(html);
        let options = Options::default();
        let selectors = CompiledSelectors::new(&options);
        let metadata = precompute_metadata(&dom, &selectors, &options);

        let prefixes: Vec<_> = metadata
            .values()
            .filter_map(|m| m.list_prefix.as_ref())
            .collect();

        assert!(prefixes.contains(&&"5. ".to_string()));
        assert!(prefixes.contains(&&"6. ".to_string()));
        assert!(prefixes.contains(&&"7. ".to_string()));
    }

    #[test]
    fn test_exclude_selector() {
        let html = r#"<div><p>Keep</p><nav>Skip</nav></div>"#;
        let dom = Html::parse_document(html);
        let options = Options {
            exclude_selectors: vec!["nav".to_string()],
            ..Default::default()
        };
        let selectors = CompiledSelectors::new(&options);
        let metadata = precompute_metadata(&dom, &selectors, &options);

        let skipped: Vec<_> = metadata.values().filter(|m| m.skip).collect();
        assert!(!skipped.is_empty());
    }

    #[test]
    fn test_include_overrides_exclude() {
        let html = r#"<nav><div class="keep">Important</div></nav>"#;
        let dom = Html::parse_document(html);
        let options = Options {
            exclude_selectors: vec!["nav".to_string()],
            include_selectors: vec![".keep".to_string()],
            ..Default::default()
        };
        let selectors = CompiledSelectors::new(&options);
        let metadata = precompute_metadata(&dom, &selectors, &options);

        let force_kept: Vec<_> = metadata.values().filter(|m| m.force_keep).collect();
        assert!(!force_kept.is_empty());
    }
}
