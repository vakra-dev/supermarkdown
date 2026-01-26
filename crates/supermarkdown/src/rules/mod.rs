//! Conversion rules for HTML elements.
//!
//! Each rule handles specific HTML tags and converts them to markdown.

mod blockquote;
mod br;
mod code;
mod deflist;
mod details;
mod emphasis;
mod figure;
mod heading;
mod hr;
mod image;
mod link;
mod list;
mod paragraph;
mod passthrough;
mod pre;
mod strikethrough;
mod subscript;
mod superscript;
mod table;

use scraper::ElementRef;

use crate::options::Options;
use crate::precompute::MetadataMap;

pub use blockquote::BlockquoteRule;
pub use br::BreakRule;
pub use code::CodeRule;
pub use deflist::{DefDescRule, DefListRule, DefTermRule};
pub use details::DetailsRule;
pub use emphasis::{EmphasisRule, StrongRule};
pub use figure::FigureRule;
pub use heading::HeadingRule;
pub use hr::HorizontalRule;
pub use image::ImageRule;
pub use link::LinkRule;
pub use list::{ListItemRule, ListRule};
pub use paragraph::ParagraphRule;
pub use passthrough::{AbbrRule, KbdRule, MarkRule, SampRule, VarRule};
pub use pre::PreRule;
pub use strikethrough::StrikethroughRule;
pub use subscript::SubscriptRule;
pub use superscript::SuperscriptRule;
pub use table::TableRule;

/// Trait for HTML to Markdown conversion rules.
pub trait Rule: Send + Sync {
    /// Tags this rule handles.
    fn tags(&self) -> &'static [&'static str];

    /// Convert the element to markdown.
    ///
    /// # Arguments
    ///
    /// * `element` - The HTML element to convert
    /// * `metadata` - Pre-computed metadata for O(1) lookups
    /// * `options` - Conversion options
    /// * `convert_children` - Function to convert child nodes
    fn convert(
        &self,
        element: ElementRef,
        metadata: &MetadataMap,
        options: &Options,
        convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
    ) -> String;
}

/// Get the default set of conversion rules.
pub fn default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // Block elements
        Box::new(HeadingRule),
        Box::new(ParagraphRule),
        Box::new(PreRule),
        Box::new(BlockquoteRule),
        Box::new(ListRule),
        Box::new(ListItemRule),
        Box::new(DefListRule),
        Box::new(DefTermRule),
        Box::new(DefDescRule),
        Box::new(TableRule),
        Box::new(HorizontalRule),
        Box::new(DetailsRule),
        Box::new(FigureRule),
        // Inline elements
        Box::new(LinkRule),
        Box::new(ImageRule),
        Box::new(StrongRule),
        Box::new(EmphasisRule),
        Box::new(StrikethroughRule),
        Box::new(CodeRule),
        Box::new(SuperscriptRule),
        Box::new(SubscriptRule),
        Box::new(BreakRule),
        // HTML passthrough elements
        Box::new(KbdRule),
        Box::new(MarkRule),
        Box::new(AbbrRule),
        Box::new(SampRule),
        Box::new(VarRule),
    ]
}

/// Find a rule that handles the given tag.
pub fn find_rule<'a>(rules: &'a [Box<dyn Rule>], tag: &str) -> Option<&'a dyn Rule> {
    rules
        .iter()
        .find(|rule| rule.tags().contains(&tag))
        .map(|r| r.as_ref())
}
