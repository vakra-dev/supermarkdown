//! Table rule (GFM tables).

use once_cell::sync::Lazy;
use regex::Regex;
use scraper::ElementRef;

use crate::options::Options;
use crate::precompute::MetadataMap;
use crate::rules::Rule;

/// Regex for normalizing whitespace in cells.
static WS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

/// Column alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Alignment {
    #[default]
    None,
    Left,
    Center,
    Right,
}

/// Cell data with content and alignment.
struct CellData {
    content: String,
    alignment: Alignment,
}

pub struct TableRule;

impl Rule for TableRule {
    fn tags(&self) -> &'static [&'static str] {
        &["table"]
    }

    fn convert(
        &self,
        element: ElementRef,
        metadata: &MetadataMap,
        options: &Options,
        convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
    ) -> String {
        let mut rows: Vec<Vec<CellData>> = Vec::new();
        let mut caption: Option<String> = None;

        // Extract rows from thead, tbody, or direct tr children
        for child in element.children() {
            if let Some(el) = ElementRef::wrap(child) {
                match el.value().name() {
                    "caption" => {
                        let text = convert_children(el, metadata, options);
                        let text = WS_RE.replace_all(text.trim(), " ").to_string();
                        if !text.is_empty() {
                            caption = Some(text);
                        }
                    }
                    "thead" => {
                        extract_rows(&el, metadata, options, convert_children, &mut rows);
                    }
                    "tbody" | "tfoot" => {
                        extract_rows(&el, metadata, options, convert_children, &mut rows);
                    }
                    "tr" => {
                        if let Some(row) = extract_row(&el, metadata, options, convert_children) {
                            rows.push(row);
                        }
                    }
                    _ => {}
                }
            }
        }

        if rows.is_empty() {
            return String::new();
        }

        // Calculate column widths and alignments
        let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut col_widths: Vec<usize> = vec![3; col_count]; // minimum width of 3
        let mut col_alignments: Vec<Alignment> = vec![Alignment::None; col_count];

        for row in &rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(cell.content.chars().count());
                    // Use alignment from first row (header) if specified
                    if col_alignments[i] == Alignment::None && cell.alignment != Alignment::None {
                        col_alignments[i] = cell.alignment;
                    }
                }
            }
        }

        // Build markdown table
        let mut result = String::from("\n\n");

        for (row_idx, row) in rows.iter().enumerate() {
            result.push('|');
            for (col_idx, cell) in row.iter().enumerate() {
                let width = col_widths.get(col_idx).copied().unwrap_or(3);
                let alignment = col_alignments.get(col_idx).copied().unwrap_or(Alignment::None);

                // Format cell content with alignment
                let formatted = match alignment {
                    Alignment::Right => format!(" {:>width$} |", cell.content, width = width),
                    Alignment::Center => format!(" {:^width$} |", cell.content, width = width),
                    _ => format!(" {:width$} |", cell.content, width = width),
                };
                result.push_str(&formatted);
            }
            // Pad missing columns
            for col_idx in row.len()..col_count {
                let width = col_widths.get(col_idx).copied().unwrap_or(3);
                result.push_str(&format!(" {:width$} |", "", width = width));
            }
            result.push('\n');

            // Add separator after header row (first row)
            if row_idx == 0 {
                result.push('|');
                for (col_idx, width) in col_widths.iter().enumerate() {
                    let alignment = col_alignments.get(col_idx).copied().unwrap_or(Alignment::None);
                    let separator = match alignment {
                        Alignment::Left => format!(" :{} |", "-".repeat(*width - 1)),
                        Alignment::Center => format!(" :{}: |", "-".repeat(width.saturating_sub(2))),
                        Alignment::Right => format!(" {}: |", "-".repeat(*width - 1)),
                        Alignment::None => format!(" {} |", "-".repeat(*width)),
                    };
                    result.push_str(&separator);
                }
                result.push('\n');
            }
        }

        // Add caption if present
        if let Some(cap) = caption {
            result.push_str(&format!("\n*{}*", cap));
        }

        result.push('\n');
        result
    }
}

fn extract_rows(
    container: &ElementRef,
    metadata: &MetadataMap,
    options: &Options,
    convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
    rows: &mut Vec<Vec<CellData>>,
) {
    for child in container.children() {
        if let Some(el) = ElementRef::wrap(child) {
            if el.value().name() == "tr" {
                if let Some(row) = extract_row(&el, metadata, options, convert_children) {
                    rows.push(row);
                }
            }
        }
    }
}

fn extract_row(
    tr: &ElementRef,
    metadata: &MetadataMap,
    options: &Options,
    convert_children: &dyn Fn(ElementRef, &MetadataMap, &Options) -> String,
) -> Option<Vec<CellData>> {
    let mut cells = Vec::new();

    for child in tr.children() {
        if let Some(el) = ElementRef::wrap(child) {
            let tag = el.value().name();
            if tag == "th" || tag == "td" {
                let content = convert_children(el, metadata, options);
                let content = WS_RE.replace_all(content.trim(), " ");
                // Escape pipes in cell content
                let content = content.replace('|', "\\|");

                // Extract alignment from align attribute or style
                let alignment = extract_alignment(&el);

                cells.push(CellData { content, alignment });
            }
        }
    }

    if cells.is_empty() {
        None
    } else {
        Some(cells)
    }
}

/// Extract alignment from element's align attribute or style.
fn extract_alignment(element: &ElementRef) -> Alignment {
    // Check align attribute first
    if let Some(align) = element.value().attr("align") {
        return match align.to_lowercase().as_str() {
            "left" => Alignment::Left,
            "center" => Alignment::Center,
            "right" => Alignment::Right,
            _ => Alignment::None,
        };
    }

    // Check style attribute for text-align
    if let Some(style) = element.value().attr("style") {
        let style_lower = style.to_lowercase();
        if style_lower.contains("text-align:") || style_lower.contains("text-align :") {
            if style_lower.contains("left") {
                return Alignment::Left;
            } else if style_lower.contains("center") {
                return Alignment::Center;
            } else if style_lower.contains("right") {
                return Alignment::Right;
            }
        }
    }

    Alignment::None
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

        TableRule.convert(element, &metadata, &Options::default(), &|e, _, _| {
            e.text().collect::<Vec<_>>().join("")
        })
    }

    #[test]
    fn test_simple_table() {
        let result = convert_test(
            r#"<table>
                <tr><th>Header 1</th><th>Header 2</th></tr>
                <tr><td>Cell 1</td><td>Cell 2</td></tr>
            </table>"#,
        );
        assert!(result.contains("| Header 1"));
        assert!(result.contains("| Cell 1"));
        assert!(result.contains("---"));
    }

    #[test]
    fn test_table_with_thead_tbody() {
        let result = convert_test(
            r#"<table>
                <thead><tr><th>Name</th><th>Age</th></tr></thead>
                <tbody><tr><td>Alice</td><td>30</td></tr></tbody>
            </table>"#,
        );
        assert!(result.contains("| Name"));
        assert!(result.contains("| Alice"));
    }

    #[test]
    fn test_table_with_pipes() {
        let result = convert_test(
            r#"<table>
                <tr><th>A</th></tr>
                <tr><td>a | b</td></tr>
            </table>"#,
        );
        assert!(result.contains("a \\| b"));
    }

    #[test]
    fn test_empty_table() {
        let result = convert_test("<table></table>");
        assert!(result.is_empty());
    }

    #[test]
    fn test_table_with_alignment() {
        let result = convert_test(
            r#"<table>
                <tr>
                    <th align="left">Left</th>
                    <th align="center">Center</th>
                    <th align="right">Right</th>
                </tr>
                <tr>
                    <td align="left">L</td>
                    <td align="center">C</td>
                    <td align="right">R</td>
                </tr>
            </table>"#,
        );
        // Left alignment: :---
        assert!(result.contains(":---"));
        // Center alignment: :----: (width based on "Center")
        assert!(result.contains(":----:"));
        // Right alignment: ---:
        assert!(result.contains("----:"));
    }

    #[test]
    fn test_table_with_caption() {
        let result = convert_test(
            r#"<table>
                <caption>Monthly Sales</caption>
                <tr><th>Month</th><th>Sales</th></tr>
                <tr><td>Jan</td><td>$100</td></tr>
            </table>"#,
        );
        assert!(result.contains("| Month"));
        assert!(result.contains("*Monthly Sales*"));
    }

    #[test]
    fn test_table_with_style_alignment() {
        let result = convert_test(
            r#"<table>
                <tr>
                    <th style="text-align: right">Right</th>
                </tr>
                <tr>
                    <td style="text-align: right">R</td>
                </tr>
            </table>"#,
        );
        assert!(result.contains("---:"));
    }

    #[test]
    fn test_table_with_missing_cells() {
        // Rows with fewer cells than header should be padded
        let result = convert_test(
            r#"<table>
                <tr><th>A</th><th>B</th><th>C</th></tr>
                <tr><td>1</td><td>2</td></tr>
                <tr><td>X</td></tr>
            </table>"#,
        );
        // All rows should have same number of columns
        let lines: Vec<&str> = result.lines().filter(|l| l.contains('|')).collect();
        assert!(lines.len() >= 3); // header + separator + 2 data rows

        // Count pipes in each row - should be consistent
        let header_pipes = lines.first().map(|l| l.matches('|').count()).unwrap_or(0);
        for line in &lines {
            assert_eq!(line.matches('|').count(), header_pipes);
        }
    }

    #[test]
    fn test_table_with_tfoot() {
        let result = convert_test(
            r#"<table>
                <thead><tr><th>Item</th><th>Price</th></tr></thead>
                <tbody><tr><td>Apple</td><td>$1</td></tr></tbody>
                <tfoot><tr><td>Total</td><td>$1</td></tr></tfoot>
            </table>"#,
        );
        assert!(result.contains("| Item"));
        assert!(result.contains("| Apple"));
        assert!(result.contains("| Total"));
    }

    #[test]
    fn test_table_header_only() {
        // Table with only a header row
        let result = convert_test(
            r#"<table>
                <tr><th>Col A</th><th>Col B</th></tr>
            </table>"#,
        );
        assert!(result.contains("| Col A"));
        assert!(result.contains("---"));
    }
}
