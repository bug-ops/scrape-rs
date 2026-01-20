# fast-scrape

[![PyPI](https://img.shields.io/pypi/v/fast-scrape)](https://pypi.org/project/fast-scrape)
[![Python](https://img.shields.io/pypi/pyversions/fast-scrape)](https://pypi.org/project/fast-scrape)
[![License](https://img.shields.io/pypi/l/fast-scrape)](../../LICENSE-MIT)

**10-50x faster** HTML parsing for Python. Rust-powered, BeautifulSoup-compatible API.

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

v0.2.0 improvements:

- **SIMD-accelerated** — Class selector matching 2-10x faster on large documents
- **Zero-copy serialization** — 50-70% memory reduction in HTML output
- **Batch processing** — Parallel parsing across multiple documents uses all CPU cores

Compared to BeautifulSoup:

| Operation | Speedup |
|-----------|---------|
| Parse (1 KB) | **9.7x** faster |
| Parse (5.9 MB) | **10.6x** faster |
| `find(".class")` | **132x** faster |
| `select(".class")` | **40x** faster |

## Built on Servo

Powered by battle-tested libraries from the [Servo](https://servo.org/) browser engine: [html5ever](https://crates.io/crates/html5ever) (HTML5 parser) and [selectors](https://crates.io/crates/selectors) (CSS selector engine).

## Related packages

| Platform | Package |
|----------|---------|
| Rust | [`scrape-core`](https://crates.io/crates/scrape-core) |
| Node.js | [`@fast-scrape/node`](https://www.npmjs.com/package/@fast-scrape/node) |
| WASM | [`@fast-scrape/wasm`](https://www.npmjs.com/package/@fast-scrape/wasm) |

## License

MIT OR Apache-2.0
