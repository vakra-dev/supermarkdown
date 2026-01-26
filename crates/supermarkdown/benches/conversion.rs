//! Benchmarks for HTML to Markdown conversion.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use supermarkdown::{convert, convert_with_options, Options};

/// Simple document with basic formatting.
const SIMPLE_HTML: &str = r#"
<article>
    <h1>Hello World</h1>
    <p>This is a <strong>simple</strong> paragraph with <em>emphasis</em>.</p>
    <p>Another paragraph with a <a href="https://example.com">link</a>.</p>
</article>
"#;

/// Medium-sized document with various elements.
const MEDIUM_HTML: &str = r#"
<article>
    <h1>Article Title</h1>
    <p>First paragraph with <strong>bold</strong> and <em>italic</em> text.</p>

    <h2>Section One</h2>
    <p>Some text with a <a href="https://example.com">link</a> and <code>inline code</code>.</p>
    <ul>
        <li>First item</li>
        <li>Second item with <strong>bold</strong></li>
        <li>Third item</li>
    </ul>

    <h2>Section Two</h2>
    <blockquote>
        <p>A quote with multiple sentences. This is the second sentence.</p>
    </blockquote>
    <pre><code class="language-rust">
fn main() {
    println!("Hello, world!");
}
    </code></pre>

    <h2>Section Three</h2>
    <table>
        <thead>
            <tr><th>Name</th><th>Age</th><th>City</th></tr>
        </thead>
        <tbody>
            <tr><td>Alice</td><td>30</td><td>New York</td></tr>
            <tr><td>Bob</td><td>25</td><td>Los Angeles</td></tr>
        </tbody>
    </table>

    <p>Final paragraph with an <img src="image.png" alt="example image">.</p>
</article>
"#;

/// Complex document with nested structures.
const COMPLEX_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head><title>Complex Document</title></head>
<body>
<nav><a href="/">Home</a> | <a href="/about">About</a></nav>
<main>
    <article>
        <header>
            <h1>Complex Article Title</h1>
            <p class="meta">Published on <time>2024-01-01</time></p>
        </header>

        <section>
            <h2>Introduction</h2>
            <p>This is a complex document with <strong>nested <em>formatting</em></strong> and various elements.</p>
            <p>It includes <a href="https://example.com" title="Example">links with titles</a> and <code>code</code>.</p>
        </section>

        <section>
            <h2>Lists</h2>
            <h3>Unordered List</h3>
            <ul>
                <li>Item one with <strong>bold</strong></li>
                <li>Item two with nested list:
                    <ul>
                        <li>Nested item A</li>
                        <li>Nested item B</li>
                    </ul>
                </li>
                <li>Item three</li>
            </ul>

            <h3>Ordered List</h3>
            <ol>
                <li>First step</li>
                <li>Second step with <a href="/step2">link</a></li>
                <li>Third step</li>
            </ol>
        </section>

        <section>
            <h2>Code Examples</h2>
            <pre><code class="language-python">
def hello():
    """Say hello."""
    print("Hello, World!")

if __name__ == "__main__":
    hello()
            </code></pre>

            <pre><code class="language-javascript">
function greet(name) {
    console.log(`Hello, ${name}!`);
}

greet("World");
            </code></pre>
        </section>

        <section>
            <h2>Tables</h2>
            <table>
                <thead>
                    <tr>
                        <th>Feature</th>
                        <th>Status</th>
                        <th>Notes</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>Headings</td>
                        <td>Complete</td>
                        <td>ATX and Setext styles</td>
                    </tr>
                    <tr>
                        <td>Lists</td>
                        <td>Complete</td>
                        <td>Ordered and unordered</td>
                    </tr>
                    <tr>
                        <td>Links</td>
                        <td>Complete</td>
                        <td>Inline and referenced</td>
                    </tr>
                </tbody>
            </table>
        </section>

        <section>
            <h2>Blockquotes</h2>
            <blockquote>
                <p>This is a blockquote with multiple paragraphs.</p>
                <p>Second paragraph in the quote with <em>emphasis</em>.</p>
                <blockquote>
                    <p>Nested blockquote for extra depth.</p>
                </blockquote>
            </blockquote>
        </section>

        <section>
            <h2>Images</h2>
            <figure>
                <img src="photo.jpg" alt="A beautiful photo">
                <figcaption>Caption for the image</figcaption>
            </figure>
        </section>

        <details>
            <summary>Click to expand</summary>
            <p>Hidden content inside a details element.</p>
        </details>

        <footer>
            <p>Article footer with <a href="/contact">contact</a> link.</p>
        </footer>
    </article>
</main>
<footer>
    <p>&copy; 2024 Example Inc. All rights reserved.</p>
</footer>
</body>
</html>
"#;

fn bench_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("conversion");

    // Simple HTML
    group.throughput(Throughput::Bytes(SIMPLE_HTML.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("simple", SIMPLE_HTML.len()),
        &SIMPLE_HTML,
        |b, html| {
            b.iter(|| convert(black_box(html)));
        },
    );

    // Medium HTML
    group.throughput(Throughput::Bytes(MEDIUM_HTML.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("medium", MEDIUM_HTML.len()),
        &MEDIUM_HTML,
        |b, html| {
            b.iter(|| convert(black_box(html)));
        },
    );

    // Complex HTML
    group.throughput(Throughput::Bytes(COMPLEX_HTML.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("complex", COMPLEX_HTML.len()),
        &COMPLEX_HTML,
        |b, html| {
            b.iter(|| convert(black_box(html)));
        },
    );

    group.finish();
}

fn bench_with_options(c: &mut Criterion) {
    let mut group = c.benchmark_group("conversion_with_options");

    // Test with different options
    let setext_options = Options::new().heading_style(supermarkdown::HeadingStyle::Setext);
    let referenced_options = Options::new().link_style(supermarkdown::LinkStyle::Referenced);
    let exclude_options = Options::new().exclude_selectors(vec!["nav".to_string(), "footer".to_string()]);

    group.throughput(Throughput::Bytes(COMPLEX_HTML.len() as u64));

    group.bench_function("default_options", |b| {
        b.iter(|| convert(black_box(COMPLEX_HTML)));
    });

    group.bench_function("setext_headings", |b| {
        b.iter(|| convert_with_options(black_box(COMPLEX_HTML), black_box(&setext_options)));
    });

    group.bench_function("referenced_links", |b| {
        b.iter(|| convert_with_options(black_box(COMPLEX_HTML), black_box(&referenced_options)));
    });

    group.bench_function("with_excludes", |b| {
        b.iter(|| convert_with_options(black_box(COMPLEX_HTML), black_box(&exclude_options)));
    });

    group.finish();
}

fn bench_repeated_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("repeated_conversion");

    // Create multiple documents
    let documents: Vec<&str> = vec![SIMPLE_HTML; 10];
    let total_bytes: usize = documents.iter().map(|d| d.len()).sum();

    group.throughput(Throughput::Bytes(total_bytes as u64));
    group.bench_function("10_simple_docs", |b| {
        b.iter(|| {
            for doc in &documents {
                let _ = convert(black_box(doc));
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_conversion,
    bench_with_options,
    bench_repeated_conversion
);
criterion_main!(benches);
