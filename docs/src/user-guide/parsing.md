# Parsing HTML

This chapter covers different ways to parse HTML documents with scrape-rs.

## Basic Parsing

The simplest way to parse HTML is with `Soup::parse()`:

```rust
use scrape_core::Soup;

let html = "<html><body><h1>Hello</h1></body></html>";
let soup = Soup::parse(html);
```

This uses default configuration and is suitable for most use cases.

## Parsing Configuration

Customize parsing behavior with `SoupConfig`:

```rust
use scrape_core::{Soup, SoupConfig};

let config = SoupConfig::builder()
    .max_depth(256)
    .preserve_whitespace(true)
    .include_comments(true)
    .build();

let soup = Soup::parse_with_config(html, config);
```

### Configuration Options

#### max_depth

Maximum nesting depth for DOM tree. Default: 512

```rust
let config = SoupConfig::builder()
    .max_depth(128)
    .build();
```

Use cases:
- Prevent stack overflow on malicious HTML
- Limit resource usage
- Enforce document structure constraints

#### preserve_whitespace

Whether to keep whitespace-only text nodes. Default: false

```rust
let config = SoupConfig::builder()
    .preserve_whitespace(true)
    .build();
```

When enabled:
```html
<div>
    <span>Text</span>
</div>
```

Preserves the newline and spaces around `<span>`.

When disabled (default), whitespace-only text nodes are removed.

#### include_comments

Whether to include comment nodes in DOM. Default: false

```rust
let config = SoupConfig::builder()
    .include_comments(true)
    .build();
```

Useful for:
- Processing conditional comments
- Extracting metadata from comments
- Preserving comments in modified HTML

## Fragment Parsing

Parse HTML fragments without wrapping in `<html><body>`:

```rust
let soup = Soup::parse_fragment("<span>A</span><span>B</span>");
```

Fragment parsing:
- Does not add `<html>` or `<body>` wrappers
- Parses as if content appeared inside `<body>`
- Useful for processing snippets

### Context Element

Specify parsing context for special elements:

```rust
// Parse table rows without <table> wrapper
let soup = Soup::parse_fragment_with_context("<tr><td>Data</td></tr>", "tbody");
```

Common contexts:
- `"body"` (default): Standard HTML elements
- `"table"`: Allows `<tr>` without `<tbody>`
- `"tbody"`: Allows `<tr>` directly
- `"tr"`: Allows `<td>` directly
- `"select"`: Allows `<option>` directly

## Parsing from File

Read and parse from filesystem:

```rust
use std::path::Path;
use scrape_core::Soup;

let soup = Soup::from_file(Path::new("index.html"))?;
```

For large files, consider streaming instead:

```rust
use scrape_core::{StreamingSoup, StreamingConfig};

let mut streaming = StreamingSoup::new();
// Register handlers...
streaming.parse_file("large.html")?;
```

## Parser Modes

### DOM Parser (Default)

Builds complete document tree in memory:

```rust
let soup = Soup::parse(html);
```

Characteristics:
- Memory usage: O(n) where n = document size
- Allows random access
- Supports tree navigation (parent, siblings)
- Can query multiple times
- Best for documents < 10MB

### Streaming Parser

Processes HTML incrementally with callbacks:

```rust
use scrape_core::StreamingSoup;

let mut streaming = StreamingSoup::new();

streaming.on_element("a[href]", |el| {
    if let Some(href) = el.get_attribute("href") {
        println!("Link: {}", href);
    }
    Ok(())
})?;

streaming.write(html.as_bytes())?;
streaming.end()?;
```

Characteristics:
- Memory usage: O(1) constant
- Sequential processing only
- No tree navigation
- One-pass extraction
- Best for documents > 100MB

Streaming parsing will be covered in Phase 20 Week 2.

## Encoding

scrape-rs expects UTF-8 input. If your HTML uses a different encoding, convert first:

```rust
use encoding_rs::WINDOWS_1252;

let (decoded, _, _) = WINDOWS_1252.decode(bytes);
let soup = Soup::parse(&decoded);
```

For automatic encoding detection:

```rust
use chardet::detect;

let (encoding_name, _confidence) = detect(bytes);
// Use encoding_rs to decode...
```

## Malformed HTML

scrape-rs handles malformed HTML gracefully:

### Unclosed Tags

```html
<div>
    <span>Content
</div>
```

Parser automatically closes `<span>` before closing `<div>`.

### Misnested Tags

```html
<b><i>Text</b></i>
```

Parser restructures to valid nesting:
```html
<b><i>Text</i></b><i></i>
```

### Invalid Attributes

```html
<div class"value">
```

Parser ignores malformed attributes but continues parsing.

### Strict Mode

Enable strict mode to fail on malformed HTML:

```rust
let config = SoupConfig::builder()
    .strict_mode(true)
    .build();

match Soup::parse_with_config(bad_html, config) {
    Ok(soup) => { /* ... */ }
    Err(e) => eprintln!("Parse error: {}", e),
}
```

## Parse Warnings

Access parse warnings from Phase 19:

```rust
use scrape_core::parser::{Html5everParser, Parser};

let parser = Html5everParser;
let result = parser.parse_with_warnings(html)?;

for warning in result.warnings() {
    println!("Warning: {} at line {}", warning.message(), warning.line());
}

let document = result.into_document();
```

Warnings include:
- Unexpected end tag
- Misnested tags
- Invalid attributes
- Encoding issues

## Performance Considerations

### Pre-allocation

For known document size, pre-allocate arena:

```rust
use scrape_core::parser::{Html5everParser, Parser, ParseConfig};

let parser = Html5everParser;
let config = ParseConfig::default();
let estimated_nodes = html.len() / 50;  // Rough estimate

let document = parser.parse_with_config_and_capacity(
    html,
    &config,
    estimated_nodes
)?;
```

Benefits:
- Reduces allocation overhead
- Improves parse speed by ~10-15%
- Useful when parsing many similar documents

### Streaming for Large Documents

For documents over 100MB, use streaming:

```rust
let mut streaming = StreamingSoup::new();
// Process in constant memory
```

## Next Steps

- Learn about [Querying](querying.md) elements
