# Migration Overview

scrape-rs provides a consistent API across platforms while being significantly faster than existing HTML parsing libraries. This guide helps you migrate from other popular libraries.

## Why Migrate?

### Performance

scrape-rs is **10-50x faster** than most HTML parsing libraries:

| Library | Language | Parse 100KB | Query | Extract Links |
|---------|----------|-------------|-------|---------------|
| BeautifulSoup | Python | 18ms | 0.80ms | 3.2ms |
| lxml | Python | 8ms | 0.15ms | 1.1ms |
| Cheerio | Node.js | 12ms | 0.12ms | 0.85ms |
| scraper | Rust | 3.2ms | 0.015ms | 0.22ms |
| **scrape-rs** | **All** | **1.8ms** | **0.006ms** | **0.18ms** |

### Memory Efficiency

scrape-rs uses arena allocation with compact node representation:

| Library | Memory per Node | 100KB Document |
|---------|----------------|----------------|
| BeautifulSoup | ~300 bytes | ~15 MB |
| lxml | ~150 bytes | ~7.5 MB |
| Cheerio | ~200 bytes | ~10 MB |
| **scrape-rs** | **~50 bytes** | **~2.5 MB** |

### Cross-Platform Consistency

Same API across Rust, Python, Node.js, and WASM:

```python
# Python
soup = Soup(html)
div = soup.find("div.product")
```

```javascript
// Node.js - identical API
const soup = new Soup(html);
const div = soup.find("div.product");
```

```rust
// Rust - identical API
let soup = Soup::parse(html);
let div = soup.find("div.product")?;
```

## Migration Guides

Detailed migration guides for the following libraries are coming in Phase 20 Week 2:

- **BeautifulSoup** (Python) - 10-50x performance improvement
- **Cheerio** (Node.js) - 6-20x performance improvement
- **lxml** (Python) - Simpler API with comparable HTML performance
- **scraper Crate** (Rust) - Better performance with cross-platform bindings

**When to migrate:**
- Need Python/Node.js bindings
- Want streaming support
- Need better performance for large documents

**Compatibility:** ~80% API compatible

## Migration Strategy

### 1. Side-by-Side Testing

Run both libraries in parallel during development:

```python
# Python example
from bs4 import BeautifulSoup
from scrape_rs import Soup

# Parse with both
soup_bs4 = BeautifulSoup(html, 'lxml')
soup_scrape = Soup(html)

# Compare results
result_bs4 = soup_bs4.find("div", class_="product").text
result_scrape = soup_scrape.find("div.product").text

assert result_bs4.strip() == result_scrape.strip()
```

### 2. Gradual Rollout

Start with non-critical code paths:

```python
USE_SCRAPE_RS = os.getenv("USE_SCRAPE_RS", "false") == "true"

if USE_SCRAPE_RS:
    from scrape_rs import Soup as Parser
else:
    from bs4 import BeautifulSoup as Parser

soup = Parser(html)
```

### 3. Performance Testing

Benchmark before and after migration:

```python
import time
from scrape_rs import Soup

start = time.time()
for html in documents:
    soup = Soup(html)
    results = soup.find_all("div.product")
    # Process results...
end = time.time()

print(f"Processed {len(documents)} documents in {end - start:.2f}s")
```

## Common Patterns

### Query Syntax Differences

| Pattern | BeautifulSoup/lxml | Cheerio | scrape-rs |
|---------|-------------------|---------|-----------|
| Find by class | `find(class_="item")` | `$(".item")` | `find(".item")` |
| Find by id | `find(id="header")` | `$("#header")` | `find("#header")` |
| Find by tag | `find("div")` | `$("div")` | `find("div")` |
| Find all | `find_all("div")` | `$("div")` | `find_all("div")` |

### Text Extraction

| Library | Method | Whitespace Handling |
|---------|--------|---------------------|
| BeautifulSoup | `.get_text()` or `.text` | Manual `strip()` needed |
| Cheerio | `.text()` | Automatic trim |
| lxml | `.text_content()` | Manual `strip()` needed |
| scrape-rs | `.text` | Automatic normalize |

### Attribute Access

| Library | Get Attribute | Check Existence |
|---------|---------------|-----------------|
| BeautifulSoup | `tag.get("href")` or `tag["href"]` | `tag.has_attr("href")` |
| Cheerio | `elem.attr("href")` | `elem.attr("href") !== undefined` |
| lxml | `elem.get("href")` | `"href" in elem.attrib` |
| scrape-rs | `tag.get("href")` | `tag.has_attr("href")` |

## Known Limitations

### Not Supported

scrape-rs intentionally does not support:

1. **DOM Modification**: The DOM is immutable after parsing
   - No `.append()`, `.insert()`, `.remove()` methods
   - Use HTML rewriting for output modification

2. **XML Parsing**: Only HTML5 parsing is supported
   - No XML namespaces
   - No XML declaration handling

3. **Encoding Detection**: Input must be UTF-8
   - Use chardet/encoding libraries before parsing
   - Or convert to UTF-8 first

### Performance Trade-offs

scrape-rs optimizes for:
- Parse speed
- Query speed
- Memory efficiency

At the cost of:
- No modification support
- Immutable DOM only

## Getting Help

If you encounter issues during migration:

1. Read the [API documentation](https://docs.rs/scrape-core)
2. Check the [Getting Started](../getting-started/quick-start.md) guide
3. Open an issue on [GitHub](https://github.com/bug-ops/scrape-rs)

Detailed migration guides for BeautifulSoup, Cheerio, lxml, and scraper are coming in Phase 20 Week 2.
