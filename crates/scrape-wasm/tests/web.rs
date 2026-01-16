//! WASM browser tests.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use scrape_wasm::{Soup, SoupConfig, has_simd_support, parse_batch, version};

// ==================== Module Tests ====================

#[wasm_bindgen_test]
fn test_version() {
    let v = version();
    assert!(!v.is_empty());
    assert!(v.starts_with("0."));
}

#[wasm_bindgen_test]
fn test_simd_check() {
    // Verify it doesn't panic and returns a bool
    let _ = has_simd_support();
}

// ==================== SoupConfig Tests ====================

#[wasm_bindgen_test]
fn test_soup_config_default() {
    let config = SoupConfig::new();
    assert_eq!(config.max_depth(), 512);
    assert!(!config.strict_mode());
    assert!(!config.preserve_whitespace());
    assert!(!config.include_comments());
}

#[wasm_bindgen_test]
fn test_soup_config_setters() {
    let mut config = SoupConfig::new();
    config.set_max_depth(256);
    config.set_strict_mode(true);
    config.set_preserve_whitespace(true);
    config.set_include_comments(true);

    assert_eq!(config.max_depth(), 256);
    assert!(config.strict_mode());
    assert!(config.preserve_whitespace());
    assert!(config.include_comments());
}

// ==================== Soup Tests ====================

#[wasm_bindgen_test]
fn test_soup_new() {
    let soup = Soup::new("<html><body>Hello</body></html>", None);
    assert!(soup.root().is_some());
}

#[wasm_bindgen_test]
fn test_soup_with_config() {
    let mut config = SoupConfig::new();
    config.set_max_depth(128);

    let soup = Soup::new("<div>Test</div>", Some(config));
    assert!(soup.root().is_some());
}

#[wasm_bindgen_test]
fn test_soup_find() {
    let soup = Soup::new("<div><span class='item'>Hello</span></div>", None);
    let span = soup.find("span.item").unwrap();

    assert!(span.is_some());
    let span = span.unwrap();
    assert_eq!(span.name(), Some("span".to_string()));
    assert_eq!(span.text(), "Hello");
}

#[wasm_bindgen_test]
fn test_soup_find_not_found() {
    let soup = Soup::new("<div>Hello</div>", None);
    let result = soup.find("span").unwrap();
    assert!(result.is_none());
}

#[wasm_bindgen_test]
fn test_soup_find_invalid_selector() {
    let soup = Soup::new("<div>Hello</div>", None);
    let result = soup.find("[");
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_soup_find_all() {
    let soup = Soup::new("<ul><li>A</li><li>B</li><li>C</li></ul>", None);
    let items = soup.find_all("li").unwrap();
    assert_eq!(items.len(), 3);
}

#[wasm_bindgen_test]
fn test_soup_select() {
    let soup = Soup::new("<div class='a'><span class='b'>Text</span></div>", None);
    let results = soup.select("div.a > span.b").unwrap();
    assert_eq!(results.len(), 1);
}

#[wasm_bindgen_test]
fn test_soup_title() {
    let soup = Soup::new("<html><head><title>Test Title</title></head></html>", None);
    assert_eq!(soup.title(), Some("Test Title".to_string()));
}

#[wasm_bindgen_test]
fn test_soup_title_missing() {
    let soup = Soup::new("<html><body>No title</body></html>", None);
    assert!(soup.title().is_none());
}

#[wasm_bindgen_test]
fn test_soup_text() {
    let soup = Soup::new("<div>Hello <b>World</b>!</div>", None);
    let text = soup.text();
    assert!(text.contains("Hello"));
    assert!(text.contains("World"));
}

#[wasm_bindgen_test]
fn test_soup_to_html() {
    let soup = Soup::new("<div><span>text</span></div>", None);
    let html = soup.to_html();
    assert!(html.contains("<div>"));
    assert!(html.contains("<span>"));
}

#[wasm_bindgen_test]
fn test_soup_length() {
    let soup = Soup::new("<div><span>A</span><span>B</span></div>", None);
    assert!(soup.length() > 0);
}

// ==================== Tag Content Tests ====================

#[wasm_bindgen_test]
fn test_tag_name() {
    let soup = Soup::new("<div>Hello</div>", None);
    let div = soup.find("div").unwrap().unwrap();
    assert_eq!(div.name(), Some("div".to_string()));
}

#[wasm_bindgen_test]
fn test_tag_text() {
    let soup = Soup::new("<div>Hello <span>World</span></div>", None);
    let div = soup.find("div").unwrap().unwrap();
    assert!(div.text().contains("Hello"));
    assert!(div.text().contains("World"));
}

#[wasm_bindgen_test]
fn test_tag_inner_html() {
    let soup = Soup::new("<div><span>Hello</span></div>", None);
    let div = soup.find("div").unwrap().unwrap();
    let inner = div.inner_html();
    assert!(inner.contains("<span>"));
    assert!(!inner.contains("<div>"));
}

#[wasm_bindgen_test]
fn test_tag_outer_html() {
    let soup = Soup::new("<div><span>Hello</span></div>", None);
    let div = soup.find("div").unwrap().unwrap();
    let outer = div.outer_html();
    assert!(outer.contains("<div>"));
    assert!(outer.contains("</div>"));
}

// ==================== Tag Attribute Tests ====================

#[wasm_bindgen_test]
fn test_tag_get_attribute() {
    let soup = Soup::new("<a href='https://example.com'>Link</a>", None);
    let a = soup.find("a").unwrap().unwrap();
    assert_eq!(a.get("href"), Some("https://example.com".to_string()));
}

#[wasm_bindgen_test]
fn test_tag_attr_alias() {
    let soup = Soup::new("<a href='https://example.com'>Link</a>", None);
    let a = soup.find("a").unwrap().unwrap();
    assert_eq!(a.attr("href"), Some("https://example.com".to_string()));
}

#[wasm_bindgen_test]
fn test_tag_has_attr() {
    let soup = Soup::new("<a href='#' class='link'>Link</a>", None);
    let a = soup.find("a").unwrap().unwrap();
    assert!(a.has_attr("href"));
    assert!(a.has_attr("class"));
    assert!(!a.has_attr("id"));
}

#[wasm_bindgen_test]
fn test_tag_has_class() {
    let soup = Soup::new("<div class='foo bar baz'>Test</div>", None);
    let div = soup.find("div").unwrap().unwrap();
    assert!(div.has_class("foo"));
    assert!(div.has_class("bar"));
    assert!(div.has_class("baz"));
    assert!(!div.has_class("qux"));
}

#[wasm_bindgen_test]
fn test_tag_classes() {
    let soup = Soup::new("<div class='foo bar'>Test</div>", None);
    let div = soup.find("div").unwrap().unwrap();
    let classes = div.classes();
    assert_eq!(classes.len(), 2);
    assert!(classes.contains(&"foo".to_string()));
    assert!(classes.contains(&"bar".to_string()));
}

// ==================== Tag Navigation Tests ====================

#[wasm_bindgen_test]
fn test_tag_parent() {
    let soup = Soup::new("<div><span>Hello</span></div>", None);
    let span = soup.find("span").unwrap().unwrap();
    let parent = span.parent().unwrap();
    assert_eq!(parent.name(), Some("div".to_string()));
}

#[wasm_bindgen_test]
fn test_tag_children() {
    let soup = Soup::new("<div><span>A</span><span>B</span><span>C</span></div>", None);
    let div = soup.find("div").unwrap().unwrap();
    let children = div.children();
    assert_eq!(children.len(), 3);
}

#[wasm_bindgen_test]
fn test_tag_next_sibling() {
    let soup = Soup::new("<div><span id='first'>A</span><span id='second'>B</span></div>", None);
    let first = soup.find("#first").unwrap().unwrap();
    let next = first.next_sibling().unwrap();
    assert_eq!(next.get("id"), Some("second".to_string()));
}

#[wasm_bindgen_test]
fn test_tag_prev_sibling() {
    let soup = Soup::new("<div><span id='first'>A</span><span id='second'>B</span></div>", None);
    let second = soup.find("#second").unwrap().unwrap();
    let prev = second.prev_sibling().unwrap();
    assert_eq!(prev.get("id"), Some("first".to_string()));
}

#[wasm_bindgen_test]
fn test_tag_descendants() {
    let soup = Soup::new("<div><ul><li>A</li><li>B</li></ul></div>", None);
    let div = soup.find("div").unwrap().unwrap();
    let descendants = div.descendants();
    // Should include ul and both li elements
    assert!(descendants.len() >= 3);
}

// ==================== Tag Scoped Query Tests ====================

#[wasm_bindgen_test]
fn test_tag_scoped_find() {
    let soup =
        Soup::new("<div class='outer'><div class='inner'><span>Target</span></div></div>", None);
    let inner = soup.find(".inner").unwrap().unwrap();
    let span = inner.find("span").unwrap().unwrap();
    assert_eq!(span.text(), "Target");
}

#[wasm_bindgen_test]
fn test_tag_scoped_find_all() {
    let soup =
        Soup::new("<div class='container'><span>A</span><span>B</span></div><span>C</span>", None);
    let container = soup.find(".container").unwrap().unwrap();
    let spans = container.find_all("span").unwrap();
    // Should only find spans inside container
    assert_eq!(spans.len(), 2);
}

#[wasm_bindgen_test]
fn test_tag_length() {
    let soup = Soup::new("<div><span>A</span><span>B</span><span>C</span></div>", None);
    let div = soup.find("div").unwrap().unwrap();
    assert_eq!(div.length(), 3);
}

// ==================== Batch Processing Tests ====================

#[wasm_bindgen_test]
fn test_parse_batch() {
    let docs =
        vec!["<div>A</div>".to_string(), "<div>B</div>".to_string(), "<div>C</div>".to_string()];
    let soups = parse_batch(docs);
    assert_eq!(soups.len(), 3);
}

#[wasm_bindgen_test]
fn test_parse_batch_empty() {
    let docs: Vec<String> = vec![];
    let soups = parse_batch(docs);
    assert_eq!(soups.len(), 0);
}

// ==================== Edge Cases ====================

#[wasm_bindgen_test]
fn test_empty_html() {
    let soup = Soup::new("", None);
    assert!(soup.find("div").unwrap().is_none());
}

#[wasm_bindgen_test]
fn test_malformed_html() {
    // Should handle gracefully
    let soup = Soup::new("<div><span>Unclosed", None);
    assert!(soup.root().is_some());
}

#[wasm_bindgen_test]
fn test_attribute_escaping() {
    let soup = Soup::new("<div data-value='a&quot;b'>Test</div>", None);
    let div = soup.find("div").unwrap().unwrap();
    // Attribute should be properly unescaped when parsed
    let value = div.get("data-value");
    assert!(value.is_some());
}

#[wasm_bindgen_test]
fn test_text_content_escaping() {
    let soup = Soup::new("<div>&lt;script&gt;alert('xss')&lt;/script&gt;</div>", None);
    let div = soup.find("div").unwrap().unwrap();
    let text = div.text();
    // Text should contain the unescaped content
    assert!(text.contains("<script>") || text.contains("&lt;script&gt;"));
}

#[wasm_bindgen_test]
fn test_void_elements() {
    let soup = Soup::new("<div><br><hr><img src='test.png'></div>", None);
    let div = soup.find("div").unwrap().unwrap();
    let html = div.inner_html();
    assert!(html.contains("<br>"));
    assert!(html.contains("<hr>"));
    assert!(html.contains("<img"));
    // Void elements should not have closing tags
    assert!(!html.contains("</br>"));
    assert!(!html.contains("</hr>"));
}

#[wasm_bindgen_test]
fn test_nested_selectors() {
    let soup = Soup::new(
        "<div id='a'><div id='b'><div id='c'><span>Target</span></div></div></div>",
        None,
    );
    let span = soup.find("#a #b #c span").unwrap().unwrap();
    assert_eq!(span.text(), "Target");
}

#[wasm_bindgen_test]
fn test_multiple_classes_selector() {
    let soup = Soup::new("<div class='foo bar baz'>Content</div>", None);
    let div = soup.find("div.foo.bar.baz").unwrap().unwrap();
    assert_eq!(div.text(), "Content");
}

#[wasm_bindgen_test]
fn test_attribute_selector() {
    let soup =
        Soup::new("<input type='text' name='email'><input type='password' name='pwd'>", None);
    let email = soup.find("input[type='text']").unwrap().unwrap();
    assert_eq!(email.get("name"), Some("email".to_string()));
}
