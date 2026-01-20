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
> Requires Python 3.10 or later. v0.2.0 introduces type-safe document lifecycle with zero performance overhead.

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

Measured benchmarks comparing against BeautifulSoup4 and lxml:

<details open>
<summary><strong>Parse speed comparison</strong></summary>

| File size | fast-scrape | BeautifulSoup4 | lxml | vs BS4 | vs lxml |
|-----------|-------------|----------------|------|--------|---------|
| 1 KB | **0.030 ms** | 0.247 ms | 0.015 ms | **8.3x faster** | 2x slower |
| 218 KB | **3.79 ms** | 30.02 ms | 1.79 ms | **7.9x faster** | 2x slower |
| 5.9 MB | **118.88 ms** | 1095.22 ms | 71.59 ms | **9.2x faster** | 1.7x slower |

**Average:** **8.5x faster than BeautifulSoup4**

> [!NOTE]
> lxml uses libxml2 (optimized C library) and is faster for parsing. However, fast-scrape dominates in query operations (see below).

</details>

<details>
<summary><strong>Query performance (on 218 KB HTML)</strong></summary>

| Operation | fast-scrape | BeautifulSoup4 | Speedup |
|-----------|-------------|----------------|---------|
| `find("div")` | **0.001 ms** | 0.016 ms | **20x** |
| `find(".product-card")` | **<0.001 ms** | 0.830 ms | **7,353x** |
| `find("#product-100")` | **<0.001 ms** | 0.828 ms | **6,928x** |
| `find_all("div")` | **0.037 ms** | 0.310 ms | **8x** |
| `select(".product-card")` | **0.004 ms** | 4.705 ms | **1,294x** |

**Average:** **3,121x faster than BeautifulSoup4**

**CSS selectors dominate:** Class and ID selectors are **thousands of times faster**.

</details>

**When to use fast-scrape:**
- **Web scraping** — Many queries per document (fast-scrape excels)
- **Data extraction** — CSS selector-heavy workloads
- **Batch processing** — `Soup.parse_batch()` uses all CPU cores

**When lxml might be better:**
- **Parse-only workloads** — If you only parse and don't query, lxml is faster
- **XML processing** — lxml has better XML support

**v0.2.0 optimizations:**
- **SIMD-accelerated CSS selectors** — 2-10x faster class/ID matching
- **Zero-copy serialization** — 50-70% memory reduction
- **Batch processing** — Rayon parallelization across CPU cores

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
| Node.js | [`@fast-scrape/node`](https://www.npmjs.com/package/@fast-scrape/node) |
| WASM | [`@fast-scrape/wasm`](https://www.npmjs.com/package/@fast-scrape/wasm) |

## License

MIT OR Apache-2.0
