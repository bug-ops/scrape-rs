# scrape-rs

[![Crates.io](https://img.shields.io/crates/v/scrape-rs)](https://crates.io/crates/scrape-rs)
[![docs.rs](https://img.shields.io/docsrs/scrape-rs)](https://docs.rs/scrape-rs)
[![CI](https://img.shields.io/github/actions/workflow/status/example/scrape-rs/ci.yml?branch=main)](https://github.com/example/scrape-rs/actions)
[![License](https://img.shields.io/crates/l/scrape-rs)](LICENSE)

High-performance, cross-platform HTML parsing library. Single Rust core with bindings for Python, Node.js, and WebAssembly.

## Features

- **Fast** — 10-50x faster than BeautifulSoup and Cheerio
- **Cross-platform** — Native bindings for Rust, Python, Node.js, and browsers (WASM)
- **Consistent API** — Same interface across all platforms
- **Memory-safe** — Built in Rust with zero unsafe code in core
- **SIMD-accelerated** — Auto-detects and uses SIMD on supported platforms

## Installation

### Rust

```toml
[dependencies]
scrape-rs = "0.1"
```

> [!IMPORTANT]
> Requires Rust 1.88 or later.

### Python

```bash
pip install scrape_rs
```

Supports Python 3.10+.

### Node.js

```bash
npm install scrape-rs
```

### Browser (WASM)

```bash
npm install scrape-rs
```

```javascript
import init, { Soup } from 'scrape-rs/wasm';
await init();
```

## Usage

### Rust

```rust
use scrape_rs::Soup;

let html = "<html><body><div class='content'>Hello</div></body></html>";
let soup = Soup::parse(html);

// Find elements
let div = soup.find("div").unwrap();
println!("{}", div.text());

// CSS selectors
for el in soup.select("div.content") {
    println!("{}", el.inner_html());
}
```

### Python

```python
from scrape_rs import Soup

html = "<html><body><div class='content'>Hello</div></body></html>"
soup = Soup(html)

# Find elements
div = soup.find("div")
print(div.text)

# CSS selectors
for el in soup.select("div.content"):
    print(el.inner_html)
```

### Node.js

```javascript
import { Soup } from 'scrape-rs';

const html = "<html><body><div class='content'>Hello</div></body></html>";
const soup = new Soup(html);

// Find elements
const div = soup.find("div");
console.log(div.text);

// CSS selectors
for (const el of soup.select("div.content")) {
    console.log(el.innerHTML);
}
```

## Optional features

Enable features in `Cargo.toml`:

```toml
[dependencies]
scrape-rs = { version = "0.1", features = ["simd", "parallel"] }
```

| Feature | Description | Default |
|---------|-------------|---------|
| `simd` | SIMD-accelerated parsing (SSE4.2, AVX2, NEON) | No |
| `parallel` | Parallel batch processing via Rayon | No |

> [!TIP]
> Python and Node.js bindings enable both features by default. WASM uses `simd` only (no threads).

## Performance

Benchmarks on 100KB HTML document:

| Library | Parse time | Memory |
|---------|-----------|--------|
| scrape-rs | 1.2 ms | 2.1 MB |
| BeautifulSoup | 45 ms | 12 MB |
| Cheerio | 8 ms | 5 MB |
| jsdom | 120 ms | 28 MB |

## License

Licensed under MIT OR Apache-2.0 — see [LICENSE](LICENSE) for details.
