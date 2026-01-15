# scrape-rs

[![CI](https://img.shields.io/github/actions/workflow/status/bug-ops/scrape-rs/ci.yml?branch=main)](https://github.com/bug-ops/scrape-rs/actions)
[![codecov](https://codecov.io/gh/bug-ops/scrape-rs/graph/badge.svg?token=6MQTONGT95)](https://codecov.io/gh/bug-ops/scrape-rs)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-MIT)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue)](https://github.com/bug-ops/scrape-rs)

High-performance, cross-platform HTML parsing library. Single Rust core with bindings for Python, Node.js, and WebAssembly.

## Features

- **Fast** — Optimized for high-throughput HTML parsing
- **Cross-platform** — Native bindings for Rust, Python, Node.js, and browsers (WASM)
- **Consistent API** — Same interface across all platforms
- **Memory-safe** — Built in Rust with zero unsafe code in core
- **SIMD-accelerated** — Auto-detects and uses SIMD on supported platforms

## Packages

| Package | Description | Status |
|---------|-------------|--------|
| [scrape-core](crates/scrape-core) | Pure Rust core library | [![Crates.io](https://img.shields.io/crates/v/scrape-core)](https://crates.io/crates/scrape-core) |
| [scrape-py](crates/scrape-py) | Python bindings | [![PyPI](https://img.shields.io/pypi/v/scrape-rs)](https://pypi.org/project/scrape-rs) |
| [scrape-node](crates/scrape-node) | Node.js bindings | [![npm](https://img.shields.io/npm/v/scrape-rs)](https://www.npmjs.com/package/scrape-rs) |
| [scrape-wasm](crates/scrape-wasm) | WebAssembly bindings | [![npm](https://img.shields.io/npm/v/@scrape-rs/wasm)](https://www.npmjs.com/package/@scrape-rs/wasm) |

## Installation

### Rust

```toml
[dependencies]
scrape-core = "0.1"
```

> [!IMPORTANT]
> Requires Rust 1.88 or later.

### Python

```bash
pip install scrape-rs
```

### Node.js

```bash
npm install scrape-rs
```

### Browser (WASM)

```bash
npm install @scrape-rs/wasm
```

```javascript
import init, { Soup } from '@scrape-rs/wasm';
await init();
```

## Quick start

### Rust

```rust
use scrape_core::Soup;

let html = "<html><body><div class='content'>Hello</div></body></html>";
let soup = Soup::new(html);

let div = soup.find("div").unwrap();
println!("{}", div.text());

for el in soup.select("div.content") {
    println!("{}", el.inner_html());
}
```

### Python

```python
from scrape_rs import Soup

html = "<html><body><div class='content'>Hello</div></body></html>"
soup = Soup(html)

div = soup.find("div")
print(div.text)

for el in soup.select("div.content"):
    print(el.inner_html)
```

### Node.js

```javascript
import { Soup } from 'scrape-rs';

const html = "<html><body><div class='content'>Hello</div></body></html>";
const soup = new Soup(html);

const div = soup.find("div");
console.log(div.text);

for (const el of soup.select("div.content")) {
    console.log(el.innerHTML);
}
```

## Feature flags (Rust)

```toml
[dependencies]
scrape-core = { version = "0.1", features = ["simd", "parallel"] }
```

| Feature | Description | Default |
|---------|-------------|---------|
| `simd` | SIMD-accelerated parsing (SSE4.2, AVX2, NEON) | No |
| `parallel` | Parallel batch processing via Rayon | No |

> [!TIP]
> Python and Node.js bindings enable both features by default. WASM uses `simd` only (no threads).

## Performance

Designed for high performance with:

- Arena-based DOM allocation (cache-friendly, zero per-node heap allocations)
- SIMD-accelerated byte scanning (SSE4.2, AVX2, NEON, WASM SIMD128)
- Parallel batch processing via Rayon

> [!NOTE]
> Benchmarks coming soon. Run `cargo bench` to test on your hardware.

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

This project adheres to the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
