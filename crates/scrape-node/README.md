# @fast-scrape/node

[![npm](https://img.shields.io/npm/v/@fast-scrape/node)](https://www.npmjs.com/package/@fast-scrape/node)
[![Node.js](https://img.shields.io/node/v/@fast-scrape/node)](https://www.npmjs.com/package/@fast-scrape/node)
[![TypeScript](https://img.shields.io/badge/TypeScript-Ready-blue)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/npm/l/@fast-scrape/node)](../../LICENSE-MIT)

**2x faster** HTML parsing than Cheerio. Rust-powered with **300x faster** CSS selector queries.

## Installation

```bash
npm install @fast-scrape/node
```

<details>
<summary>Other package managers</summary>

```bash
yarn add @fast-scrape/node
pnpm add @fast-scrape/node
bun add @fast-scrape/node
```

</details>

> [!NOTE]
> Includes TypeScript definitions. No separate `@types` package needed.

## Quick start

```typescript
import { Soup } from '@fast-scrape/node';

const soup = new Soup("<html><body><div class='content'>Hello, World!</div></body></html>");

const div = soup.find("div");
console.log(div.text);  // Hello, World!
```

## Usage

<details open>
<summary><strong>Find elements</strong></summary>

```typescript
import { Soup } from '@fast-scrape/node';

const soup = new Soup(html);

// Find first element by tag
const div = soup.find("div");

// Find all elements
const divs = soup.findAll("div");

// CSS selectors
for (const el of soup.select("div.content > p")) {
    console.log(el.text);
}
```

</details>

<details>
<summary><strong>Element properties</strong></summary>

```typescript
const element = soup.find("a");

const text = element.text;                    // Get text content
const html = element.innerHTML;               // Get inner HTML
const href = element.getAttribute("href");    // Get attribute
```

</details>

<details>
<summary><strong>Batch processing</strong></summary>

```typescript
import { Soup } from '@fast-scrape/node';

// Process multiple documents in parallel
const documents = [html1, html2, html3];
const soups = Soup.parseBatch(documents);

for (const soup of soups) {
    console.log(soup.find("title")?.text);
}
```

> [!TIP]
> Use `parseBatch()` for multiple documents. Uses all CPU cores via native threads.

</details>

<details>
<summary><strong>TypeScript</strong></summary>

Full TypeScript support with exported types:

```typescript
import { Soup, Tag, type SoupOptions } from '@fast-scrape/node';

function extractLinks(soup: Soup): string[] {
    return soup.select("a[href]").map(a => a.getAttribute("href") ?? "");
}
```

</details>

## Requirements

- Node.js >= 18
- Platforms: macOS (arm64, x64), Linux (x64, arm64, musl), Windows (x64)

## Performance

Measured benchmarks comparing against Cheerio:

<details open>
<summary><strong>Parse speed comparison</strong></summary>

| File size | @fast-scrape/node | Cheerio | Speedup |
|-----------|-------------------|---------|---------|
| 1 KB | **0.030 ms** | 0.099 ms | **3.3x faster** |
| 218 KB | **3.85 ms** | 6.08 ms | **1.6x faster** |
| 5.9 MB | **124.14 ms** | 168.57 ms | **1.4x faster** |

**Average:** **2.1x faster than Cheerio**

> [!NOTE]
> Cheerio uses htmlparser2, which is already highly optimized JavaScript. The 2x parsing speedup is respectable given FFI overhead and Cheerio's maturity.

</details>

<details>
<summary><strong>Query performance (on 218 KB HTML)</strong></summary>

| Operation | @fast-scrape/node | Cheerio | Speedup |
|-----------|-------------------|---------|---------|
| `find("div")` | **0.001 ms** | 0.234 ms | **190x** |
| `find(".product-card")` | **<0.001 ms** | 0.327 ms | **733x** |
| `find("#product-100")` | **0.001 ms** | 0.277 ms | **503x** |
| `findAll("div")` | **0.226 ms** | 0.248 ms | **1.1x** |
| `select(".product-card")` | **0.143 ms** | 0.329 ms | **2.3x** |

**Average:** **286x faster than Cheerio**

**CSS selectors dominate:** Single-element queries are **hundreds of times faster** due to Rust's optimized selector engine.

</details>

**When to use @fast-scrape/node:**
- **Web scraping** — Many queries per document (massive speedup)
- **Data extraction** — CSS selector-heavy workloads
- **Batch processing** — `Soup.parseBatch()` uses all CPU cores

**When Cheerio might be sufficient:**
- **Simple parsing** — If you only parse and extract simple data
- **Established codebase** — Cheerio has huge ecosystem and community

**v0.2.0 optimizations:**
- **SIMD-accelerated CSS selectors** — 2-10x faster class/ID matching
- **Zero-copy serialization** — 50-70% memory reduction
- **Native threads** — Batch processing uses all CPU cores via napi-rs

See [complete benchmarks](https://github.com/bug-ops/scrape-rs#performance) comparing all platforms.

## Built on Servo and Cloudflare

**Parsing & Selection (Servo browser engine):**
- [html5ever](https://crates.io/crates/html5ever) — Spec-compliant HTML5 parser
- [selectors](https://crates.io/crates/selectors) — CSS selector matching engine

**Streaming Parser (Cloudflare):**
- [lol_html](https://github.com/cloudflare/lol_html) — High-performance streaming HTML parser with constant-memory event-driven API

## Related packages

| Platform | Package |
|----------|---------|
| Rust | [`scrape-core`](https://crates.io/crates/scrape-core) |
| Python | [`fast-scrape`](https://pypi.org/project/fast-scrape) |
| WASM | [`@fast-scrape/wasm`](https://www.npmjs.com/package/@fast-scrape/wasm) |

## License

MIT OR Apache-2.0
