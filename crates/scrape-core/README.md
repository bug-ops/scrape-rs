# scrape-core

[![Crates.io](https://img.shields.io/crates/v/scrape-core)](https://crates.io/crates/scrape-core)
[![docs.rs](https://img.shields.io/docsrs/scrape-core)](https://docs.rs/scrape-core)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue)](https://github.com/bug-ops/scrape-rs)
[![License](https://img.shields.io/crates/l/scrape-core)](../../LICENSE-MIT)

High-performance HTML parsing library core. Pure Rust implementation with no FFI dependencies.

## Installation

```toml
[dependencies]
scrape-core = "0.1"
```

Or with cargo:

```bash
cargo add scrape-core
```

> [!IMPORTANT]
> Requires Rust 1.88 or later.

## Usage

```rust
use scrape_core::Soup;

let html = r#"
    <html>
        <body>
            <div class="content">Hello, World!</div>
            <div class="content">Another div</div>
        </body>
    </html>
"#;

let soup = Soup::new(html);

// Find first element by tag
if let Some(div) = soup.find("div") {
    println!("Text: {}", div.text());
}

// CSS selectors
for el in soup.select("div.content") {
    println!("{}", el.inner_html());
}
```

## Features

Enable optional features in `Cargo.toml`:

```toml
[dependencies]
scrape-core = { version = "0.1", features = ["simd", "parallel"] }
```

| Feature | Description | Default |
|---------|-------------|---------|
| `simd` | SIMD-accelerated byte scanning (SSE4.2, AVX2, NEON, WASM SIMD128) | No |
| `parallel` | Parallel batch processing via Rayon | No |

> [!TIP]
> Start with default features for fastest compile times. Add `simd` for production workloads.

## Performance

Optimized for high throughput:

- Arena-based DOM allocation (cache-friendly, zero per-node heap allocations)
- SIMD-accelerated byte scanning when `simd` feature is enabled
- Parallel batch processing via Rayon when `parallel` feature is enabled

Benchmarks show **10x faster parsing** and **up to 132x faster queries** compared to BeautifulSoup. See full benchmark results in the [main project README](https://github.com/bug-ops/scrape-rs#performance).

## Architecture

```
scrape-core/
├── dom/       # Arena-based DOM representation
├── parser/    # html5ever integration
├── query/     # CSS selector engine
├── simd/      # Platform-specific SIMD acceleration
└── parallel/  # Rayon-based parallelization
```

### Built on Servo

The parsing and selector engine is powered by battle-tested libraries from the [Servo](https://servo.org/) browser engine:

- [html5ever](https://crates.io/crates/html5ever) — Spec-compliant HTML5 parser
- [selectors](https://crates.io/crates/selectors) — CSS selector matching engine
- [cssparser](https://crates.io/crates/cssparser) — CSS parser
- [markup5ever](https://crates.io/crates/markup5ever) — Common HTML/XML tree data structures

## MSRV policy

Minimum Supported Rust Version: **1.88**. MSRV increases are minor version bumps.

## Related packages

This crate is part of [fast-scrape](https://github.com/bug-ops/scrape-rs):

| Platform | Package |
|----------|---------|
| Python | [`fast-scrape`](https://pypi.org/project/fast-scrape) |
| Node.js | [`@fast-scrape/node`](https://www.npmjs.com/package/@fast-scrape/node) |
| WASM | [`@fast-scrape/wasm`](https://www.npmjs.com/package/@fast-scrape/wasm) |

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
