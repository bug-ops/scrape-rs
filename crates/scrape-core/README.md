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

## Architecture

```
scrape-core/
├── dom/       # Arena-based DOM representation
├── parser/    # html5ever integration
├── query/     # CSS selector engine
├── simd/      # Platform-specific SIMD acceleration
└── parallel/  # Rayon-based parallelization
```

## MSRV policy

Minimum Supported Rust Version: **1.88**. MSRV increases are minor version bumps.

## Related crates

This crate is part of the [scrape-rs](https://github.com/bug-ops/scrape-rs) workspace:

- `scrape-py` — Python bindings
- `scrape-node` — Node.js bindings
- `scrape-wasm` — WebAssembly bindings

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
