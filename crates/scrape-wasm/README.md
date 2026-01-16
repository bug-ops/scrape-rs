# @fast-scrape/wasm

[![npm](https://img.shields.io/npm/v/@fast-scrape/wasm)](https://www.npmjs.com/package/@fast-scrape/wasm)
[![Bundle Size](https://img.shields.io/bundlephobia/minzip/@fast-scrape/wasm)](https://bundlephobia.com/package/@fast-scrape/wasm)
[![TypeScript](https://img.shields.io/badge/TypeScript-Ready-blue)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/npm/l/@fast-scrape/wasm)](../../LICENSE-MIT)

**10-50x faster** HTML parsing in the browser. Native-speed parsing via WebAssembly.

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

## Bundle size

| Build | Size |
|-------|------|
| Minified + gzip | ~150 KB |
| Minified | ~400 KB |

> [!TIP]
> SIMD enabled automatically on Chrome 91+, Firefox 89+, Safari 16.4+.

## Browser support

| Browser | Version | SIMD |
|---------|---------|------|
| Chrome | 80+ | 91+ |
| Firefox | 75+ | 89+ |
| Safari | 13+ | 16.4+ |
| Edge | 80+ | 91+ |

## Related packages

| Platform | Package |
|----------|---------|
| Rust | [`scrape-core`](https://crates.io/crates/scrape-core) |
| Python | [`fast-scrape`](https://pypi.org/project/fast-scrape) |
| Node.js | [`@fast-scrape/node`](https://www.npmjs.com/package/@fast-scrape/node) |

## License

MIT OR Apache-2.0
