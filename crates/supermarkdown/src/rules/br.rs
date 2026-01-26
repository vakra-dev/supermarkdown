//! Line break rule.

use scraper::ElementRef;

use crate::options::Options;
use crate::precompute::MetadataMap;
use crate::rules::Rule;

pub struct BreakRule;

impl Rule for BreakRule {
    fn tags(&self) -> &'static [&'static str] {
        &["br"]
    }

    fn convert(
        &self,
        _element: ElementRef,
        _metadata: &MetadataMap,
        _options: &Options,
        _convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
    ) -> String {
        // Use two trailing spaces for line break (CommonMark)
        "  \n".to_string()
    }
}
