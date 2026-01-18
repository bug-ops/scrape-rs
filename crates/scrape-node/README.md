# @fast-scrape/node

[![npm](https://img.shields.io/npm/v/@fast-scrape/node)](https://www.npmjs.com/package/@fast-scrape/node)
[![Node.js](https://img.shields.io/node/v/@fast-scrape/node)](https://www.npmjs.com/package/@fast-scrape/node)
[![TypeScript](https://img.shields.io/badge/TypeScript-Ready-blue)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/npm/l/@fast-scrape/node)](../../LICENSE-MIT)

**10-50x faster** HTML parsing for Node.js. Rust-powered, Cheerio-compatible API.

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

## Built on Servo

Powered by battle-tested libraries from the [Servo](https://servo.org/) browser engine: [html5ever](https://crates.io/crates/html5ever) (HTML5 parser) and [selectors](https://crates.io/crates/selectors) (CSS selector engine).

## Related packages

| Platform | Package |
|----------|---------|
| Rust | [`scrape-core`](https://crates.io/crates/scrape-core) |
| Python | [`fast-scrape`](https://pypi.org/project/fast-scrape) |
| WASM | [`@fast-scrape/wasm`](https://www.npmjs.com/package/@fast-scrape/wasm) |

## License

MIT OR Apache-2.0
