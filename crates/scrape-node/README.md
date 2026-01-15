# scrape-rs (Node.js)

[![npm](https://img.shields.io/npm/v/scrape-rs)](https://www.npmjs.com/package/scrape-rs)
[![Node.js](https://img.shields.io/node/v/scrape-rs)](https://www.npmjs.com/package/scrape-rs)
[![TypeScript](https://img.shields.io/badge/TypeScript-Ready-blue)](https://www.typescriptlang.org/)
[![codecov](https://codecov.io/gh/bug-ops/scrape-rs/graph/badge.svg?token=6MQTONGT95&flag=node)](https://codecov.io/gh/bug-ops/scrape-rs)
[![License](https://img.shields.io/npm/l/scrape-rs)](../../LICENSE-MIT)

Node.js bindings for scrape-rs, a high-performance HTML parsing library.

## Installation

```bash
# npm
npm install scrape-rs

# yarn
yarn add scrape-rs

# pnpm
pnpm add scrape-rs

# bun
bun add scrape-rs
```

> [!NOTE]
> This package includes TypeScript definitions. No need for separate `@types` package.

## Quick start

```typescript
import { Soup } from 'scrape-rs';

const html = "<html><body><div class='content'>Hello, World!</div></body></html>";
const soup = new Soup(html);

const div = soup.find("div");
console.log(div.text);
// Hello, World!
```

## Usage

### Find elements

```typescript
import { Soup } from 'scrape-rs';

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

### Element properties

```typescript
const element = soup.find("a");

// Get text content
const text = element.text;

// Get inner HTML
const html = element.innerHTML;

// Get attribute
const href = element.getAttribute("href");
```

### Batch processing

```typescript
import { Soup } from 'scrape-rs';

// Process multiple documents in parallel
const documents = [html1, html2, html3];
const soups = Soup.parseBatch(documents);

for (const soup of soups) {
    console.log(soup.find("title")?.text);
}
```

> [!TIP]
> Use `parseBatch()` for processing multiple documents. It uses all CPU cores via native threads.

## TypeScript

Full TypeScript support with exported types:

```typescript
import { Soup, Tag, type SoupOptions } from 'scrape-rs';

function extractLinks(soup: Soup): string[] {
    return soup.select("a[href]").map(a => a.getAttribute("href") ?? "");
}
```

## Requirements

- Node.js >= 18.0.0
- Supported platforms: macOS (arm64, x64), Linux (x64, arm64), Windows (x64)

## Related packages

Part of the [scrape-rs](https://github.com/bug-ops/scrape-rs) project:

- `scrape-core` — Rust core library
- `scrape-rs` (PyPI) — Python bindings
- `@scrape-rs/wasm` — Browser/WASM bindings

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
