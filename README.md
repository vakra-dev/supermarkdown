# supermarkdown

[![npm version](https://img.shields.io/npm/v/@vakra-dev/supermarkdown.svg)](https://www.npmjs.com/package/@vakra-dev/supermarkdown)
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

### Node.js

```bash
npm install @vakra-dev/supermarkdown
```

### Rust

```bash
cargo add supermarkdown
```

### CLI

Install the CLI binary via cargo:

```bash
cargo install supermarkdown
```

## Command Line Usage

The CLI allows you to convert HTML files from the command line or via stdin:

```bash
# Convert a file
supermarkdown page.html > page.md

# Pipe HTML from curl
curl -s https://example.com | supermarkdown

# Exclude navigation and ads
supermarkdown --exclude "nav,.ad,#sidebar" page.html

# Use setext-style headings and referenced links
supermarkdown --heading-style setext --link-style referenced page.html
```

### CLI Options

| Option | Description |
| ------ | ----------- |
| `-h, --help` | Print help message |
| `-v, --version` | Print version |
| `--heading-style <STYLE>` | `atx` (default) or `setext` |
| `--link-style <STYLE>` | `inline` (default) or `referenced` |
| `--code-fence <CHAR>` | `` ` `` (default) or `~` |
| `--bullet <CHAR>` | `-` (default), `*`, or `+` |
| `--exclude <SELECTORS>` | CSS selectors to exclude (comma-separated) |

## Quick Start

```javascript
import { convert } from "@vakra-dev/supermarkdown";

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

## Common Use Cases

### Cleaning Web Scrapes

When scraping websites, HTML often contains navigation, ads, and other non-content elements. Use selectors to extract only what you need:

```javascript
import { convert } from "@vakra-dev/supermarkdown";

// Raw HTML from a web scrape
const scrapedHtml = await fetchPage("https://example.com/article");

// Clean conversion - remove nav, ads, sidebars
const markdown = convert(scrapedHtml, {
  excludeSelectors: [
    "nav",
    "header",
    "footer",
    ".sidebar",
    ".advertisement",
    ".cookie-banner",
    ".social-share",
    ".comments",
    "script",
    "style",
  ],
});
```

### Preparing Content for LLMs

When feeding web content to LLMs, you want clean, focused text without HTML artifacts:

```javascript
import { convert } from "@vakra-dev/supermarkdown";

// Extract just the article content for RAG pipelines
const markdown = convert(html, {
  excludeSelectors: [
    "nav",
    "header",
    "footer",
    "aside",
    ".related-posts",
    ".author-bio",
  ],
  includeSelectors: ["article", ".post-content", "main"],
});

// Now feed to your LLM
const response = await llm.chat({
  messages: [
    {
      role: "user",
      content: `Summarize this article:\n\n${markdown}`,
    },
  ],
});
```

### Processing Blog Posts

Convert blog HTML while preserving code blocks and formatting:

```javascript
import { convert } from "@vakra-dev/supermarkdown";

const blogHtml = `
<article>
  <h1>Getting Started with Rust</h1>
  <p>Rust is a systems programming language focused on safety.</p>
  <pre><code class="language-rust">fn main() {
    println!("Hello, world!");
}</code></pre>
  <p>The <code>println!</code> macro prints to stdout.</p>
</article>
`;

const markdown = convert(blogHtml);
// Output:
// # Getting Started with Rust
//
// Rust is a systems programming language focused on safety.
//
// ```rust
// fn main() {
//     println!("Hello, world!");
// }
// ```
//
// The `println!` macro prints to stdout.
```

### Converting Documentation Pages

Handle tables, definition lists, and nested structures common in docs:

```javascript
import { convert } from "@vakra-dev/supermarkdown";

const docsHtml = `
<h2>API Reference</h2>
<table>
  <tr><th>Method</th><th>Description</th></tr>
  <tr><td><code>convert()</code></td><td>Sync conversion</td></tr>
  <tr><td><code>convertAsync()</code></td><td>Async conversion</td></tr>
</table>
<dl>
  <dt>headingStyle</dt>
  <dd>ATX (#) or Setext (underlines)</dd>
</dl>
`;

const markdown = convert(docsHtml);
// Output:
// ## API Reference
//
// | Method | Description |
// | --- | --- |
// | `convert()` | Sync conversion |
// | `convertAsync()` | Async conversion |
//
// headingStyle
// :   ATX (#) or Setext (underlines)
```

### Batch Processing

Process multiple documents efficiently with async conversion:

```javascript
import { convertAsync } from "@vakra-dev/supermarkdown";

const urls = [
  "https://example.com/page1",
  "https://example.com/page2",
  "https://example.com/page3",
];

// Fetch and convert in parallel
const markdownDocs = await Promise.all(
  urls.map(async (url) => {
    const html = await fetch(url).then((r) => r.text());
    return convertAsync(html, {
      excludeSelectors: ["nav", "footer"],
    });
  })
);
```

## Usage

### Basic Conversion

```javascript
import { convert } from "@vakra-dev/supermarkdown";

const markdown = convert("<h1>Title</h1><p>Paragraph</p>");
```

### With Options

```javascript
import { convert } from "@vakra-dev/supermarkdown";

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
import { convertAsync } from "@vakra-dev/supermarkdown";

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

## Edge Cases

supermarkdown handles many edge cases gracefully:

### Malformed HTML

Invalid or malformed HTML is parsed via html5ever, which applies browser-like error recovery:

```javascript
// Missing closing tags, nested issues - all handled
const html = "<p>Unclosed paragraph<div>Mixed<p>nesting</div>";
const markdown = convert(html); // Produces sensible output
```

### Deeply Nested Lists

Nested lists maintain proper indentation:

```javascript
const html = `
<ul>
  <li>Level 1
    <ul>
      <li>Level 2
        <ul>
          <li>Level 3</li>
        </ul>
      </li>
    </ul>
  </li>
</ul>`;
// Output:
// - Level 1
//   - Level 2
//     - Level 3
```

### Code Blocks with Backticks

When code contains backticks, the fence automatically uses more backticks:

```javascript
const html = "<pre><code>Use `backticks` for code</code></pre>";
// Output uses 4 backticks as fence:
// ````
// Use `backticks` for code
// ````
```

### Empty Elements

Empty paragraphs, divs, and spans are stripped to avoid blank lines:

```javascript
const html = "<p></p><p>Real content</p><p>   </p>";
const markdown = convert(html);
// Output: "Real content" (empty paragraphs removed)
```

### Special Characters in URLs

Spaces, parentheses, and other special characters in URLs are percent-encoded:

```javascript
const html = '<a href="https://example.com/file (1).pdf">Download</a>';
// Output: [Download](https://example.com/file%20%281%29.pdf)
```

### Tables Without Headers

Tables missing `<thead>` use the first row as header:

```javascript
const html = `
<table>
  <tr><td>A</td><td>B</td></tr>
  <tr><td>1</td><td>2</td></tr>
</table>`;
// Output:
// | A | B |
// | --- | --- |
// | 1 | 2 |
```

### Mixed Content in Lists

List items with mixed block/inline content are handled:

```javascript
const html = `
<ul>
  <li>Simple item</li>
  <li>
    <p>Paragraph in list</p>
    <pre><code>code block</code></pre>
  </li>
</ul>`;
// Outputs proper markdown with preserved formatting
```

## Troubleshooting

### Empty or Minimal Output

**Problem:** `convert()` returns empty string or very little content.

**Causes & Solutions:**

1. **Content is in excluded elements** - Check if your content is inside `nav`, `header`, etc. that might match default patterns
   ```javascript
   // Try without selectors first
   const markdown = convert(html);
   ```

2. **JavaScript-rendered content** - supermarkdown converts static HTML only. If the page uses client-side rendering, you need to render it first (e.g., with Puppeteer or Playwright)

3. **Content in iframes** - iframe content is not extracted. Fetch iframe src separately if needed

### Missing Code Block Language

**Problem:** Code blocks don't have language annotation.

**Solution:** supermarkdown looks for `language-*`, `lang-*`, or `highlight-*` class patterns. Ensure your HTML uses standard class naming:

```html
<!-- Detected -->
<pre><code class="language-python">...</code></pre>
<pre><code class="lang-js">...</code></pre>

<!-- Not detected -->
<pre><code class="python-code">...</code></pre>
```

### Tables Not Rendering Correctly

**Problem:** Tables appear as plain text or are malformed.

**Causes & Solutions:**

1. **Missing table structure** - Ensure proper `<table>`, `<tr>`, `<td>` structure
2. **Nested tables** - GFM doesn't support nested tables; inner tables are flattened
3. **colspan/rowspan** - These are not supported in GFM; content goes in first cell

### Links Missing or Broken

**Problem:** Links don't appear or have wrong URLs.

**Solutions:**

1. **Relative URLs** - Use `baseUrl` option to resolve relative links:
   ```javascript
   convert(html, { baseUrl: "https://example.com" });
   ```

2. **Links in excluded elements** - Navigation links are often in `<nav>` which may be excluded

### Performance Issues with Large Documents

**Problem:** Conversion is slow for very large HTML files.

**Solutions:**

1. **Use async** - `convertAsync()` won't block the event loop
2. **Pre-filter HTML** - Remove obvious non-content before conversion
3. **Stream processing** - For very large docs, consider splitting into sections

### Special Characters Appearing Wrong

**Problem:** Characters like `<`, `>`, `&` appear as entities.

**Solution:** This is usually correct behavior - these characters need escaping in markdown. If you're seeing `&amp;` where you expect `&`, the source HTML may have double-encoded entities.

## Rust Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
supermarkdown = "0.0.2"
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
