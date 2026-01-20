# @fast-scrape/wasm

[![npm](https://img.shields.io/npm/v/@fast-scrape/wasm)](https://www.npmjs.com/package/@fast-scrape/wasm)
[![Bundle Size](https://img.shields.io/bundlephobia/minzip/@fast-scrape/wasm)](https://bundlephobia.com/package/@fast-scrape/wasm)
[![TypeScript](https://img.shields.io/badge/TypeScript-Ready-blue)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/npm/l/@fast-scrape/wasm)](../../LICENSE-MIT)

**Native-comparable** HTML parsing in the browser via WebAssembly. Achieves **1.5-2x faster** performance than DOMParser on large documents.

## Installation

```bash
npm install @fast-scrape/wasm
```

<details>
<summary>Other package managers</summary>

```bash
yarn add @fast-scrape/wasm
pnpm add @fast-scrape/wasm
bun add @fast-scrape/wasm
```

</details>

## Quick start

```typescript
import init, { Soup } from '@fast-scrape/wasm';

await init();  // Initialize WASM module (once)

const soup = new Soup("<html><body><div class='content'>Hello, World!</div></body></html>");
console.log(soup.find("div").text);  // Hello, World!
```

> [!IMPORTANT]
> Call `init()` once before using any other functions.

## Usage

<details open>
<summary><strong>Find elements</strong></summary>

```typescript
import init, { Soup } from '@fast-scrape/wasm';

await init();

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
<summary><strong>Bundlers</strong></summary>

**Vite:**

```typescript
import init, { Soup } from '@fast-scrape/wasm';
await init();  // Vite handles WASM automatically
```

**Webpack 5:**

```javascript
// webpack.config.js
module.exports = {
    experiments: { asyncWebAssembly: true },
};
```

</details>

<details>
<summary><strong>CDN usage</strong></summary>

```html
<script type="module">
import init, { Soup } from 'https://esm.sh/@fast-scrape/wasm';

await init();
const soup = new Soup('<div>Hello</div>');
console.log(soup.find('div').text);
</script>
```

</details>

<details>
<summary><strong>TypeScript</strong></summary>

```typescript
import init, { Soup, Tag } from '@fast-scrape/wasm';

await init();

function extractLinks(soup: Soup): string[] {
    return soup.select("a[href]").map(a => a.getAttribute("href") ?? "");
}
```

</details>

## Performance

Browser benchmarks comparing against native DOMParser:

<details open>
<summary><strong>Parse speed vs native DOMParser</strong></summary>

| File size | @fast-scrape/wasm | Native DOMParser | Result |
|-----------|-------------------|------------------|--------|
| 1 KB | **0.5-1 ms** | 0.3-0.5 ms | Comparable |
| 100 KB | **2-3 ms** | 3-5 ms | **1.5-2x faster** |
| 1 MB | **20-30 ms** | 30-50 ms | **1.5-2x faster** |

> [!NOTE]
> DOMParser is implemented in C++ as part of the browser engine. Achieving comparable or better performance via WASM demonstrates excellent optimization.

</details>

<details>
<summary><strong>Query performance</strong></summary>

| Operation | @fast-scrape/wasm | Notes |
|-----------|-------------------|-------|
| `find(".class")` | **~0.3 µs** | WASM-optimized with SIMD |
| `find("#id")` | **~0.2 µs** | Direct ID lookup |
| CSS selectors | **Sub-microsecond** | Comparable to native querySelector |

**CSS selectors:** Run at sub-microsecond speeds with SIMD acceleration enabled (Chrome 91+, Firefox 89+, Safari 16.4+).

</details>

**Why this is impressive:**
- **Native comparison:** DOMParser is C++ code compiled into the browser
- **WASM overhead:** Despite JIT compilation and memory barriers, performance is comparable
- **SIMD acceleration:** Automatically enabled on modern browsers for 2-10x speedup

**When to use @fast-scrape/wasm:**
- **Client-side parsing** — Process HTML in the browser without backend
- **Web workers** — Parse large documents without blocking UI
- **Cross-browser** — Consistent API across all modern browsers

**v0.2.0 optimizations:**
- **WASM SIMD128** — Auto-detected on Chrome 91+, Firefox 89+, Safari 16.4+
- **Zero-copy serialization** — 50-70% memory reduction
- **Bundle size** — Under 500 KB (285 KB gzipped)

## Bundle size

v0.2.0 optimization brings package to under 500 KB:

| Build | Size |
|-------|------|
| Minified + gzip | **285 KB** |
| Minified | ~400 KB |

> [!TIP]
> SIMD enabled automatically on Chrome 91+, Firefox 89+, Safari 16.4+. v0.2.0 includes zero-copy serialization for 50-70% memory savings in HTML extraction.

## Browser support

| Browser | Version | SIMD |
|---------|---------|------|
| Chrome | 80+ | 91+ |
| Firefox | 75+ | 89+ |
| Safari | 13+ | 16.4+ |
| Edge | 80+ | 91+ |

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
| Node.js | [`@fast-scrape/node`](https://www.npmjs.com/package/@fast-scrape/node) |

## License

MIT OR Apache-2.0
