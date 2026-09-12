const https = require('https');
const http = require('http');
const fs = require('fs');
const path = require('path');

const FIXTURES_DIR = path.join(__dirname, 'fixtures');

const FIXTURES = [
  // --- Small (<50KB) ---
  {
    name: 'small-commonmark.html',
    url: 'https://commonmark.org/help/',
    description: 'CommonMark help page',
  },
  {
    name: 'small-sqlite-lang.html',
    url: 'https://www.sqlite.org/lang_select.html',
    description: 'SQLite SELECT reference',
  },
  {
    name: 'small-semver.html',
    url: 'https://semver.org/',
    description: 'Semantic Versioning spec',
  },
  {
    name: 'small-caniuse.html',
    url: 'https://caniuse.com/css-grid',
    description: 'Can I Use CSS Grid',
  },
  {
    name: 'small-keepachangelog.html',
    url: 'https://keepachangelog.com/en/1.1.0/',
    description: 'Keep a Changelog spec',
  },

  // --- Medium (50-200KB) ---
  {
    name: 'medium-python-json.html',
    url: 'https://docs.python.org/3/library/json.html',
    description: 'Python json module docs',
  },
  {
    name: 'medium-express-routing.html',
    url: 'https://expressjs.com/en/guide/routing.html',
    description: 'Express.js routing guide',
  },
  {
    name: 'medium-python-re.html',
    url: 'https://docs.python.org/3/library/re.html',
    description: 'Python regex docs',
  },
  {
    name: 'medium-python-functions.html',
    url: 'https://docs.python.org/3/library/functions.html',
    description: 'Python built-in functions',
  },
  {
    name: 'medium-mdn-fetch.html',
    url: 'https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch',
    description: 'MDN Fetch API guide',
  },

  // --- Large (200-500KB) ---
  {
    name: 'large-mdn-array.html',
    url: 'https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array',
    description: 'MDN Array reference',
  },
  {
    name: 'large-express-full.html',
    url: 'https://expressjs.com/en/api.html',
    description: 'Express.js full API reference',
  },
  {
    name: 'large-wiki-markdown.html',
    url: 'https://en.wikipedia.org/wiki/Markdown',
    description: 'Wikipedia Markdown article',
  },
  {
    name: 'large-go-spec.html',
    url: 'https://go.dev/ref/spec',
    description: 'Go language specification',
  },
  {
    name: 'large-mdn-css.html',
    url: 'https://developer.mozilla.org/en-US/docs/Web/CSS/Reference',
    description: 'MDN CSS reference',
  },
  {
    name: 'large-python-stdtypes.html',
    url: 'https://docs.python.org/3/library/stdtypes.html',
    description: 'Python standard types docs',
  },
  {
    name: 'large-wiki-python.html',
    url: 'https://en.wikipedia.org/wiki/Python_(programming_language)',
    description: 'Wikipedia Python article',
  },
  {
    name: 'large-mdn-html-elements.html',
    url: 'https://developer.mozilla.org/en-US/docs/Web/HTML/Element',
    description: 'MDN HTML elements reference',
  },

  // --- Very large (500KB+) ---
  {
    name: 'xlarge-node-fs.html',
    url: 'https://nodejs.org/docs/latest/api/fs.html',
    description: 'Node.js fs module docs',
  },
  {
    name: 'xlarge-wiki-rust.html',
    url: 'https://en.wikipedia.org/wiki/Rust_(programming_language)',
    description: 'Wikipedia Rust article',
  },
  {
    name: 'xlarge-wiki-javascript.html',
    url: 'https://en.wikipedia.org/wiki/JavaScript',
    description: 'Wikipedia JavaScript article',
  },
  {
    name: 'xlarge-python-logging.html',
    url: 'https://docs.python.org/3/library/logging.html',
    description: 'Python logging module docs',
  },
  {
    name: 'xlarge-mdn-js-reference.html',
    url: 'https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference',
    description: 'MDN JavaScript reference index',
  },
  {
    name: 'xlarge-wiki-linux.html',
    url: 'https://en.wikipedia.org/wiki/Linux',
    description: 'Wikipedia Linux article',
  },
  {
    name: 'xlarge-node-http.html',
    url: 'https://nodejs.org/docs/latest/api/http.html',
    description: 'Node.js http module docs',
  },
];

function fetch(url) {
  return new Promise((resolve, reject) => {
    const client = url.startsWith('https') ? https : http;
    client.get(url, { headers: { 'User-Agent': 'supermarkdown-bench/1.0' } }, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        return fetch(res.headers.location).then(resolve, reject);
      }
      if (res.statusCode !== 200) {
        return reject(new Error(`HTTP ${res.statusCode} for ${url}`));
      }
      const chunks = [];
      res.on('data', (c) => chunks.push(c));
      res.on('end', () => resolve(Buffer.concat(chunks).toString('utf-8')));
      res.on('error', reject);
    }).on('error', reject);
  });
}

async function main() {
  if (!fs.existsSync(FIXTURES_DIR)) {
    fs.mkdirSync(FIXTURES_DIR, { recursive: true });
  }

  console.log(`Downloading ${FIXTURES.length} fixtures...\n`);

  let downloaded = 0;
  let skipped = 0;

  for (const fixture of FIXTURES) {
    const filepath = path.join(FIXTURES_DIR, fixture.name);

    if (fs.existsSync(filepath)) {
      const size = fs.statSync(filepath).size;
      console.log(`  skip  ${fixture.name} (${(size / 1024).toFixed(1)} KB, already exists)`);
      skipped++;
      continue;
    }

    try {
      const html = await fetch(fixture.url);
      fs.writeFileSync(filepath, html);
      const size = Buffer.byteLength(html);
      console.log(`  done  ${fixture.name} (${(size / 1024).toFixed(1)} KB)`);
      downloaded++;
    } catch (err) {
      console.error(`  FAIL  ${fixture.name}: ${err.message}`);
    }
  }

  console.log(`\n${downloaded} downloaded, ${skipped} skipped`);

  const files = fs.readdirSync(FIXTURES_DIR).filter((f) => f.endsWith('.html'));
  const totalSize = files.reduce((sum, f) => sum + fs.statSync(path.join(FIXTURES_DIR, f)).size, 0);
  console.log(`Total: ${files.length} fixtures, ${(totalSize / 1024).toFixed(1)} KB`);
}

main().catch(console.error);
