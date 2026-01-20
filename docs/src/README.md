# Introduction

Welcome to the scrape-rs documentation. scrape-rs is a high-performance, cross-platform HTML parsing library with a pure Rust core and bindings for Python, Node.js, and WASM.

## Why scrape-rs?

scrape-rs is designed to be **10-50x faster** than popular HTML parsing libraries while maintaining a consistent, idiomatic API across all platforms.

### Key Features

- **Blazing Fast**: Built on html5ever with SIMD-accelerated text processing
- **Cross-Platform**: Identical API for Rust, Python, Node.js, and WASM
- **Memory Efficient**: Arena-based DOM allocation with minimal overhead
- **Spec-Compliant**: Full HTML5 parsing with comprehensive CSS selector support
- **Modern**: Support for streaming parsing, compiled selectors, and parallel processing

### Performance Highlights

| Operation | BeautifulSoup | Cheerio | scrape-rs | Speedup |
|-----------|---------------|---------|-----------|---------|
| Parse 1KB HTML | 0.23ms | 0.18ms | **0.024ms** | 9.7-7.5x |
| Parse 100KB HTML | 18ms | 12ms | **1.8ms** | 10-6.7x |
| CSS selector query | 0.80ms | 0.12ms | **0.006ms** | 133-20x |
| Extract all links | 3.2ms | 0.85ms | **0.18ms** | 17.8-4.7x |

## Quick Example

### Rust

```rust
use scrape_core::Soup;

let html = r#"<div class="product"><h2>Widget</h2><span class="price">$19.99</span></div>"#;
let soup = Soup::parse(html);

let product = soup.find(".product")?.expect("product not found");
let name = product.find("h2")?.expect("name not found").text();
let price = product.find(".price")?.expect("price not found").text();

println!("{}: {}", name, price);
```

### Python

```python
from scrape_rs import Soup

html = '<div class="product"><h2>Widget</h2><span class="price">$19.99</span></div>'
soup = Soup(html)

product = soup.find(".product")
name = product.find("h2").text
price = product.find(".price").text

print(f"{name}: {price}")
```

### Node.js

```typescript
import { Soup } from '@scrape-rs/scrape';

const html = '<div class="product"><h2>Widget</h2><span class="price">$19.99</span></div>';
const soup = new Soup(html);

const product = soup.find(".product");
const name = product.find("h2").text;
const price = product.find(".price").text;

console.log(`${name}: ${price}`);
```

### WASM

```typescript
import init, { Soup } from '@scrape-rs/wasm';

await init();

const html = '<div class="product"><h2>Widget</h2><span class="price">$19.99</span></div>';
const soup = new Soup(html);

const product = soup.find(".product");
const name = product.find("h2").text;
const price = product.find(".price").text;

console.log(`${name}: ${price}`);
```

## Where to Go Next

- **New to scrape-rs?** Start with the [Quick Start](getting-started/quick-start.md) guide
- **Migrating from another library?** Check out the [Migration Guides](migration/overview.md)
- **API Reference?** See [Rust docs on docs.rs](https://docs.rs/scrape-core)

## Platform Support

| Platform | Status | Package |
|----------|--------|---------|
| Rust | Stable | [`scrape-core`](https://crates.io/crates/scrape-core) |
| Python 3.10+ | Stable | [`fast-scrape`](https://pypi.org/project/fast-scrape) |
| Node.js 18+ | Stable | [`@scrape-rs/scrape`](https://npmjs.com/package/@scrape-rs/scrape) |
| WASM | Stable | [`@scrape-rs/wasm`](https://npmjs.com/package/@scrape-rs/wasm) |

## License

scrape-rs is dual-licensed under Apache 2.0 and MIT. See [LICENSE-APACHE](https://github.com/bug-ops/scrape-rs/blob/main/LICENSE-APACHE) and [LICENSE-MIT](https://github.com/bug-ops/scrape-rs/blob/main/LICENSE-MIT) for details.
