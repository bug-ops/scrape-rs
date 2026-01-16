# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.6] - 2026-01-16

### Fixed

- Use `--ignore-scripts` for npm main package publish (platform packages already published in loop)

## [0.1.5] - 2026-01-16

### Fixed

- Fixed `contents: write` permission for npm publish job (napi prepublish needs to create releases)

## [0.1.4] - 2026-01-16

### Fixed

- Fixed npm auth for platform packages publishing (explicit .npmrc setup)

## [0.1.3] - 2026-01-16

### Fixed

- Added `GITHUB_TOKEN` for napi prepublish in npm release workflow

## [0.1.2] - 2026-01-16

### Fixed

- Fixed doctests using private `dom` module path
- Fixed pnpm lockfile compatibility in CI (upgraded to pnpm 10)
- Fixed maturin cross-compilation with `--find-interpreter` flag
- Disabled `linux-arm64-musl` Node.js build temporarily (requires proper cross-compiler)

## [0.1.1] - 2026-01-16

### Added

- **CLI tool** (`scrape-cli`): Command-line HTML extraction tool
  - CSS selector-based extraction with `-s`/`--selector`
  - Multiple output formats: text, JSON, CSV, HTML (`-f`/`--format`)
  - Batch processing with parallel execution via Rayon
  - Named extractions support (`-n`/`--named`)
  - Attribute extraction (`-a`/`--attrs`)
  - HTML fragment output (`--html`)
  - File and stdin input support
- CLI binary builds for 5 platforms in GitHub releases (linux-x64, linux-arm64, darwin-x64, darwin-arm64, windows-x64)

### Changed

- Renamed Python package from `scrape-rs` to `fast-scrape` on PyPI
- Renamed Node.js package from `scrape-rs` to `@fast-scrape/node` on npm
- Renamed WASM package from `@scrape-rs/wasm` to `@fast-scrape/wasm` on npm

### Fixed

- HTML comment escaping in CLI output to prevent injection via filenames

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

[Unreleased]: https://github.com/bug-ops/scrape-rs/compare/v0.1.6...HEAD
[0.1.6]: https://github.com/bug-ops/scrape-rs/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/bug-ops/scrape-rs/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/bug-ops/scrape-rs/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/bug-ops/scrape-rs/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/bug-ops/scrape-rs/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/bug-ops/scrape-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/bug-ops/scrape-rs/releases/tag/v0.1.0
