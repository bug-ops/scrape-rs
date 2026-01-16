# scrape-rs (Python)

[![PyPI](https://img.shields.io/pypi/v/scrape-rs)](https://pypi.org/project/scrape-rs)
[![Python](https://img.shields.io/pypi/pyversions/scrape-rs)](https://pypi.org/project/scrape-rs)
[![codecov](https://codecov.io/gh/bug-ops/scrape-rs/graph/badge.svg?token=6MQTONGT95&flag=python)](https://codecov.io/gh/bug-ops/scrape-rs)
[![License](https://img.shields.io/pypi/l/scrape-rs)](../../LICENSE-MIT)

Python bindings for scrape-rs, a high-performance HTML parsing library.

## Installation

```bash
pip install scrape-rs
```

Alternative package managers:

```bash
# uv (recommended - 10-100x faster)
uv pip install scrape-rs

# Poetry
poetry add scrape-rs

# Pipenv
pipenv install scrape-rs
```

> [!IMPORTANT]
> Requires Python 3.10 or later.

## Quick start

```python
from scrape_rs import Soup

html = "<html><body><div class='content'>Hello, World!</div></body></html>"
soup = Soup(html)

div = soup.find("div")
print(div.text)
# Hello, World!
```

## Usage

### Find elements

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

### Element properties

```python
element = soup.find("a")

# Get text content
text = element.text

# Get inner HTML
html = element.inner_html

# Get attribute
href = element.get("href")
```

### Batch processing

```python
from scrape_rs import Soup

# Process multiple documents in parallel
documents = [html1, html2, html3]
soups = Soup.parse_batch(documents)

for soup in soups:
    print(soup.find("title").text)
```

> [!TIP]
> Use `parse_batch()` for processing multiple documents. It uses all CPU cores automatically.

## Type hints

This package includes type stubs for full IDE support:

```python
from scrape_rs import Soup, Tag

def extract_links(soup: Soup) -> list[str]:
    return [a.get("href") for a in soup.select("a[href]")]
```

## Performance

Compared to BeautifulSoup on the same HTML documents:

| Operation | Speedup |
|-----------|---------|
| Parse (1 KB) | **9.7x** faster |
| Parse (219 KB) | **9.2x** faster |
| Parse (5.9 MB) | **10.6x** faster |
| `find(".class")` | **132x** faster |
| `select(".class")` | **40x** faster |

> [!TIP]
> Run `python benches/compare_python.py` from the project root to benchmark on your hardware.

## Related packages

Part of the [scrape-rs](https://github.com/bug-ops/scrape-rs) project:

- `scrape-core` — Rust core library
- `scrape-rs` (npm) — Node.js bindings
- `@scrape-rs/wasm` — Browser/WASM bindings

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or [MIT License](../../LICENSE-MIT) at your option.
