# Contributing to scrape-rs

Thank you for your interest in contributing to scrape-rs! This document provides guidelines and instructions for contributing.

## Code of Conduct

This project adheres to the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and constructive in all interactions. We welcome contributors of all experience levels.

## Getting Started

### Prerequisites

- Rust 1.88+ (check `rust-toolchain.toml` for MSRV)
- Python 3.12+ with [uv](https://github.com/astral-sh/uv)
- Node.js 22+ with [pnpm](https://pnpm.io/)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/) for WASM builds

### Setup

```bash
# Clone the repository
git clone https://github.com/bug-ops/scrape-rs.git
cd scrape-rs

# Build workspace
cargo build

# Run tests
cargo nextest run
```

## Development Workflow

### Branch Naming

- `feat/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `ci/` - CI/CD changes

### Making Changes

1. **Fork** the repository
2. **Create a branch** from `main`
3. **Make your changes** following the style guidelines
4. **Write tests** for new functionality
5. **Run verification** before committing
6. **Submit a PR** with a clear description

### Verification Commands

Always run these before submitting a PR:

```bash
# Rust (core library)
cargo +nightly fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run

# Python bindings
cd crates/scrape-py
uv run ruff check .
uv run ruff format --check .
uv run maturin develop
uv run pytest

# Node.js bindings
cd crates/scrape-node
pnpm exec biome check .
pnpm run build
pnpm test

# WASM bindings
cd crates/scrape-wasm
pnpm exec biome check .
wasm-pack build --target web --release
```

## Code Style

### Rust

- Use `rustfmt` (nightly) for formatting
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Avoid excessive comments; only explain complex logic
- Use `// TODO:` for deferred work
- Doc comments (`///`) required for public API

### Python

- Use `ruff` for linting and formatting
- Follow PEP 8 conventions

### TypeScript/JavaScript

- Use `biome` for linting and formatting

## Architecture

### Workspace Structure

```
crates/
├── scrape-core/    # Pure Rust library (no FFI)
├── scrape-py/      # Python bindings (PyO3)
├── scrape-node/    # Node.js bindings (napi-rs)
└── scrape-wasm/    # WASM bindings (wasm-bindgen)
```

### Key Principles

1. **scrape-core must remain pure Rust** - no FFI, no platform-specific code
2. **API consistency** - identical interface across all bindings
3. **Performance first** - benchmark before/after changes
4. **WASM size budget** - keep bundle under 500KB

## Testing

- **Unit tests** - in the same file as the code (`#[cfg(test)]`)
- **Integration tests** - in `tests/` directory
- **Benchmarks** - use `criterion` in `benches/`

```bash
# Run specific test
cargo nextest run test_name

# Run benchmarks
cargo bench

# Run with coverage
cargo llvm-cov
```

## Pull Request Guidelines

### PR Title

Use conventional commit format:
- `feat: add streaming parser API`
- `fix: handle malformed UTF-8`
- `docs: update Python examples`
- `refactor: simplify arena allocator`

### PR Description

- Summarize changes in 1-3 bullet points
- List affected bindings
- Describe test plan
- Link related issues

### Review Process

1. CI must pass
2. At least one maintainer approval
3. No unresolved conversations
4. Up-to-date with `main`

## Reporting Issues

### Bug Reports

Use the bug report template and include:
- Which binding (Rust/Python/Node/WASM)
- Version number
- Minimal reproduction
- Expected vs actual behavior

### Feature Requests

Use the feature request template and describe:
- Problem you're solving
- Proposed solution
- Alternatives considered

## License

By contributing, you agree that your contributions will be licensed under the same dual MIT/Apache-2.0 license as the project.

## Questions?

- Open a [Discussion](https://github.com/bug-ops/scrape-rs/discussions)
- Check existing issues and PRs
