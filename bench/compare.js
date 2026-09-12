const fs = require('fs');
const path = require('path');
const os = require('os');
const { convert } = require('../crates/supermarkdown-napi');
const TurndownService = require('turndown');
const { NodeHtmlMarkdown } = require('node-html-markdown');

const FIXTURES_DIR = path.join(__dirname, 'fixtures');
const RESULTS_DIR = path.join(__dirname, 'results');
const ITERATIONS = 100;
const WARMUP = 10;

const gc = globalThis.gc || (() => {});

const CONVERTERS = [
  {
    name: 'supermarkdown',
    label: 'Supermarkdown (Rust/NAPI)',
    setup: () => convert,
    run: (fn, html) => fn(html),
  },
  {
    name: 'turndown',
    label: 'Turndown (JavaScript)',
    setup: () => new TurndownService(),
    run: (td, html) => td.turndown(html),
  },
  {
    name: 'node-html-markdown',
    label: 'node-html-markdown (JavaScript)',
    setup: () => new NodeHtmlMarkdown(),
    run: (nhm, html) => nhm.translate(html),
  },
];

function loadFixtures() {
  const files = fs.readdirSync(FIXTURES_DIR).filter((f) => f.endsWith('.html')).sort();
  if (files.length === 0) {
    console.error('No fixtures found. Run: npm run download');
    process.exit(1);
  }
  return files.map((name) => {
    const html = fs.readFileSync(path.join(FIXTURES_DIR, name), 'utf-8');
    return { name, html, bytes: Buffer.byteLength(html) };
  });
}

function bench(fn) {
  let sink = 0;

  for (let i = 0; i < WARMUP; i++) {
    const r = fn();
    sink += r.length;
  }

  const times = new Float64Array(ITERATIONS);
  for (let i = 0; i < ITERATIONS; i++) {
    const start = process.hrtime.bigint();
    const r = fn();
    const end = process.hrtime.bigint();
    sink += r.length;
    times[i] = Number(end - start) / 1e6;
  }

  // Prevent dead code elimination
  if (sink === -Infinity) console.log(sink);

  times.sort();
  const median = times[Math.floor(ITERATIONS / 2)];
  const mean = times.reduce((a, b) => a + b, 0) / ITERATIONS;
  const p95 = times[Math.floor(ITERATIONS * 0.95)];
  const p99 = times[Math.floor(ITERATIONS * 0.99)];
  const min = times[0];
  const max = times[ITERATIONS - 1];

  return { median, mean, p95, p99, min, max };
}

function formatMs(ms) {
  if (ms < 0.01) return `${(ms * 1000).toFixed(1)}us`;
  if (ms < 1) return `${ms.toFixed(3)}ms`;
  if (ms < 100) return `${ms.toFixed(2)}ms`;
  if (ms < 1000) return `${ms.toFixed(1)}ms`;
  return `${(ms / 1000).toFixed(2)}s`;
}

function formatSize(bytes) {
  if (bytes >= 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)}MB`;
  return `${(bytes / 1024).toFixed(1)}KB`;
}

function getCpuModel() {
  const cpus = os.cpus();
  return cpus.length > 0 ? cpus[0].model.trim() : 'unknown';
}

function verifyOutputs(fixtures) {
  console.log('Verifying converter outputs...\n');
  const instances = CONVERTERS.map((c) => ({ ...c, instance: c.setup() }));

  for (const fixture of fixtures) {
    const outputs = instances.map((c) => ({
      name: c.name,
      output: c.run(c.instance, fixture.html),
    }));

    for (const { name, output } of outputs) {
      if (!output || output.length === 0) {
        console.error(`  FAIL: ${name} produced empty output for ${fixture.name}`);
        process.exit(1);
      }
    }

    const sizes = outputs.map((o) => `${o.name}: ${formatSize(Buffer.byteLength(o.output))}`);
    console.log(`  ${fixture.name}: ${sizes.join(', ')}`);
  }
  console.log();
}

function main() {
  const fixtures = loadFixtures();
  const totalBytes = fixtures.reduce((sum, f) => sum + f.bytes, 0);
  const cpuModel = getCpuModel();
  const startTime = Date.now();

  console.log(`\nsupermarkdown benchmark`);
  console.log(`${'='.repeat(80)}`);
  console.log(`CPU: ${cpuModel}`);
  console.log(`Platform: ${process.platform} ${process.arch}, Node ${process.version}`);
  console.log(`GC exposed: ${globalThis.gc ? 'yes' : 'no (run with --expose-gc for lower variance)'}`);
  console.log(`Fixtures: ${fixtures.length} files, ${formatSize(totalBytes)} total`);
  console.log(`Iterations: ${ITERATIONS} measured, ${WARMUP} warmup`);
  console.log(`Converters: ${CONVERTERS.map((c) => c.name).join(', ')}`);
  console.log(`Instance reuse: yes (all converters reuse a single instance, matching production usage)\n`);

  verifyOutputs(fixtures);

  const instances = CONVERTERS.map((c) => ({ ...c, instance: c.setup() }));
  const results = [];

  for (const fixture of fixtures) {
    process.stdout.write(`  ${fixture.name} (${formatSize(fixture.bytes)})...`);
    const fixtureStart = Date.now();

    const timings = {};
    for (const converter of instances) {
      gc();
      timings[converter.name] = bench(
        () => converter.run(converter.instance, fixture.html)
      );
    }

    const smMedian = timings['supermarkdown'].median;
    const ratios = {};
    for (const converter of instances) {
      if (converter.name !== 'supermarkdown') {
        ratios[converter.name] = timings[converter.name].median / smMedian;
      }
    }

    results.push({ fixture: fixture.name, bytes: fixture.bytes, timings, ratios });

    const elapsed = ((Date.now() - fixtureStart) / 1000).toFixed(0);
    const parts = instances.map((c) => {
      const t = formatMs(timings[c.name].median);
      if (c.name === 'supermarkdown') return `SM ${t}`;
      return `${c.name} ${t} (${ratios[c.name].toFixed(1)}x)`;
    });
    console.log(` ${parts.join(', ')} [${elapsed}s]`);
  }

  // Results table
  const colW = { name: 24, size: 8, sm: 10, td: 10, nhm: 10, tdR: 8, nhmR: 8 };
  const totalW = colW.name + colW.size + colW.sm + colW.td + colW.nhm + colW.tdR + colW.nhmR;

  console.log(`\n${'='.repeat(totalW)}`);
  console.log(`RESULTS (median, ${ITERATIONS} iterations, ${WARMUP} warmup)\n`);

  console.log(
    'Fixture'.padEnd(colW.name) +
    'Size'.padStart(colW.size) +
    'SM'.padStart(colW.sm) +
    'Turndown'.padStart(colW.td) +
    'NHM'.padStart(colW.nhm) +
    'TD/SM'.padStart(colW.tdR) +
    'NHM/SM'.padStart(colW.nhmR)
  );
  console.log('-'.repeat(totalW));

  for (const r of results) {
    console.log(
      r.fixture.replace('.html', '').padEnd(colW.name) +
      formatSize(r.bytes).padStart(colW.size) +
      formatMs(r.timings['supermarkdown'].median).padStart(colW.sm) +
      formatMs(r.timings['turndown'].median).padStart(colW.td) +
      formatMs(r.timings['node-html-markdown'].median).padStart(colW.nhm) +
      `${r.ratios['turndown'].toFixed(1)}x`.padStart(colW.tdR) +
      `${r.ratios['node-html-markdown'].toFixed(1)}x`.padStart(colW.nhmR)
    );
  }

  // Aggregates
  const aggSm = results.reduce((s, r) => s + r.timings['supermarkdown'].median, 0);
  const aggTd = results.reduce((s, r) => s + r.timings['turndown'].median, 0);
  const aggNhm = results.reduce((s, r) => s + r.timings['node-html-markdown'].median, 0);

  console.log('-'.repeat(totalW));
  console.log(
    'TOTAL'.padEnd(colW.name) +
    formatSize(totalBytes).padStart(colW.size) +
    formatMs(aggSm).padStart(colW.sm) +
    formatMs(aggTd).padStart(colW.td) +
    formatMs(aggNhm).padStart(colW.nhm) +
    `${(aggTd / aggSm).toFixed(1)}x`.padStart(colW.tdR) +
    `${(aggNhm / aggSm).toFixed(1)}x`.padStart(colW.nhmR)
  );

  const smTput = (totalBytes / 1024 / 1024) / (aggSm / 1000);
  const tdTput = (totalBytes / 1024 / 1024) / (aggTd / 1000);
  const nhmTput = (totalBytes / 1024 / 1024) / (aggNhm / 1000);
  console.log(`\nThroughput: SM ${smTput.toFixed(1)} MB/s, Turndown ${tdTput.toFixed(1)} MB/s, NHM ${nhmTput.toFixed(1)} MB/s`);

  // Detailed stats per converter
  for (const converter of instances) {
    console.log(`\nDetailed stats (${converter.name}):`);
    for (const r of results) {
      const s = r.timings[converter.name];
      console.log(`  ${r.fixture}: median=${formatMs(s.median)} mean=${formatMs(s.mean)} p95=${formatMs(s.p95)} p99=${formatMs(s.p99)} min=${formatMs(s.min)}`);
    }
  }

  const totalElapsed = ((Date.now() - startTime) / 1000).toFixed(0);
  console.log(`\nTotal benchmark time: ${totalElapsed}s`);

  // Save results
  if (!fs.existsSync(RESULTS_DIR)) fs.mkdirSync(RESULTS_DIR, { recursive: true });
  const output = {
    timestamp: new Date().toISOString(),
    config: { iterations: ITERATIONS, warmup: WARMUP, instanceReuse: true },
    platform: {
      cpu: cpuModel,
      arch: process.arch,
      os: process.platform,
      node: process.version,
      gcExposed: !!globalThis.gc,
    },
    fixtures: results.map((r) => ({
      name: r.fixture,
      bytes: r.bytes,
      timings: r.timings,
      ratios: r.ratios,
    })),
    aggregate: {
      totalBytes,
      supermarkdown: { total: aggSm, throughput: smTput },
      turndown: { total: aggTd, throughput: tdTput, ratio: aggTd / aggSm },
      'node-html-markdown': { total: aggNhm, throughput: nhmTput, ratio: aggNhm / aggSm },
    },
  };
  const resultsPath = path.join(RESULTS_DIR, 'latest.json');
  fs.writeFileSync(resultsPath, JSON.stringify(output, null, 2));
  console.log(`Results saved to ${resultsPath}`);
}

main();
