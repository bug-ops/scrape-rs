# scrape-cli

[![Crates.io](https://img.shields.io/crates/v/scrape-cli)](https://crates.io/crates/scrape-cli)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue)](https://github.com/bug-ops/scrape-rs)
[![License](https://img.shields.io/crates/l/scrape-cli)](../../LICENSE-MIT)

**10-50x faster** HTML extraction from command line. Rust-powered, shell-friendly.

## Installation

```bash
cargo install scrape-cli
```

<details>
<summary>Pre-built binaries</summary>

Download from [GitHub Releases](https://github.com/bug-ops/scrape-rs/releases):

```bash
# macOS (Apple Silicon)
curl -L https://github.com/bug-ops/scrape-rs/releases/latest/download/scrape-darwin-aarch64.tar.gz | tar xz

# macOS (Intel)
curl -L https://github.com/bug-ops/scrape-rs/releases/latest/download/scrape-darwin-x86_64.tar.gz | tar xz

# Linux (x86_64)
curl -L https://github.com/bug-ops/scrape-rs/releases/latest/download/scrape-linux-x86_64.tar.gz | tar xz

# Linux (ARM64)
curl -L https://github.com/bug-ops/scrape-rs/releases/latest/download/scrape-linux-aarch64.tar.gz | tar xz
```

</details>

> [!IMPORTANT]
> Requires Rust 1.88 or later when building from source.

## Quick start

```bash
# Extract h1 text from file
scrape 'h1' page.html

# Extract from stdin
curl -s example.com | scrape 'title'

# Extract links as JSON
scrape -o json 'a[href]' page.html
```

## Usage

<details open>
<summary><strong>Basic extraction</strong></summary>

```bash
# Extract text content
scrape 'h1' page.html
# Output: Welcome to Our Site

# Extract attribute value
scrape -a href 'a.nav-link' page.html
# Output: /home
#         /about
#         /contact

# First match only
scrape -1 'p' page.html
# Output: First paragraph text
```

</details>

<details>
<summary><strong>Output formats</strong></summary>

```bash
# Plain text (default)
scrape 'h1' page.html
# Output: Hello World

# JSON
scrape -o json 'a[href]' page.html
# Output: ["Link 1","Link 2"]

# Pretty JSON
scrape -o json -p 'a' page.html

# HTML fragments
scrape -o html 'div.content' page.html

# CSV (requires named selectors)
scrape -o csv -s name='td:nth-child(1)' -s price='td:nth-child(2)' table.html
# Output: name,price
#         "Product A","$10.00"
```

</details>

<details>
<summary><strong>Named selectors</strong></summary>

```bash
# Extract multiple fields
scrape -o json \
  -s title='h1' \
  -s links='a[href]' \
  -s images='img[src]' \
  page.html
# Output: {"title":["Page Title"],"links":[...],"images":[...]}
```

</details>

<details>
<summary><strong>Batch processing</strong></summary>

```bash
# Process multiple files (parallel by default)
scrape 'h1' pages/*.html
# Output: page1.html: Welcome
#         page2.html: About Us
#         page3.html: Contact

# Control parallelism
scrape -j 4 'h1' pages/*.html
```

> [!TIP]
> Batch processing uses all CPU cores by default. Use `-j N` to limit threads.

</details>

<details>
<summary><strong>Pipeline integration</strong></summary>

```bash
# NUL delimiter for xargs
scrape -0 -a href 'a' page.html | xargs -0 -I{} curl {}

# Suppress errors
scrape -q 'h1' *.html 2>/dev/null

# Disable filename prefix
scrape --no-filename 'h1' *.html
```

</details>

## Options

| Option | Short | Description |
|--------|-------|-------------|
| `--output FORMAT` | `-o` | Output format: text, json, html, csv |
| `--select NAME=SEL` | `-s` | Named selector extraction |
| `--attribute ATTR` | `-a` | Extract attribute instead of text |
| `--first` | `-1` | Return only first match |
| `--pretty` | `-p` | Pretty-print JSON output |
| `--null` | `-0` | Use NUL delimiter (for xargs) |
| `--color MODE` | `-c` | Colorize: auto, always, never |
| `--parallel N` | `-j` | Parallel threads for batch |
| `--quiet` | `-q` | Suppress error messages |
| `--with-filename` | `-H` | Always show filename prefix |
| `--no-filename` | | Never show filename prefix |

## Performance

Performance improvements:

- **SIMD-accelerated parsing** — 2-10x faster class selector matching on large documents
- **Batch parallelization** — Scales near-linearly with thread count when processing multiple files
- **Zero-copy serialization** — 50-70% memory reduction in output generation

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success, matches found |
| 1 | No matches found |
| 2 | Runtime error (invalid selector, I/O error) |
| 4 | Argument validation error |

## Built on Servo and Cloudflare

**Parsing & Selection (Servo browser engine):**
- [html5ever](https://crates.io/crates/html5ever) — Spec-compliant HTML5 parser
- [selectors](https://crates.io/crates/selectors) — CSS selector matching engine

**Streaming Parser (Cloudflare):**
- [lol_html](https://github.com/cloudflare/lol_html) — High-performance streaming HTML parser with constant-memory event-driven API

## Related packages

| Platform | Package |
|----------|---------|
| Rust | [`scrape-core`](https://crates.io/crates/scrape-core) |
| Python | [`fast-scrape`](https://pypi.org/project/fast-scrape) |
| Node.js | [`@fast-scrape/node`](https://www.npmjs.com/package/@fast-scrape/node) |
| WASM | [`@fast-scrape/wasm`](https://www.npmjs.com/package/@fast-scrape/wasm) |

## License

MIT OR Apache-2.0
