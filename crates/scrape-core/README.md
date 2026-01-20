# scrape-core

[![Crates.io](https://img.shields.io/crates/v/scrape-core)](https://crates.io/crates/scrape-core)
[![docs.rs](https://img.shields.io/docsrs/scrape-core)](https://docs.rs/scrape-core)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue)](https://github.com/bug-ops/scrape-rs)
[![License](https://img.shields.io/crates/l/scrape-core)](../../LICENSE-MIT)

High-performance HTML parsing library core. Pure Rust implementation with no FFI dependencies.

## Installation

```toml
[dependencies]
scrape-core = "0.2"
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
scrape-core = { version = "0.2", features = ["simd", "parallel"] }
```

| Feature | Description | Default |
|---------|-------------|---------|
| `simd` | SIMD-accelerated byte scanning (SSE4.2, AVX2, NEON, WASM SIMD128) | No |
| `parallel` | Parallel batch processing via Rayon | No |

> [!TIP]
> Start with default features for fastest compile times. Add `simd` for production workloads.

## Performance

v0.2.0 includes massive performance improvements across all metrics:

| Metric | Result | vs Competitors |
|--------|--------|----------------|
| **Parse 1KB** | 11 µs | 20-38x faster |
| **Parse 100KB** | 2.96 ms | 9.5-22x faster |
| **Parse 1MB** | 15.5 ms | 66-135x faster |
| **Query (by class)** | 20 ns | 40,000x faster |
| **Memory (100MB doc)** | 145 MB | 14-22x smaller |

**Architecture optimizations:**
- **SIMD-accelerated class selector matching** — 2-10x faster on large documents
- **Selector fast-paths** — Direct optimization for tag-only, class-only, ID-only patterns
- **Arena-based DOM allocation** — Cache-friendly, zero per-node heap allocations
- **50-70% memory reduction** — Zero-copy serialization via Cow<str>
- **Parallel batch processing** — Rayon-powered when `parallel` feature is enabled

See full comparative benchmarks in the [main project README](https://github.com/bug-ops/scrape-rs#performance) comparing against BeautifulSoup4, lxml, Cheerio, and other Rust parsers.

## Type Safety

v0.2.0 introduces compile-time safety via the **typestate pattern**:

- **Document lifecycle states** — Building (construction) → Queryable (ready) → Sealed (immutable)
- **Sealed traits** — Prevent unintended implementations while allowing future extensions
- **Zero runtime overhead** — State encoding uses PhantomData with no allocation cost
- **Trait abstractions** — HtmlSerializer trait and ElementFilter iterators for consistent DOM access

All safety guarantees are verified at compile time with zero performance impact.

## Architecture

```
scrape-core/
├── dom/       # Arena-based DOM representation
├── parser/    # html5ever integration
├── query/     # CSS selector engine
├── simd/      # Platform-specific SIMD acceleration
└── parallel/  # Rayon-based parallelization
```

### Built on Servo and Cloudflare

**Parsing & Selection (Servo browser engine):**
- [html5ever](https://crates.io/crates/html5ever) — Spec-compliant HTML5 parser
- [selectors](https://crates.io/crates/selectors) — CSS selector matching engine
- [cssparser](https://crates.io/crates/cssparser) — CSS parser
- [markup5ever](https://crates.io/crates/markup5ever) — Common HTML/XML tree data structures

**Streaming Parser (Cloudflare):**
- [lol_html](https://github.com/cloudflare/lol_html) — High-performance streaming HTML parser with constant-memory event-driven API

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
