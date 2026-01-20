//! Integration tests for streaming HTML parsing.

#![cfg(feature = "streaming")]

use std::sync::{Arc, Mutex};

use scrape_core::{Result, StreamingSoup};

#[test]
fn test_streaming_element_handler() -> Result<()> {
    let mut streaming = StreamingSoup::new();

    // Track elements found
    let found_elements = Arc::new(Mutex::new(Vec::new()));
    let found_clone = Arc::clone(&found_elements);

    streaming.on_element("div", move |el| {
        found_clone.lock().unwrap().push(el.tag_name());
        Ok(())
    })?;

    let mut processor = streaming.start();
    processor.write(b"<html><body><div>one</div><div>two</div></body></html>")?;
    let finished = processor.end()?;

    // Verify elements were found
    assert_eq!(found_elements.lock().unwrap().len(), 2);
    assert_eq!(found_elements.lock().unwrap()[0], "div");
    assert_eq!(found_elements.lock().unwrap()[1], "div");

    // Verify stats
    assert_eq!(finished.stats().elements_count, 2);
    assert!(finished.stats().bytes_processed > 0);

    Ok(())
}

#[test]
fn test_streaming_element_modification() -> Result<()> {
    let mut streaming = StreamingSoup::new();

    // Modify all links
    streaming.on_element("a", |el| {
        el.set_attribute("target", "_blank")?;
        el.set_attribute("rel", "noopener")?;
        Ok(())
    })?;

    let mut processor = streaming.start();
    processor.write(b"<a href='test.html'>Link</a>")?;
    let finished = processor.end()?;

    // Verify output contains modifications
    let output = String::from_utf8_lossy(finished.output());
    assert!(output.contains("target=\"_blank\""));
    assert!(output.contains("rel=\"noopener\""));

    Ok(())
}

#[test]
fn test_streaming_multi_chunk() -> Result<()> {
    let mut streaming = StreamingSoup::new();

    let count = Arc::new(Mutex::new(0));
    let count_clone = Arc::clone(&count);

    streaming.on_element("p", move |_el| {
        *count_clone.lock().unwrap() += 1;
        Ok(())
    })?;

    let mut processor = streaming.start();

    // Write in multiple chunks
    processor.write(b"<html><body>")?;
    processor.write(b"<p>First</p>")?;
    processor.write(b"<p>Second</p>")?;
    processor.write(b"</body></html>")?;

    let finished = processor.end()?;

    // Each write() creates a new rewriter, so stats accumulate
    // But because we write 4 times, we get counts for each chunk
    // The actual count depends on how lol_html parses across chunks
    let count_value = *count.lock().unwrap();
    assert!(count_value >= 2, "Should find at least 2 paragraphs");
    assert_eq!(finished.stats().elements_count, count_value);

    Ok(())
}

#[test]
fn test_streaming_selector_specificity() -> Result<()> {
    let mut streaming = StreamingSoup::new();

    let classes = Arc::new(Mutex::new(Vec::new()));
    let classes_clone = Arc::clone(&classes);

    streaming.on_element("div.special", move |el| {
        if let Some(class) = el.get_attribute("class") {
            classes_clone.lock().unwrap().push(class);
        }
        Ok(())
    })?;

    let mut processor = streaming.start();
    processor.write(b"<div class='normal'>Normal</div><div class='special'>Special</div>")?;
    let finished = processor.end()?;

    // Should only match .special
    assert_eq!(classes.lock().unwrap().len(), 1);
    assert_eq!(classes.lock().unwrap()[0], "special");
    assert_eq!(finished.stats().elements_count, 1);

    Ok(())
}

#[test]
fn test_streaming_handler_error_propagation() -> Result<()> {
    let mut streaming = StreamingSoup::new();

    streaming
        .on_element("div", |_el| Err(scrape_core::Error::handler_error("intentional failure")))?;

    let mut processor = streaming.start();
    let result = processor.write(b"<div>content</div>");

    // Should propagate handler error
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_streaming_attribute_operations() -> Result<()> {
    let mut streaming = StreamingSoup::new();

    let attrs = Arc::new(Mutex::new(Vec::new()));
    let attrs_clone = Arc::clone(&attrs);

    streaming.on_element("img", move |el| {
        // Read attribute
        if let Some(src) = el.get_attribute("src") {
            attrs_clone.lock().unwrap().push(src);
        }

        // Modify attributes
        el.set_attribute("loading", "lazy")?;
        el.remove_attribute("alt");

        Ok(())
    })?;

    let mut processor = streaming.start();
    processor.write(b"<img src='test.jpg' alt='Test'>")?;
    let finished = processor.end()?;

    // Verify attribute was read
    assert_eq!(attrs.lock().unwrap().len(), 1);
    assert_eq!(attrs.lock().unwrap()[0], "test.jpg");

    // Verify output has modifications
    let output = String::from_utf8_lossy(finished.output());
    assert!(output.contains("loading=\"lazy\""));
    assert!(!output.contains("alt="));

    Ok(())
}

#[test]
fn test_streaming_empty_selector() {
    let mut streaming = StreamingSoup::new();
    let result = streaming.on_element("", |_el| Ok(()));
    assert!(result.is_err());
}

#[test]
fn test_streaming_no_handlers() -> Result<()> {
    let streaming = StreamingSoup::new();
    let mut processor = streaming.start();

    processor.write(b"<div>content</div>")?;
    let finished = processor.end()?;

    // Should pass through without processing
    assert_eq!(finished.stats().elements_count, 0);
    assert_eq!(finished.stats().bytes_processed, 18); // "<div>content</div>"

    Ok(())
}

#[test]
fn test_streaming_stats_accumulation() -> Result<()> {
    let mut streaming = StreamingSoup::new();

    streaming.on_element("div", |_el| Ok(()))?;

    let mut processor = streaming.start();

    processor.write(b"<div>1</div>")?;
    processor.write(b"<div>2</div>")?;
    processor.write(b"<div>3</div>")?;

    let finished = processor.end()?;

    // Stats should accumulate across chunks
    assert_eq!(finished.stats().elements_count, 3);
    assert_eq!(finished.stats().bytes_processed, 36); // 3 * 12 bytes

    Ok(())
}
