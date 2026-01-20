# Querying Elements

This chapter covers finding and selecting elements using CSS selectors.

## Finding Elements

### find() - First Match

Find the first element matching a selector:

```rust
use scrape_core::Soup;

let soup = Soup::parse(html);

// Returns Ok(Some(tag)) if found, Ok(None) if not found
match soup.find("div.product")? {
    Some(tag) => println!("Found: {}", tag.text()),
    None => println!("Not found"),
}
```

### find_all() - All Matches

Find all elements matching a selector:

```rust
let tags = soup.find_all("div.product")?;

for tag in tags {
    println!("Product: {}", tag.text());
}
```

## CSS Selector Syntax

### Basic Selectors

```rust
// Type selector - matches tag name
soup.find("div")?

// Class selector
soup.find(".product")?

// ID selector
soup.find("#header")?

// Multiple classes
soup.find(".product.featured")?

// Compound selector
soup.find("div.product")?
```

### Attribute Selectors

```rust
// Has attribute
soup.find("[href]")?

// Exact value
soup.find("[type='text']")?

// Contains word
soup.find("[class~='active']")?

// Starts with
soup.find("[href^='https://']")?

// Ends with
soup.find("[src$='.png']")?

// Contains substring
soup.find("[href*='example']")?

// Case-insensitive
soup.find("[type='TEXT' i]")?
```

### Combinators

```rust
// Descendant - any level
soup.find("div span")?

// Child - direct children only
soup.find("ul > li")?

// Adjacent sibling - next element
soup.find("h1 + p")?

// General sibling - following elements
soup.find("h1 ~ p")?
```

### Pseudo-classes

```rust
// First/last child
soup.find("li:first-child")?
soup.find("li:last-child")?

// Nth child
soup.find("li:nth-child(2)")?       // Second
soup.find("li:nth-child(2n)")?      // Even
soup.find("li:nth-child(2n+1)")?    // Odd
soup.find("li:nth-child(odd)")?     // Odd (shorthand)
soup.find("li:nth-child(even)")?    // Even (shorthand)

// Empty elements
soup.find("div:empty")?

// Negation
soup.find("input:not([type='hidden'])")?
```

## Compiled Selectors

For repeated queries, compile the selector once:

```rust
use scrape_core::compile_selector;

let selector = compile_selector("div.product")?;

// Reuse for multiple documents
for html in documents {
    let soup = Soup::parse(html);
    let products = soup.find_all_compiled(&selector)?;
    // Process products...
}
```

Performance improvement: ~50% faster for complex selectors

## Selector Explanation

Use `explain()` to understand selector performance:

```rust
use scrape_core::explain;

let explanation = explain("div.product > span.price")?;

println!("Specificity: {:?}", explanation.specificity());
println!("Optimization hints: {:?}", explanation.hints());
```

With document context:

```rust
use scrape_core::explain_with_document;

let soup = Soup::parse(html);
let explanation = explain_with_document("div.product", soup.document())?;

println!("Matches: {}", explanation.match_count());
println!("Estimated cost: {}", explanation.estimated_cost());
```

## Scoped Queries

Query within a specific element:

```rust
let container = soup.find("#products")?.unwrap();

// Find within container only
let products = container.find_all(".product")?;

for product in products {
    let name = product.find(".name")?.unwrap().text();
    println!("Product: {}", name);
}
```

## Error Handling

```rust
use scrape_core::Error;

match soup.find("div[invalid") {
    Err(Error::InvalidSelector { selector }) => {
        eprintln!("Bad selector: {}", selector);
    }
    Ok(Some(tag)) => {
        // Process tag
    }
    Ok(None) => {
        // Not found
    }
}
```

## Performance Tips

1. Use ID selectors when possible (O(1) lookup)
2. Prefer child combinator (`>`) over descendant
3. Compile selectors for reuse
4. Use `find()` instead of `find_all()` when only one result needed

## Next Steps

- Read more in the [Parsing](parsing.md) guide
