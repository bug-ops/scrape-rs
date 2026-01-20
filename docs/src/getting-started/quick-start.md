# Quick Start

This guide will get you parsing and querying HTML in under 5 minutes.

## Your First Program

### Rust

```rust
use scrape_core::Soup;

fn main() {
    let html = r#"
        <html>
            <body>
                <div class="product">
                    <h2>Laptop</h2>
                    <span class="price">$999</span>
                </div>
            </body>
        </html>
    "#;

    let soup = Soup::parse(html);

    if let Ok(Some(product)) = soup.find(".product") {
        let name = product.find("h2")
            .ok()
            .flatten()
            .map(|t| t.text())
            .unwrap_or_default();

        let price = product.find(".price")
            .ok()
            .flatten()
            .map(|t| t.text())
            .unwrap_or_default();

        println!("Product: {}, Price: {}", name, price);
    }
}
```

Output:
```
Product: Laptop, Price: $999
```

### Python

```python
from scrape_rs import Soup

html = """
<html>
    <body>
        <div class="product">
            <h2>Laptop</h2>
            <span class="price">$999</span>
        </div>
    </body>
</html>
"""

soup = Soup(html)
product = soup.find(".product")

if product:
    name = product.find("h2").text
    price = product.find(".price").text
    print(f"Product: {name}, Price: {price}")
```

### Node.js

```typescript
import { Soup } from '@scrape-rs/scrape';

const html = `
<html>
    <body>
        <div class="product">
            <h2>Laptop</h2>
            <span class="price">$999</span>
        </div>
    </body>
</html>
`;

const soup = new Soup(html);
const product = soup.find(".product");

if (product) {
    const name = product.find("h2").text;
    const price = product.find(".price").text;
    console.log(`Product: ${name}, Price: ${price}`);
}
```

### WASM

```typescript
import init, { Soup } from '@scrape-rs/wasm';

await init();

const html = `
<html>
    <body>
        <div class="product">
            <h2>Laptop</h2>
            <span class="price">$999</span>
        </div>
    </body>
</html>
`;

const soup = new Soup(html);
const product = soup.find(".product");

if (product) {
    const name = product.find("h2").text;
    const price = product.find(".price").text;
    console.log(`Product: ${name}, Price: ${price}`);
}
```

## Core Concepts

### Parsing

scrape-rs parses HTML into a document object model (DOM):

```rust
let soup = Soup::parse(html_string);
```

The parser is:
- **Spec-compliant**: Uses html5ever for HTML5 parsing
- **Forgiving**: Handles malformed HTML gracefully
- **Fast**: Parses 100KB in ~2ms on modern hardware

### Finding Elements

Use CSS selectors to find elements:

```rust
// Find first matching element
let element = soup.find("div.product")?;

// Find all matching elements
let elements = soup.find_all("div.product")?;
```

Supported selectors:
- Type: `div`, `span`, `a`
- Class: `.product`, `.price`
- ID: `#main`, `#header`
- Attributes: `[href]`, `[type="text"]`
- Combinators: `div > span`, `h1 + p`, `div span`
- Pseudo-classes: `:first-child`, `:last-child`, `:nth-child(2n)`

### Extracting Data

Once you have an element, extract its content:

```rust
let tag = soup.find("h1")?.unwrap();

// Get text content
let text = tag.text();

// Get HTML content
let html = tag.html();

// Get attribute value
if let Some(href) = tag.get("href") {
    println!("Link: {}", href);
}

// Check if attribute exists
if tag.has_attr("data-id") {
    // ...
}

// Check for CSS class
if tag.has_class("active") {
    // ...
}
```

### Navigating the Tree

Traverse the DOM tree:

```rust
let tag = soup.find("span")?.unwrap();

// Parent element
if let Some(parent) = tag.parent() {
    println!("Parent: {}", parent.name().unwrap());
}

// Children
for child in tag.children() {
    println!("Child: {:?}", child.name());
}

// Next sibling
if let Some(next) = tag.next_sibling() {
    println!("Next: {:?}", next.name());
}

// Previous sibling
if let Some(prev) = tag.prev_sibling() {
    println!("Previous: {:?}", prev.name());
}
```

## Common Patterns

### Extract All Links

```rust
let soup = Soup::parse(html);

for link in soup.find_all("a[href]")? {
    if let Some(href) = link.get("href") {
        println!("{}", href);
    }
}
```

### Extract Table Data

```rust
let soup = Soup::parse(html);

if let Ok(Some(table)) = soup.find("table") {
    for row in table.find_all("tr")? {
        let cells: Vec<String> = row
            .find_all("td")?
            .iter()
            .map(|cell| cell.text())
            .collect();
        println!("{:?}", cells);
    }
}
```

### Filter by Attribute

```rust
let soup = Soup::parse(html);

// Find all images with alt text
for img in soup.find_all("img[alt]")? {
    let src = img.get("src").unwrap_or("");
    let alt = img.get("alt").unwrap_or("");
    println!("{}: {}", src, alt);
}
```

### Extract Nested Data

```rust
let soup = Soup::parse(html);

for article in soup.find_all("article.post")? {
    let title = article
        .find(".title")?
        .map(|t| t.text())
        .unwrap_or_default();

    let author = article
        .find(".author")?
        .map(|t| t.text())
        .unwrap_or_default();

    let date = article
        .find("time")?
        .and_then(|t| t.get("datetime"))
        .unwrap_or("");

    println!("{} by {} on {}", title, author, date);
}
```

## Error Handling

scrape-rs uses `Result` for fallible operations:

### Rust

```rust
use scrape_core::{Soup, Error};

fn extract_title(html: &str) -> Result<String, Error> {
    let soup = Soup::parse(html);
    let title = soup
        .find("title")?
        .ok_or_else(|| Error::not_found("title"))?;
    Ok(title.text())
}
```

### Python

```python
from scrape_rs import Soup, ScrapeError

def extract_title(html):
    try:
        soup = Soup(html)
        title = soup.find("title")
        if not title:
            raise ScrapeError("Title not found")
        return title.text
    except ScrapeError as e:
        print(f"Error: {e}")
        return None
```

### Node.js

```typescript
import { Soup, ScrapeError } from '@scrape-rs/scrape';

function extractTitle(html: string): string | null {
    try {
        const soup = new Soup(html);
        const title = soup.find("title");
        if (!title) {
            throw new Error("Title not found");
        }
        return title.text;
    } catch (error) {
        console.error(`Error: ${error}`);
        return null;
    }
}
```

## Performance Tips

For best performance:

1. **Reuse compiled selectors** when querying many documents:

```rust
use scrape_core::compile_selector;

let selector = compile_selector("div.product")?;

for html in documents {
    let soup = Soup::parse(html);
    let products = soup.find_all_compiled(&selector)?;
    // Process products...
}
```

2. **Use streaming for large files** (requires `streaming` feature):

```rust
use scrape_core::{StreamingSoup, StreamingConfig};

let mut streaming = StreamingSoup::with_config(
    StreamingConfig::default()
);

streaming.on_element("a[href]", |el| {
    println!("Found link: {:?}", el.get_attribute("href"));
    Ok(())
})?;
```

3. **Process in parallel** (requires `parallel` feature):

```rust
use scrape_core::parallel::parse_batch;

let results = parse_batch(&html_documents)?;
```

## Next Steps

Now that you know the basics:

- Read [Core Concepts](concepts.md) for deeper understanding
- Explore the [User Guide](../user-guide/parsing.md) for advanced features
- Check [Migration Guides](../migration/overview.md) if coming from another library
