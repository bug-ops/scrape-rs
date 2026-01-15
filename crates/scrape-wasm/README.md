# @scrape-rs/wasm

[![npm](https://img.shields.io/npm/v/@scrape-rs/wasm)](https://www.npmjs.com/package/@scrape-rs/wasm)
[![Bundle Size](https://img.shields.io/bundlephobia/minzip/@scrape-rs/wasm)](https://bundlephobia.com/package/@scrape-rs/wasm)
[![TypeScript](https://img.shields.io/badge/TypeScript-Ready-blue)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/npm/l/@scrape-rs/wasm)](../../LICENSE-MIT)

WebAssembly bindings for scrape-rs, a high-performance HTML parsing library. Run native-speed parsing in the browser.

## Installation

```bash
# npm
npm install @scrape-rs/wasm

# yarn
yarn add @scrape-rs/wasm

# pnpm
pnpm add @scrape-rs/wasm

# bun
bun add @scrape-rs/wasm
```

## Quick start

```typescript
import init, { Soup } from '@scrape-rs/wasm';

// Initialize WASM module (required once)
await init();

const html = "<html><body><div class='content'>Hello, World!</div></body></html>";
const soup = new Soup(html);

const div = soup.find("div");
console.log(div.text);
// Hello, World!
```

> [!IMPORTANT]
> Call `init()` once before using any other functions. It loads and compiles the WASM module.

## Usage

### Find elements

```typescript
import init, { Soup } from '@scrape-rs/wasm';

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

### With bundlers

**Vite:**

```typescript
import init, { Soup } from '@scrape-rs/wasm';

// Vite handles WASM automatically
await init();
```

**Webpack 5:**

```javascript
// webpack.config.js
module.exports = {
    experiments: {
        asyncWebAssembly: true,
    },
};
```

### CDN usage

```html
<script type="module">
import init, { Soup } from 'https://esm.sh/@scrape-rs/wasm';

await init();

const soup = new Soup('<div>Hello</div>');
console.log(soup.find('div').text);
</script>
```

## Bundle size

| Build | Size |
|-------|------|
| Minified + gzip | ~150 KB |
| Minified | ~400 KB |

> [!TIP]
> The WASM module includes SIMD optimizations. Modern browsers (Chrome 91+, Firefox 89+, Safari 16.4+) run SIMD automatically.

## TypeScript

Full TypeScript support with exported types:

```typescript
import init, { Soup, Tag } from '@scrape-rs/wasm';

await init();

function extractLinks(soup: Soup): string[] {
    return soup.select("a[href]").map(a => a.getAttribute("href") ?? "");
}
```

## Browser support

| Browser | Version | SIMD |
|---------|---------|------|
| Chrome | 80+ | 91+ |
| Firefox | 75+ | 89+ |
| Safari | 13+ | 16.4+ |
| Edge | 80+ | 91+ |

## Limitations

- No parallel processing (WASM threads have limited browser support)
- Must call `init()` before using the API
- Slightly higher memory usage than native bindings

## Related packages

Part of the [scrape-rs](https://github.com/bug-ops/scrape-rs) project:

- `scrape-core` — Rust core library
- `scrape-rs` (PyPI) — Python bindings
- `scrape-rs` (npm) — Node.js bindings

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
