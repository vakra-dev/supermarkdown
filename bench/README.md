# Benchmark

Comparison benchmark for supermarkdown (Rust/NAPI) vs JavaScript alternatives on real-world HTML documents. Methodology follows [h2m-parser](https://github.com/nichochar/h2m-parser)'s benchmark standard.

## Results

Tested on Apple M5, Node v26.0.0, 24 real web pages totaling 10.8MB.

| Fixture | Size | supermarkdown | Turndown | NHM | TD/SM | NHM/SM |
|---------|------|--------------|----------|-----|-------|--------|
| small-commonmark | 8.9KB | 0.149ms | 0.684ms | 0.354ms | 4.6x | 2.4x |
| small-keepachangelog | 25.7KB | 0.368ms | 1.13ms | 0.836ms | 3.1x | 2.3x |
| small-semver | 30.4KB | 0.447ms | 1.11ms | 0.888ms | 2.5x | 2.0x |
| small-caniuse | 33.8KB | 0.404ms | 1.87ms | 0.913ms | 4.6x | 2.3x |
| medium-python-json | 109KB | 2.02ms | 12.42ms | 9.34ms | 6.2x | 4.6x |
| medium-mdn-fetch | 180KB | 1.88ms | 6.51ms | 9.95ms | 3.5x | 5.3x |
| medium-express-routing | 227KB | 2.92ms | 21.62ms | 13.17ms | 7.4x | 4.5x |
| medium-python-re | 238KB | 4.56ms | 26.25ms | 47.99ms | 5.8x | 10.5x |
| medium-python-functions | 309KB | 5.79ms | 31.09ms | 81.99ms | 5.4x | 14.2x |
| large-express-full | 85KB | 0.868ms | 4.49ms | 2.72ms | 5.2x | 3.1x |
| large-mdn-array | 239KB | 2.74ms | 10.78ms | 28.82ms | 3.9x | 10.5x |
| large-wiki-markdown | 304KB | 3.88ms | 15.15ms | 34.20ms | 3.9x | 8.8x |
| large-go-spec | 334KB | 6.20ms | 50.98ms | 296.5ms | 8.2x | 47.8x |
| large-mdn-css | 461KB | 7.79ms | 31.09ms | 373.0ms | 4.0x | 47.9x |
| large-python-stdtypes | 754KB | 15.37ms | 79.94ms | 617.6ms | 5.2x | 40.2x |
| large-wiki-python | 985KB | 12.79ms | 47.01ms | 701.3ms | 3.7x | 54.8x |
| xlarge-python-logging | 187KB | 3.59ms | 18.49ms | 36.89ms | 5.2x | 10.3x |
| xlarge-mdn-js-reference | 224KB | 2.89ms | 10.19ms | 29.25ms | 3.5x | 10.1x |
| xlarge-node-http | 633KB | 10.47ms | 62.77ms | 1.48s | 6.0x | 141.0x |
| xlarge-wiki-javascript | 689KB | 10.92ms | 41.38ms | 311.1ms | 3.8x | 28.5x |
| xlarge-wiki-linux | 951KB | 14.55ms | 54.50ms | 510.2ms | 3.7x | 35.1x |
| xlarge-wiki-rust | 999KB | 14.27ms | 64.79ms | 485.7ms | 4.5x | 34.0x |
| xlarge-node-fs | 1.1MB | 19.62ms | 116.8ms | 28.53s | 6.0x | 1453.6x |
| small-sqlite-lang | 1.9MB | 14.94ms | 74.26ms | 27.19ms | 5.0x | 1.8x |
| **TOTAL** | **10.8MB** | **159ms** | **785ms** | **33.6s** | **4.9x** | **210.9x** |

### Throughput

| Converter | Throughput |
|-----------|-----------|
| supermarkdown | 67.8 MB/s |
| Turndown | 13.8 MB/s |
| node-html-markdown | 0.3 MB/s |

### Key findings

- **supermarkdown is 3-8x faster than Turndown** across all document sizes, with a 4.9x aggregate speedup.
- **node-html-markdown exhibits O(n^2) scaling** on large documents. On the 1.1MB Node.js fs docs, NHM takes 28.5 seconds per conversion (1453x slower). On small documents (<50KB), NHM is only 2-2.4x slower.
- supermarkdown maintains **linear O(n) scaling**: 0.15ms for 9KB, 15ms for 1MB.

## Quick Start

```bash
npm install
npm run download   # fetch HTML fixtures from public URLs
npm run compare    # run benchmark (takes ~75 minutes)
```

Or in one step:

```bash
npm run run
```

For lower variance, expose the garbage collector:

```bash
node --expose-gc bench/compare.js
```

## Methodology

- **Fixtures**: 24 real webpages (documentation, Wikipedia, spec pages) ranging from 9KB to 1.9MB, downloaded from public URLs
- **Warmup**: 10 iterations discarded before measurement
- **Iterations**: 100 measured iterations per fixture per converter
- **Timing**: `process.hrtime.bigint()` (nanosecond precision)
- **Dead code elimination prevention**: All return values accumulated into a sink variable
- **GC control**: `global.gc()` called between fixtures when available via `--expose-gc`
- **Instance reuse**: All converters create a single instance and reuse it, matching production usage patterns
- **Metrics**: median, mean, p95, p99 per fixture

Fixtures are downloaded on first run via `download-fixtures.js` and cached locally. They are gitignored to avoid bloating the repository.

## Tools Compared

| Tool | Language | npm package |
|------|----------|-------------|
| supermarkdown | Rust (NAPI) | `@vakra-dev/supermarkdown` |
| Turndown | JavaScript | `turndown` |
| node-html-markdown | JavaScript | `node-html-markdown` |

## Reproducing

Results will vary by machine. To reproduce:

1. Clone the repo and build the NAPI binary: `cd crates/supermarkdown-napi && npm run build`
2. `cd bench && npm install && npm run run`

Raw results are saved to `results/latest.json`.
