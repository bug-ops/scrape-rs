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

Compared to Cheerio (the popular Node.js choice):

| Test | @fast-scrape/node | Cheerio | Speedup |
|------|-------------------|---------|---------|
| Parse 100KB | 0.28 ms | 64.8 ms | **228x** |
| find(".item") | 45 ns | 422 µs | **9,333x** |
| select 5 levels | 0.89 µs | 4.4 µs | **4,940x** |
| Memory (100MB) | 165 MB | 1,800 MB | **11x** |

**v0.2.0 highlights:**
- **SIMD-accelerated class matching** — 2-10x faster on documents with many class selectors
- **Zero-copy serialization** — 50-70% memory reduction in HTML output
- **Batch processing** — `Soup.parseBatch()` parallelizes across all CPU cores
- **Query speed dominance** — CSS selectors run in nanoseconds vs microseconds

See [complete benchmarks](https://github.com/bug-ops/scrape-rs#performance) comparing all platforms and competitors.

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
