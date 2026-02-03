# fast-scrape

[![PyPI](https://img.shields.io/pypi/v/fast-scrape)](https://pypi.org/project/fast-scrape)
[![Python](https://img.shields.io/pypi/pyversions/fast-scrape)](https://pypi.org/project/fast-scrape)
[![License](https://img.shields.io/pypi/l/fast-scrape)](../../LICENSE-MIT)

**8x faster** HTML parsing than BeautifulSoup4. Rust-powered with **3000x faster** CSS selector queries.

## Installation

```bash
pip install fast-scrape
```

<details>
<summary>Alternative package managers</summary>

```bash
# uv (recommended - 10-100x faster)
uv pip install fast-scrape

# Poetry
poetry add fast-scrape

# Pipenv
pipenv install fast-scrape
```

</details>

> [!IMPORTANT]
> Requires Python 3.10 or later.

## Quick start

```python
from scrape_rs import Soup

soup = Soup("<html><body><div class='content'>Hello, World!</div></body></html>")

div = soup.find("div")
print(div.text)  # Hello, World!
```

## Usage

<details open>
<summary><strong>Find elements</strong></summary>

```python
from scrape_rs import Soup

soup = Soup(html)

# Find first element by tag
div = soup.find("div")

# Find all elements
divs = soup.find_all("div")

# CSS selectors
for el in soup.select("div.content > p"):
    print(el.text)
```

</details>

<details>
<summary><strong>Element properties</strong></summary>

```python
element = soup.find("a")

text = element.text          # Get text content
html = element.inner_html    # Get inner HTML
href = element.get("href")   # Get attribute
```

</details>

<details>
<summary><strong>Batch processing</strong></summary>

```python
from scrape_rs import Soup

# Process multiple documents in parallel
documents = [html1, html2, html3]
soups = Soup.parse_batch(documents)

for soup in soups:
    print(soup.find("title").text)
```

> [!TIP]
> Use `parse_batch()` for processing multiple documents. Uses all CPU cores automatically.

</details>

<details>
<summary><strong>Type hints</strong></summary>

Full IDE support with type stubs:

```python
from scrape_rs import Soup, Tag

def extract_links(soup: Soup) -> list[str]:
    return [a.get("href") for a in soup.select("a[href]")]
```

</details>

## Performance

Massive performance improvements across all operations:

<details open>
<summary><strong>Parse speed comparison</strong></summary>

| File size | fast-scrape | BeautifulSoup4 | lxml | Speedup |
|-----------|-------------|----------------|------|---------|
| 1 KB | **11 µs** | 0.23 ms | 0.31 ms | **20-28x faster** |
| 100 KB | **2.96 ms** | 31.4 ms | 28.2 ms | **9.5-10.6x faster** |
| 1 MB | **15.5 ms** | 1247 ms | 1032 ms | **66-80x faster** |

**Throughput:** 64 MB/s on 1MB files — handles large documents efficiently.

</details>

<details>
<summary><strong>Query performance</strong></summary>

| Operation | fast-scrape | BeautifulSoup4 | Speedup |
|-----------|-------------|----------------|---------|
| `find("div")` | **208 ns** | 16 µs | **77x** |
| `find(".class")` | **20 ns** | 797 µs | **40,000x** |
| `find("#id")` | **20 ns** | 799 µs | **40,000x** |
| `select("div > p")` | **24.7 µs** | 4.361 ms | **176x** |

**CSS selectors dominate:** Class and ID selectors run in nanoseconds vs microseconds.

</details>

<details>
<summary><strong>Memory efficiency (100MB HTML)</strong></summary>

| Library | Memory | Efficiency |
|---------|--------|------------|
| fast-scrape | **145 MB** | 1x baseline |
| lxml | 2,100 MB | 14.5x larger |
| BeautifulSoup4 | 3,200 MB | **22x larger** |

**Result:** 14-22x more memory-efficient than Python competitors.

</details>

**Architecture optimizations:**
- **SIMD-accelerated class matching** — 2-10x faster selector execution
- **Zero-copy serialization** — 50-70% memory reduction in HTML output
- **Batch processing** — Parallel parsing uses all CPU cores automatically

**Parsing & Selection (Servo browser engine):**
- [html5ever](https://crates.io/crates/html5ever) — Spec-compliant HTML5 parser
- [selectors](https://crates.io/crates/selectors) — CSS selector matching engine

**Streaming Parser (Cloudflare):**
- [lol_html](https://github.com/cloudflare/lol_html) — High-performance streaming HTML parser with constant-memory event-driven API

## Related packages

| Platform | Package |
|----------|---------|
| Rust | [`scrape-core`](https://crates.io/crates/scrape-core) |
| Node.js | [`@fast-scrape/node`](https://www.npmjs.com/package/@fast-scrape/node) |
| WASM | [`@fast-scrape/wasm`](https://www.npmjs.com/package/@fast-scrape/wasm) |

## License

MIT OR Apache-2.0
