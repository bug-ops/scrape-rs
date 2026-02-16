# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.3] - 2026-02-16

### Dependencies

- Bump workspace Rust dependencies (rust-minor group)
- Bump @biomejs/biome in Node.js and WASM packages

## [0.2.2] - 2026-02-03

### Fixed

- Switch CLI from `native-tls` to `rustls` for cross-compilation support (fixes linux-aarch64 build)

## [0.2.1] - 2026-02-03

### Added

- **Phase 18: Streaming & Advanced Performance**
  - Full lol_html streaming integration with constant O(1) memory usage
  - StreamingParser with event-driven architecture for large documents
  - Memory-mapped file support via `mmap` feature
  - Streaming text extraction and element handlers

- **Phase 19: Developer Experience**
  - Enhanced error tracking and diagnostics
  - Improved error messages with source context
  - Better panic handling and recovery

- **Phase 20: Benchmark Infrastructure**
  - Comprehensive benchmark suite with criterion
  - Cross-platform performance comparison tools
  - Memory profiling infrastructure

- **Documentation**
  - mdBook-based documentation site
  - Getting Started guide
  - User Guide with examples
  - API reference documentation

### Changed

- Updated PyO3 to 0.28.0 with `skip_from_py_object` compatibility fix
- Dependabot auto-merge workflow for patch updates

### Fixed

- CI wait-on-check circular dependency issue
- Dependabot workflow auto-approve removal

### Dependencies

- pyo3: 0.27.2 → 0.28.0
- bytes: 1.11.0 → 1.11.1
- clap: 4.5.54 → 4.5.56
- actions/checkout: 4 → 6
- actions/cache: 4 → 5
- actions/upload-pages-artifact: 3 → 4
- @biomejs/biome: updated in Node.js and WASM packages

## [0.2.0] - 2026-01-20

### Added

- **Phase 14: Performance Optimization**
  - SIMD-accelerated class selector matching with automatic platform detection (SSE4.2/AVX2/NEON/SIMD128)
  - TagId enum for efficient HTML5 tag interning (113 tags with O(1) lookup)
  - DocumentIndex for optimized ID and class-based lookups (O(1) ID, O(k) class)
  - Rayon-powered batch parsing via `parse_batch()` for parallel document processing
  - Selector fast-paths for common patterns (tag only, class only, ID only)
  - Query optimization with compiled selector caching and pre-compiled selector support

- **Phase 15: Core Utilities Extraction**
  - New `scrape-core/src/utils.rs` module with shared HTML escaping utilities
  - New `scrape-core/src/serialize.rs` module with centralized HTML serialization
  - `escape_text()` and `escape_attr()` with zero-copy Cow<str> optimization
  - `serialize_node()` and `collect_text()` functions for consistent HTML handling
  - Zero-copy optimization reduces allocations by 50-70% in typical HTML serialization

- **Phase 16: Trait Abstractions and Iterator Extensions**
  - HtmlSerializer trait for unified HTML serialization API on Tag type
  - ElementFilter iterator extensions (.elements()) for element-only iteration
  - 6 new iterator types: ElementChildrenIter, ElementDescendantsIter, ElementAncestorsIter, etc.
  - Simplified binding navigation code by 45% per method across all platforms (Python/Node.js/WASM)
  - Comprehensive benchmark suite validating zero-overhead abstractions

- **Phase 17: Advanced Type Safety with Typestate Patterns**
  - DocumentState sealed trait with three lifecycle states: Building, Queryable, Sealed
  - Compile-time enforced document lifecycle via PhantomData (zero runtime overhead)
  - Type-safe state transitions: Building → Queryable → Sealed
  - NodeType sealed trait preventing external implementations with private module pattern
  - Marker types (ElementMarker, TextMarker, CommentMarker) for enhanced type safety
  - Full backward compatibility via Document type alias (Document = DocumentImpl<Queryable>)

### Changed

- **Breaking Changes (Internal API)**
  - Document is now DocumentImpl<Queryable> with generic state parameter
  - Parser APIs now use DocumentImpl<Building> during construction
  - Public Document type alias maintains backward compatibility
  - New types exported: DocumentImpl, DocumentState, Building, Queryable, Sealed, NodeType, markers

- Performance improvements across all binding libraries (Python/Node.js/WASM)
- Eliminated 308 lines of duplicated code across bindings
- More efficient DOM traversal with cached selector state
- Reduced memory allocations in text extraction and HTML serialization

### Fixed

- Resolved 5 FIXME/TODO markers related to code duplication and unfinished implementations
- Fixed inconsistent text escaping patterns between core and bindings
- Improved consistency in HTML serialization across all platforms

### Security

- Zero unsafe code in Phase 17 implementation
- All sealed traits prevent unintended implementations
- Type-safe state transitions prevent invalid operations at compile time

### Performance

- **Query Performance**: SIMD-accelerated class matching (2-10x improvement on large documents)
- **Memory**: 50-70% reduction in serialization allocations via Cow<str>
- **Parallel**: Batch parsing scales near-linearly with thread count (Rayon)
- **Zero-cost abstractions**: Phase 16-17 traits generate identical code to manual implementations

### Test Coverage

- 1,121 comprehensive tests passing across all platforms
- 506 Rust unit tests + 116 doctests
- 180 Python tests, 201 Node.js tests, 118 WASM tests
- 100% coverage on all new modules (utils.rs, serialize.rs, state.rs, node_type.rs)
- Compile-time type safety verified via sealed traits and typestate patterns

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

[Unreleased]: https://github.com/bug-ops/scrape-rs/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/bug-ops/scrape-rs/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/bug-ops/scrape-rs/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/bug-ops/scrape-rs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/bug-ops/scrape-rs/compare/v0.1.6...v0.2.0
[0.1.6]: https://github.com/bug-ops/scrape-rs/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/bug-ops/scrape-rs/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/bug-ops/scrape-rs/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/bug-ops/scrape-rs/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/bug-ops/scrape-rs/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/bug-ops/scrape-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/bug-ops/scrape-rs/releases/tag/v0.1.0
