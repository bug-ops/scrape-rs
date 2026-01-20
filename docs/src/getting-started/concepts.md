# Core Concepts

This chapter explains the fundamental concepts behind scrape-rs.

## Document Object Model (DOM)

When you parse HTML, scrape-rs builds a Document Object Model (DOM) - a tree representation of the HTML structure.

```html
<html>
  <body>
    <div class="container">
      <h1>Title</h1>
      <p>Paragraph</p>
    </div>
  </body>
</html>
```

This becomes:

```
Document
└── html
    └── body
        └── div.container
            ├── h1 ("Title")
            └── p ("Paragraph")
```

### Arena Allocation

scrape-rs uses arena allocation for DOM nodes, which provides:

- **Fast allocation**: All nodes allocated in contiguous memory
- **No reference counting**: Zero overhead compared to `Rc<RefCell<T>>`
- **Cache-friendly**: Better CPU cache utilization
- **Safe lifetimes**: Rust's borrow checker prevents dangling references

### Node Types

The DOM contains three types of nodes:

1. **Element nodes**: HTML tags (`<div>`, `<span>`, etc.)
2. **Text nodes**: Raw text content
3. **Comment nodes**: HTML comments (if `include_comments` is enabled)

## Parsing Modes

scrape-rs offers two parsing approaches:

### DOM Parser

The default parser builds the entire document tree in memory:

```rust
let soup = Soup::parse(html);
```

Best for:
- Documents under 10MB
- Random access to elements
- Tree navigation (parent, siblings)
- Multiple queries on the same document

Memory usage: O(n) where n is document size

### Streaming Parser

The streaming parser processes HTML incrementally:

```rust
let mut streaming = StreamingSoup::new();
streaming.on_element("a", |el| {
    println!("Link: {:?}", el.get_attribute("href"));
    Ok(())
})?;
```

Best for:
- Large documents (100MB+)
- Sequential processing
- One-pass extraction
- Memory-constrained environments

Memory usage: O(1) constant memory

## CSS Selectors

CSS selectors are patterns for matching elements:

### Basic Selectors

```rust
// Type selector - matches tag name
soup.find("div")?

// Class selector - matches class attribute
soup.find(".product")?

// ID selector - matches id attribute
soup.find("#header")?

// Universal selector - matches all elements
soup.find("*")?
```

### Attribute Selectors

```rust
// Has attribute
soup.find("[href]")?

// Exact match
soup.find("[type='text']")?

// Contains word
soup.find("[class~='active']")?

// Starts with
soup.find("[href^='https://']")?

// Ends with
soup.find("[src$='.png']")?

// Contains substring
soup.find("[href*='example']")?
```

### Combinators

```rust
// Descendant - any level deep
soup.find("div span")?

// Child - direct children only
soup.find("ul > li")?

// Adjacent sibling - immediately following
soup.find("h1 + p")?

// General sibling - any following sibling
soup.find("h1 ~ p")?
```

### Pseudo-classes

```rust
// First/last child
soup.find("li:first-child")?
soup.find("li:last-child")?

// Nth child
soup.find("li:nth-child(2)")?      // Second child
soup.find("li:nth-child(2n)")?     // Even children
soup.find("li:nth-child(2n+1)")?   // Odd children

// Empty elements
soup.find("div:empty")?

// Negation
soup.find("input:not([type='hidden'])")?
```

### Selector Performance

Different selectors have different performance characteristics:

| Selector | Complexity | Notes |
|----------|------------|-------|
| `#id` | O(1) | Uses ID index |
| `.class` | O(n) | Linear scan with early exit |
| `tag` | O(n) | Linear scan |
| `[attr]` | O(n) | Linear scan |
| `div > span` | O(n) | Depends on tree depth |
| `div span` | O(n²) | Checks all descendants |

Optimization tips:
- Start with ID selector when possible: `#container .item` not `.item`
- Use child combinator (`>`) instead of descendant when appropriate
- Compile selectors for reuse

## Compiled Selectors

For repeated queries, compile the selector once:

```rust
use scrape_core::compile_selector;

// Compile once
let selector = compile_selector("div.product")?;

// Reuse many times
for html in documents {
    let soup = Soup::parse(html);
    let products = soup.find_all_compiled(&selector)?;
    // Process...
}
```

Performance benefit: ~50% faster for complex selectors

## Element References

Elements are represented by the `Tag` type:

```rust
let tag = soup.find("div")?.unwrap();
```

### Lifetime Relationship

`Tag` borrows from the `Soup`:

```rust
let soup = Soup::parse(html);
let tag = soup.find("div")?.unwrap();  // Borrows from soup
// soup cannot be modified or dropped while tag is in scope
```

This prevents dangling references at compile time.

### Copy Semantics

`Tag` implements `Copy`, so it can be duplicated cheaply:

```rust
let tag1 = soup.find("div")?.unwrap();
let tag2 = tag1;  // Copies, both valid
```

The copy is just a reference (pointer + ID), not the actual element data.

## Text Extraction

Text can be extracted in different ways:

### Direct Text

```rust
// Just the text of this element's direct text nodes
let text = tag.text();
```

### Deep Text

```rust
// Text of this element and all descendants
let all_text = tag.text();  // Default behavior
```

### Normalized Text

Text extraction automatically:
- Collapses multiple spaces into one
- Trims leading/trailing whitespace
- Converts newlines to spaces

To preserve whitespace, use configuration:

```rust
let config = SoupConfig::builder()
    .preserve_whitespace(true)
    .build();
let soup = Soup::parse_with_config(html, config);
```

## Error Handling

scrape-rs uses explicit error types:

### QueryError

Returned when a CSS selector is invalid:

```rust
match soup.find("div[") {
    Err(Error::InvalidSelector { selector }) => {
        println!("Bad selector: {}", selector);
    }
    _ => {}
}
```

### NotFound

Not an error - use `Option`:

```rust
// Returns Ok(Some(tag)) or Ok(None), not Err
match soup.find(".missing")? {
    Some(tag) => println!("Found: {}", tag.text()),
    None => println!("Not found"),
}
```

## Memory Efficiency

scrape-rs minimizes memory usage through:

### String Interning

Tag names and attribute names are interned:

```rust
// Many <div> elements share the same "div" string
let divs = soup.find_all("div")?;
// Memory: O(1) for tag names, not O(n)
```

### Compact Node Representation

Nodes use space-efficient layouts:

- Element: 64 bytes
- Text: 40 bytes
- Comment: 40 bytes

For comparison, BeautifulSoup's Python objects use 200-400 bytes per node.

### Zero-copy Text

Text content is stored as references into the original HTML string when possible, avoiding duplication.

## Thread Safety

### DOM Types

`Soup` and `Tag` are `!Send` and `!Sync` by design:

```rust
// This won't compile:
let soup = Soup::parse(html);
std::thread::spawn(move || {
    soup.find("div");  // Error: Soup is not Send
});
```

Rationale: DOM uses interior mutability for caching, unsafe to share.

### Parallel Processing

Use the parallel module for multi-threaded parsing:

```rust
use scrape_core::parallel::parse_batch;

// Parses documents in parallel
let results = parse_batch(&documents)?;
```

Each document gets its own thread-local DOM.

## Configuration Options

Customize parsing behavior:

```rust
use scrape_core::SoupConfig;

let config = SoupConfig::builder()
    .max_depth(256)              // Limit nesting depth
    .strict_mode(true)           // Fail on malformed HTML
    .preserve_whitespace(true)   // Keep whitespace-only text nodes
    .include_comments(true)      // Include comment nodes
    .build();

let soup = Soup::parse_with_config(html, config);
```

### max_depth

Limits DOM tree depth to prevent stack overflow on deeply nested HTML.

Default: 512

### strict_mode

When enabled, parsing fails on malformed HTML instead of attempting recovery.

Default: false (forgiving mode)

### preserve_whitespace

Keeps text nodes that contain only whitespace.

Default: false (whitespace-only nodes removed)

### include_comments

Includes HTML comments in the DOM tree.

Default: false (comments ignored)

## Next Steps

Now that you understand the core concepts:

- Learn about [Parsing](../user-guide/parsing.md) options
- Explore [Querying](../user-guide/querying.md) techniques
