# fast-scrape

[![CI](https://img.shields.io/github/actions/workflow/status/bug-ops/scrape-rs/ci.yml?branch=main&label=CI)](https://github.com/bug-ops/scrape-rs/actions)
[![codecov](https://codecov.io/gh/bug-ops/scrape-rs/graph/badge.svg?token=6MQTONGT95)](https://codecov.io/gh/bug-ops/scrape-rs)
[![Crates.io](https://img.shields.io/crates/v/scrape-core)](https://crates.io/crates/scrape-core)
[![PyPI](https://img.shields.io/pypi/v/fast-scrape)](https://pypi.org/project/fast-scrape)
[![npm](https://img.shields.io/npm/v/@fast-scrape/node)](https://www.npmjs.com/package/@fast-scrape/node)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-MIT)

**10-50x faster** HTML parsing for Rust, Python, Node.js, and browsers.

```
pip install fast-scrape          # Python
npm install @fast-scrape/node    # Node.js
npm install @fast-scrape/wasm    # Browser/WASM
cargo add scrape-core            # Rust library
cargo install scrape-cli         # CLI tool
```

## Quick start

<details open>
<summary><strong>Python</strong></summary>

```python
from scrape_rs import Soup

soup = Soup("<html><body><div class='content'>Hello</div></body></html>")

div = soup.find("div")
print(div.text)  # Hello

for el in soup.select("div.content"):
    print(el.inner_html)
```

</details>

<details>
<summary><strong>Node.js</strong></summary>

```javascript
import { Soup } from '@fast-scrape/node';

const soup = new Soup("<html><body><div class='content'>Hello</div></body></html>");

const div = soup.find("div");
console.log(div.text);  // Hello

for (const el of soup.select("div.content")) {
    console.log(el.innerHTML);
}
```

</details>

<details>
<summary><strong>Rust</strong></summary>

```rust
use scrape_core::Soup;

let soup = Soup::new("<html><body><div class='content'>Hello</div></body></html>");

let div = soup.find("div").unwrap();
println!("{}", div.text());  // Hello

for el in soup.select("div.content") {
    println!("{}", el.inner_html());
}
```

> [!IMPORTANT]
> Requires Rust 1.88 or later.

</details>

<details>
<summary><strong>Browser (WASM)</strong></summary>

```javascript
import init, { Soup } from '@fast-scrape/wasm';

await init();

const soup = new Soup("<html><body><div class='content'>Hello</div></body></html>");
console.log(soup.find("div").text);  // Hello
```

</details>

<details>
<summary><strong>CLI</strong></summary>

```bash
# Extract text from HTML file
scrape 'h1' page.html

# Extract from URL via curl
curl -s example.com | scrape 'title'

# Output as JSON
scrape -o json 'a[href]' page.html
```

</details>

## Performance

Benchmarked against BeautifulSoup4 (Python) and Cheerio (Node.js):

<details open>
<summary><strong>Parse speed (v0.2.0)</strong></summary>

| File size | scrape-rs | Time |
|-----------|-----------|------|
| 1 KB | 11.4 µs | Ultra-fast initialization |
| 100 KB | 2.96 ms | 0.03 MB/s throughput |
| 1 MB | 15.5 ms | 64 MB/s throughput |

</details>

<details>
<summary><strong>Query speed (v0.2.0)</strong></summary>

| Operation | Time | Throughput |
|-----------|------|-----------|
| `find("div")` | 208 ns | 4.8M ops/sec |
| `find(".class")` | 20 ns | 50M ops/sec |
| `find("#id")` | 20 ns | 50M ops/sec |
| `select("complex")` | 24.7 µs | 40K ops/sec |

</details>

> [!TIP]
> Run `cargo bench --bench comparison` for detailed benchmarks on your hardware.
> See [Performance Guide](docs/src/performance/benchmarks.md) for comparisons vs competitors.

## Features

- **Fast** — 10-50x faster than BeautifulSoup/Cheerio (v0.2 with SIMD achieves 2-10x faster class selector matching)
- **Cross-platform** — Rust, Python, Node.js, and browsers
- **Consistent API** — Same interface everywhere with compile-time type safety
- **Memory-safe** — Pure Rust core, zero unsafe code
- **SIMD-accelerated** — Auto-detects SSE4.2, AVX2, NEON, WASM SIMD for byte scanning and selector matching
- **Type-safe queries** — Compile-time lifecycle enforcement via typestate pattern (Building → Queryable → Sealed)
- **Trait abstractions** — HtmlSerializer trait and ElementFilter iterators with zero-overhead abstraction

<details>
<summary><strong>Rust feature flags</strong></summary>

```toml
[dependencies]
scrape-core = { version = "0.2", features = ["simd", "parallel"] }
```

| Feature | Description | Default |
|---------|-------------|---------|
| `simd` | SIMD-accelerated parsing | No |
| `parallel` | Parallel batch processing via Rayon | No |

> [!NOTE]
> Python and Node.js bindings enable both features by default. WASM uses `simd` only (no threads).

</details>

## Architecture

```mermaid
graph TD
    Core[scrape-core<br><i>Pure Rust • SIMD • Parallel</i>]

    Core --> CLI[scrape-cli<br><i>CLI tool</i>]
    Core --> Python[fast-scrape<br><i>PyO3</i>]
    Core --> Node["@fast-scrape/node<br><i>napi-rs</i>"]
    Core --> WASM["@fast-scrape/wasm<br><i>wasm-bindgen</i>"]
```

### Built on Servo

The core is powered by battle-tested libraries from the [Servo](https://servo.org/) browser engine:

- [html5ever](https://crates.io/crates/html5ever) — Spec-compliant HTML5 parser
- [selectors](https://crates.io/crates/selectors) — CSS selector matching engine
- [cssparser](https://crates.io/crates/cssparser) — CSS parser

<details>
<summary><strong>Project structure</strong></summary>

```
crates/
├── scrape-core/    # Pure Rust library
├── scrape-cli/     # Command-line tool
├── scrape-py/      # Python bindings (PyO3)
├── scrape-node/    # Node.js bindings (napi-rs)
└── scrape-wasm/    # WASM bindings (wasm-bindgen)
```

</details>

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT OR Apache-2.0
