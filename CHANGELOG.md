# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-01-16

### Added

- Core HTML parsing engine based on html5ever for spec-compliant parsing
- Arena-based DOM tree for memory efficiency and cache-friendly traversal
- CSS selector support via `selectors` crate with full CSS3 selector syntax
- SIMD-accelerated byte scanning with memchr (SSE4.2/AVX2 on x86_64, NEON on ARM64)
- Python bindings via PyO3 (PyPI: `scrape-rs`)
- Node.js bindings via napi-rs (npm: `scrape-rs`)
- WASM bindings for browsers (npm: `@scrape-rs/wasm`)
- Cross-platform API consistency across all bindings
- BeautifulSoup-compatible API with `Soup` and `Tag` types
- `find()` and `find_all()` methods for element search
- Attribute access and text extraction utilities
- Parallel processing support via Rayon (native platforms only)
- Comprehensive test suite with shared test cases across bindings
- Fuzzing targets for parser and selector components
- Benchmarks comparing against BeautifulSoup and Cheerio

### Performance

- 10-50x faster than BeautifulSoup for parsing and queries
- Zero-copy DOM navigation
- WASM bundle under 500KB gzipped

[Unreleased]: https://github.com/bug-ops/scrape-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/bug-ops/scrape-rs/releases/tag/v0.1.0
