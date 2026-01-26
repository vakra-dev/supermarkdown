# supermarkdown

[![npm version](https://img.shields.io/npm/v/supermarkdown.svg)](https://www.npmjs.com/package/supermarkdown)
[![crates.io](https://img.shields.io/crates/v/supermarkdown.svg)](https://crates.io/crates/supermarkdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

High-performance HTML to Markdown converter with full GitHub Flavored Markdown support. Written in Rust, available for Node.js and as a native Rust crate.

## Features

- **Fast** - Written in Rust with O(n) algorithms, significantly faster than JavaScript alternatives
- **Full GFM Support** - Tables with alignment, strikethrough, autolinks, fenced code blocks
- **Accurate** - Handles malformed HTML gracefully via html5ever
- **Configurable** - Multiple heading styles, link styles, custom selectors
- **Zero Dependencies** - Single native binary, no JavaScript runtime overhead
- **Cross-Platform** - Pre-built binaries for Windows, macOS, and Linux (x64 & ARM64)
- **TypeScript Ready** - Full type definitions included
- **Async Support** - Non-blocking conversion for large documents

## Installation

```bash
npm install supermarkdown
```

## Quick Start

```javascript
import { convert } from "supermarkdown";

const html = `
  <h1>Hello World</h1>
  <p>This is a <strong>test</strong> with a <a href="https://example.com">link</a>.</p>
`;

const markdown = convert(html);
console.log(markdown);
// # Hello World
//
// This is a **test** with a [link](https://example.com).
```

## Usage

### Basic Conversion

```javascript
import { convert } from "supermarkdown";

const markdown = convert("<h1>Title</h1><p>Paragraph</p>");
```

### With Options

```javascript
import { convert } from "supermarkdown";

const markdown = convert(html, {
  headingStyle: "setext", // 'atx' (default) or 'setext'
  linkStyle: "referenced", // 'inline' (default) or 'referenced'
  excludeSelectors: ["nav", ".sidebar", "#ads"],
  includeSelectors: [".important"], // Override excludes for specific elements
});
```

### Async Conversion

For large documents, use `convertAsync` to avoid blocking the main thread:

```javascript
import { convertAsync } from "supermarkdown";

const markdown = await convertAsync(largeHtml);

// Process multiple documents in parallel
const results = await Promise.all([
  convertAsync(html1),
  convertAsync(html2),
  convertAsync(html3),
]);
```

## API Reference

### `convert(html, options?)`

Converts HTML to Markdown synchronously.

**Parameters:**

- `html` (string) - The HTML string to convert
- `options` (object, optional) - Conversion options

**Returns:** string - The converted Markdown

### `convertAsync(html, options?)`

Converts HTML to Markdown asynchronously.

**Parameters:**

- `html` (string) - The HTML string to convert
- `options` (object, optional) - Conversion options

**Returns:** Promise<string> - The converted Markdown

### Options

| Option             | Type                         | Default     | Description                                      |
| ------------------ | ---------------------------- | ----------- | ------------------------------------------------ |
| `headingStyle`     | `'atx'` \| `'setext'`        | `'atx'`     | ATX uses `#` prefix, Setext uses underlines      |
| `linkStyle`        | `'inline'` \| `'referenced'` | `'inline'`  | Inline: `[text](url)`, Referenced: `[text][1]`   |
| `codeFence`        | `` '`' `` \| `'~'`           | `` '`' ``   | Character for fenced code blocks                 |
| `bulletMarker`     | `'-'` \| `'*'` \| `'+'`      | `'-'`       | Character for unordered list items               |
| `baseUrl`          | `string`                     | `undefined` | Base URL for resolving relative links            |
| `excludeSelectors` | `string[]`                   | `[]`        | CSS selectors for elements to exclude            |
| `includeSelectors` | `string[]`                   | `[]`        | CSS selectors to force keep (overrides excludes) |

## Supported Elements

### Block Elements

| HTML                       | Markdown                                       |
| -------------------------- | ---------------------------------------------- |
| `<h1>` - `<h6>`            | `#` headings or setext underlines              |
| `<p>`                      | Paragraphs with blank lines                    |
| `<blockquote>`             | `>` quoted blocks (supports nesting)           |
| `<ul>`, `<ol>`             | `-` or `1.` lists (supports `start` attribute) |
| `<pre><code>`              | Fenced code blocks with language detection     |
| `<table>`                  | GFM tables with alignment and captions         |
| `<hr>`                     | `---` horizontal rules                         |
| `<dl>`, `<dt>`, `<dd>`     | Definition lists                               |
| `<details>`, `<summary>`   | Collapsible sections                           |
| `<figure>`, `<figcaption>` | Images with captions                           |

### Inline Elements

| HTML                       | Markdown                                |
| -------------------------- | --------------------------------------- |
| `<a>`                      | `[text](url)`, `[text][ref]`, or `<url>` (autolink) |
| `<img>`                    | `![alt](src)`                           |
| `<strong>`, `<b>`          | `**bold**`                              |
| `<em>`, `<i>`              | `*italic*`                              |
| `<code>`                   | `` `code` `` (handles nested backticks) |
| `<del>`, `<s>`, `<strike>` | `~~strikethrough~~`                     |
| `<sub>`                    | `<sub>subscript</sub>`                  |
| `<sup>`                    | `<sup>superscript</sup>`                |
| `<br>`                     | Line breaks                             |

### HTML Passthrough

Elements without Markdown equivalents are preserved as HTML:

- `<kbd>` - Keyboard input
- `<mark>` - Highlighted text
- `<abbr>` - Abbreviations (preserves `title` attribute)
- `<samp>` - Sample output
- `<var>` - Variables

## Advanced Features

### Table Alignment

Extracts alignment from `align` attribute or `text-align` style:

```html
<table>
  <tr>
    <th align="left">Left</th>
    <th align="center">Center</th>
    <th align="right">Right</th>
  </tr>
</table>
```

Output:

```markdown
| Left | Center | Right |
| :--- | :----: | ----: |
```

### Ordered List Start

Respects the `start` attribute on ordered lists:

```html
<ol start="5">
  <li>Fifth item</li>
  <li>Sixth item</li>
</ol>
```

Output:

```markdown
5. Fifth item
6. Sixth item
```

### Autolinks

When a link's text matches its URL or email, autolink syntax is used:

```html
<a href="https://example.com">https://example.com</a>
<a href="mailto:test@example.com">test@example.com</a>
```

Output:

```markdown
<https://example.com>
<test@example.com>
```

### Code Block Language Detection

Automatically detects language from class names:

- `language-*` (e.g., `language-rust`)
- `lang-*` (e.g., `lang-python`)
- `highlight-*` (e.g., `highlight-go`)
- `hljs-*` (highlight.js classes, excluding token classes like `hljs-keyword`)
- Bare language names (e.g., `javascript`, `python`) as fallback

```html
<pre><code class="language-rust">fn main() {}</code></pre>
```

Output:

````markdown
```rust
fn main() {}
```
````

Code blocks containing backticks automatically use more backticks as delimiters.

### Line Number Handling

Line number gutters are automatically stripped from code blocks. Elements with these class patterns are skipped:

- `gutter`
- `line-number`
- `line-numbers`
- `lineno`
- `linenumber`

### URL Encoding

Spaces and parentheses in URLs are automatically percent-encoded:

```javascript
// <a href="https://example.com/path (1)">link</a>
// → [link](https://example.com/path%20%281%29)
```

### Selector-Based Filtering

Remove unwanted elements like navigation, ads, or sidebars:

```javascript
const markdown = convert(html, {
  excludeSelectors: [
    "nav",
    "header",
    "footer",
    ".sidebar",
    ".advertisement",
    "#cookie-banner",
  ],
  includeSelectors: [".main-content"],
});
```

## Limitations

Some HTML features cannot be fully represented in Markdown:

| Feature                 | Behavior                                   |
| ----------------------- | ------------------------------------------ |
| Table colspan/rowspan   | Content placed in first cell               |
| Nested tables           | Inner tables converted inline              |
| Form elements           | Skipped                                    |
| iframe/video/audio      | Skipped (no standard Markdown equivalent)  |
| CSS styling             | Ignored (except `text-align` for tables)   |
| Empty elements          | Removed from output                        |

## Rust Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
supermarkdown = "0.0.1"
```

```rust
use supermarkdown::{convert, convert_with_options, Options, HeadingStyle};

// Basic conversion
let markdown = convert("<h1>Hello</h1>");

// With options
let options = Options::new()
    .heading_style(HeadingStyle::Setext)
    .exclude_selectors(vec!["nav".to_string()]);

let markdown = convert_with_options("<h1>Hello</h1>", &options);
```

## Performance

supermarkdown is designed for high performance:

- **Single-pass parsing** - O(n) HTML traversal
- **Pre-computed metadata** - List indices and CSS selectors computed in one pass
- **Zero-copy where possible** - Minimal string allocations
- **Native code** - No JavaScript runtime overhead

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

```bash
# Clone the repository
git clone https://github.com/vakra-dev/supermarkdown.git
cd supermarkdown

# Run tests
cargo test

# Build Node.js bindings
cd crates/supermarkdown-napi
npm install
npm run build
```

## License

MIT License - see [LICENSE](LICENSE) for details.
