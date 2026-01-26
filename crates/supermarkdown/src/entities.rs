//! HTML entity decoding.

use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::FxHashMap;

/// Static map of common HTML entities to their character equivalents.
static ENTITIES: Lazy<FxHashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = FxHashMap::default();
    // Essential entities
    m.insert("&amp;", "&");
    m.insert("&lt;", "<");
    m.insert("&gt;", ">");
    m.insert("&quot;", "\"");
    m.insert("&apos;", "'");
    m.insert("&nbsp;", " ");
    // Symbols
    m.insert("&copy;", "©");
    m.insert("&reg;", "®");
    m.insert("&trade;", "™");
    // Punctuation
    m.insert("&mdash;", "—");
    m.insert("&ndash;", "–");
    m.insert("&hellip;", "…");
    m.insert("&bull;", "•");
    m.insert("&middot;", "·");
    // Quotes
    m.insert("&lsquo;", "\u{2018}"); // '
    m.insert("&rsquo;", "\u{2019}"); // '
    m.insert("&ldquo;", "\u{201C}"); // "
    m.insert("&rdquo;", "\u{201D}"); // "
    m.insert("&laquo;", "\u{00AB}"); // «
    m.insert("&raquo;", "»");
    // Math
    m.insert("&times;", "×");
    m.insert("&divide;", "÷");
    m.insert("&plusmn;", "±");
    m.insert("&minus;", "−");
    m.insert("&le;", "≤");
    m.insert("&ge;", "≥");
    m.insert("&ne;", "≠");
    m.insert("&infin;", "∞");
    // Currency
    m.insert("&cent;", "¢");
    m.insert("&pound;", "£");
    m.insert("&euro;", "€");
    m.insert("&yen;", "¥");
    // Arrows
    m.insert("&larr;", "←");
    m.insert("&rarr;", "→");
    m.insert("&uarr;", "↑");
    m.insert("&darr;", "↓");
    m
});

/// Regex for matching HTML entities (named, decimal, and hex).
static ENTITY_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&(?:#(\d+)|#x([0-9a-fA-F]+)|(\w+));").unwrap());

/// Decode HTML entities in text.
///
/// Handles:
/// - Named entities: `&amp;` → `&`
/// - Decimal numeric entities: `&#123;` → `{`
/// - Hexadecimal numeric entities: `&#x7B;` → `{`
///
/// Unrecognized entities are left as-is.
pub fn decode_entities(text: &str) -> String {
    if !text.contains('&') {
        return text.to_string();
    }

    ENTITY_RE
        .replace_all(text, |caps: &regex::Captures| {
            // Numeric decimal: &#123;
            if let Some(decimal) = caps.get(1) {
                if let Ok(code) = decimal.as_str().parse::<u32>() {
                    if let Some(c) = char::from_u32(code) {
                        return c.to_string();
                    }
                }
            }
            // Numeric hex: &#x7B;
            if let Some(hex) = caps.get(2) {
                if let Ok(code) = u32::from_str_radix(hex.as_str(), 16) {
                    if let Some(c) = char::from_u32(code) {
                        return c.to_string();
                    }
                }
            }
            // Named entity: &amp;
            if let Some(name) = caps.get(3) {
                let entity = format!("&{};", name.as_str());
                if let Some(replacement) = ENTITIES.get(entity.as_str()) {
                    return (*replacement).to_string();
                }
            }
            // Return original if not recognized
            caps[0].to_string()
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_named_entities() {
        assert_eq!(decode_entities("&amp;"), "&");
        assert_eq!(decode_entities("&lt;"), "<");
        assert_eq!(decode_entities("&gt;"), ">");
        assert_eq!(decode_entities("&quot;"), "\"");
        assert_eq!(decode_entities("&apos;"), "'");
        assert_eq!(decode_entities("&nbsp;"), " ");
    }

    #[test]
    fn test_numeric_decimal() {
        assert_eq!(decode_entities("&#38;"), "&");
        assert_eq!(decode_entities("&#60;"), "<");
        assert_eq!(decode_entities("&#62;"), ">");
        assert_eq!(decode_entities("&#123;"), "{");
    }

    #[test]
    fn test_numeric_hex() {
        assert_eq!(decode_entities("&#x26;"), "&");
        assert_eq!(decode_entities("&#x3C;"), "<");
        assert_eq!(decode_entities("&#x3E;"), ">");
        assert_eq!(decode_entities("&#x7B;"), "{");
    }

    #[test]
    fn test_mixed() {
        assert_eq!(
            decode_entities("Hello &amp; World &lt;test&gt;"),
            "Hello & World <test>"
        );
    }

    #[test]
    fn test_unknown_entity() {
        assert_eq!(decode_entities("&unknown;"), "&unknown;");
    }

    #[test]
    fn test_no_entities() {
        assert_eq!(decode_entities("Hello World"), "Hello World");
    }

    #[test]
    fn test_special_chars() {
        assert_eq!(decode_entities("&mdash;"), "—");
        assert_eq!(decode_entities("&hellip;"), "…");
        assert_eq!(
            decode_entities("&ldquo;test&rdquo;"),
            "\u{201C}test\u{201D}"
        );
    }
}
