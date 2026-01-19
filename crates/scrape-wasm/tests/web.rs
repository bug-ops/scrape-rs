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

// ==================== Phase 12: Parents and Ancestors ====================

#[wasm_bindgen_test]
fn test_tag_parents() {
    let html = "<html><body><div><span><a>link</a></span></div></body></html>";
    let soup = Soup::new(html, None);
    let link = soup.find("a").unwrap().unwrap();

    let parents = link.parents();
    assert_eq!(parents.len(), 4); // span, div, body, html
    assert_eq!(parents[0].name(), Some("span".to_string()));
    assert_eq!(parents[1].name(), Some("div".to_string()));
    assert_eq!(parents[2].name(), Some("body".to_string()));
    assert_eq!(parents[3].name(), Some("html".to_string()));
}

#[wasm_bindgen_test]
fn test_tag_ancestors() {
    let html = "<html><body><div><span>text</span></div></body></html>";
    let soup = Soup::new(html, None);
    let span = soup.find("span").unwrap().unwrap();

    let parents = span.parents();
    let ancestors = span.ancestors();

    assert_eq!(parents.len(), ancestors.len());
    for i in 0..parents.len() {
        assert_eq!(parents[i].name(), ancestors[i].name());
    }
}

#[wasm_bindgen_test]
fn test_tag_parents_empty_for_root() {
    let html = "<html><body><div>text</div></body></html>";
    let soup = Soup::new(html, None);
    let root = soup.root().unwrap();

    assert_eq!(root.parents().len(), 0);
}

// ==================== Phase 12: Closest ====================

#[wasm_bindgen_test]
fn test_tag_closest() {
    let html = "<div class='outer'><div class='middle'><span>text</span></div></div>";
    let soup = Soup::new(html, None);
    let span = soup.find("span").unwrap().unwrap();

    let result = span.closest(".outer").unwrap();
    assert!(result.is_some());
    let closest = result.unwrap();
    assert_eq!(closest.get("class"), Some("outer".to_string()));
}

#[wasm_bindgen_test]
fn test_tag_closest_finds_nearest() {
    let html = "<div class='target'><div class='target'><span>text</span></div></div>";
    let soup = Soup::new(html, None);
    let span = soup.find("span").unwrap().unwrap();

    let result = span.closest(".target").unwrap();
    assert!(result.is_some());
    let closest = result.unwrap();
    // Should be the inner div (nearest)
    let parent = span.parent().unwrap();
    assert_eq!(closest.outer_html(), parent.outer_html());
}

#[wasm_bindgen_test]
fn test_tag_closest_not_found() {
    let html = "<div><span>text</span></div>";
    let soup = Soup::new(html, None);
    let span = soup.find("span").unwrap().unwrap();

    let result = span.closest(".nonexistent").unwrap();
    assert!(result.is_none());
}

#[wasm_bindgen_test]
fn test_tag_closest_invalid_selector() {
    let html = "<div><span>text</span></div>";
    let soup = Soup::new(html, None);
    let span = soup.find("span").unwrap().unwrap();

    let result = span.closest("[[[invalid");
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_tag_closest_excludes_self() {
    let html = "<div class='target'><span class='target'>text</span></div>";
    let soup = Soup::new(html, None);
    let span = soup.find("span").unwrap().unwrap();

    let result = span.closest(".target").unwrap();
    assert!(result.is_some());
    let closest = result.unwrap();
    assert_eq!(closest.name(), Some("div".to_string())); // Parent, not self
}

// ==================== Phase 12: NextSiblings ====================

#[wasm_bindgen_test]
fn test_tag_next_siblings() {
    let html = "<div><span id='a'>A</span><span id='b'>B</span><span id='c'>C</span></div>";
    let soup = Soup::new(html, None);
    let first = soup.find("#a").unwrap().unwrap();

    let siblings = first.next_siblings();
    assert_eq!(siblings.len(), 2);
    assert_eq!(siblings[0].get("id"), Some("b".to_string()));
    assert_eq!(siblings[1].get("id"), Some("c".to_string()));
}

#[wasm_bindgen_test]
fn test_tag_next_siblings_empty_for_last() {
    let html = "<div><span id='a'>A</span><span id='b'>B</span></div>";
    let soup = Soup::new(html, None);
    let last = soup.find("#b").unwrap().unwrap();

    assert_eq!(last.next_siblings().len(), 0);
}

#[wasm_bindgen_test]
fn test_tag_next_siblings_skips_text() {
    let html = "<div><span id='a'>A</span>text<span id='b'>B</span></div>";
    let soup = Soup::new(html, None);
    let first = soup.find("#a").unwrap().unwrap();

    let siblings = first.next_siblings();
    assert_eq!(siblings.len(), 1);
    assert_eq!(siblings[0].get("id"), Some("b".to_string()));
}

// ==================== Phase 12: PrevSiblings ====================

#[wasm_bindgen_test]
fn test_tag_prev_siblings() {
    let html = "<div><span id='a'>A</span><span id='b'>B</span><span id='c'>C</span></div>";
    let soup = Soup::new(html, None);
    let last = soup.find("#c").unwrap().unwrap();

    let siblings = last.prev_siblings();
    assert_eq!(siblings.len(), 2);
    // Note: prevSiblings returns in reverse order
    assert_eq!(siblings[0].get("id"), Some("b".to_string()));
    assert_eq!(siblings[1].get("id"), Some("a".to_string()));
}

#[wasm_bindgen_test]
fn test_tag_prev_siblings_empty_for_first() {
    let html = "<div><span id='a'>A</span><span id='b'>B</span></div>";
    let soup = Soup::new(html, None);
    let first = soup.find("#a").unwrap().unwrap();

    assert_eq!(first.prev_siblings().len(), 0);
}

// ==================== Phase 12: Siblings ====================

#[wasm_bindgen_test]
fn test_tag_siblings() {
    let html = "<div><span id='a'>A</span><span id='b'>B</span><span id='c'>C</span></div>";
    let soup = Soup::new(html, None);
    let middle = soup.find("#b").unwrap().unwrap();

    let siblings = middle.siblings();
    assert_eq!(siblings.len(), 2);
    assert_eq!(siblings[0].get("id"), Some("a".to_string()));
    assert_eq!(siblings[1].get("id"), Some("c".to_string()));
}

#[wasm_bindgen_test]
fn test_tag_siblings_empty_for_only_child() {
    let html = "<div><span id='only'>text</span></div>";
    let soup = Soup::new(html, None);
    let only = soup.find("#only").unwrap().unwrap();

    assert_eq!(only.siblings().len(), 0);
}

#[wasm_bindgen_test]
fn test_tag_siblings_skips_text() {
    let html = "<div>text1<span id='a'>A</span>text2<span id='b'>B</span>text3</div>";
    let soup = Soup::new(html, None);
    let first = soup.find("#a").unwrap().unwrap();

    let siblings = first.siblings();
    assert_eq!(siblings.len(), 1);
    assert_eq!(siblings[0].get("id"), Some("b".to_string()));
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

// ==================== Phase 13b: Fragment Parsing Tests ====================

#[wasm_bindgen_test]
fn test_parse_fragment_empty() {
    let soup = Soup::parse_fragment("", None, None);
    assert_eq!(soup.length(), 0);
}

#[wasm_bindgen_test]
fn test_parse_fragment_text_only() {
    let soup = Soup::parse_fragment("Hello World", None, None);
    assert!(soup.text().contains("Hello World"));
}

#[wasm_bindgen_test]
fn test_parse_fragment_simple() {
    let soup = Soup::parse_fragment("<div>Test</div>", None, None);
    let div = soup.find("div").unwrap().unwrap();
    assert_eq!(div.text(), "Test");
}

#[wasm_bindgen_test]
fn test_parse_fragment_nested() {
    let html = "<div><span>A</span><span>B</span></div>";
    let soup = Soup::parse_fragment(html, None, None);
    let spans = soup.find_all("span").unwrap();
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].text(), "A");
    assert_eq!(spans[1].text(), "B");
}

#[wasm_bindgen_test]
fn test_parse_fragment_no_wrapper() {
    let soup = Soup::parse_fragment("<div>Content</div>", None, None);
    let html = soup.to_html();
    assert!(html.contains("<div>"));
    assert!(soup.root().is_some());
}

#[wasm_bindgen_test]
fn test_parse_fragment_with_context_body() {
    let soup = Soup::parse_fragment("<div>Test</div>", Some("body".to_string()), None);
    let div = soup.find("div").unwrap().unwrap();
    assert_eq!(div.text(), "Test");
}

#[wasm_bindgen_test]
fn test_parse_fragment_with_context_table() {
    let html = "<tr><td>Cell 1</td><td>Cell 2</td></tr>";
    let soup = Soup::parse_fragment(html, Some("table".to_string()), None);
    let tds = soup.find_all("td").unwrap();
    assert_eq!(tds.len(), 2);
    assert_eq!(tds[0].text(), "Cell 1");
    assert_eq!(tds[1].text(), "Cell 2");
}

#[wasm_bindgen_test]
fn test_parse_fragment_with_context_tbody() {
    let html = "<tr><td>A</td></tr><tr><td>B</td></tr>";
    let soup = Soup::parse_fragment(html, Some("tbody".to_string()), None);
    let trs = soup.find_all("tr").unwrap();
    assert_eq!(trs.len(), 2);
}

#[wasm_bindgen_test]
fn test_parse_fragment_multiple_roots() {
    let html = "<div>First</div><div>Second</div><div>Third</div>";
    let soup = Soup::parse_fragment(html, None, None);
    let divs = soup.find_all("div").unwrap();
    assert_eq!(divs.len(), 3);
}

#[wasm_bindgen_test]
fn test_parse_fragment_preserves_attributes() {
    let html = "<span class='highlight' id='item'>Text</span>";
    let soup = Soup::parse_fragment(html, None, None);
    let span = soup.find("span").unwrap().unwrap();
    assert_eq!(span.get("class"), Some("highlight".to_string()));
    assert_eq!(span.get("id"), Some("item".to_string()));
}

// ==================== Phase 13b: CompiledSelector Tests ====================

#[wasm_bindgen_test]
fn test_compiled_selector_compile() {
    let selector = scrape_wasm::CompiledSelector::compile("div.item").unwrap();
    assert_eq!(selector.source(), "div.item");
}

#[wasm_bindgen_test]
fn test_compiled_selector_complex() {
    let selector = scrape_wasm::CompiledSelector::compile("ul > li.active:first-child").unwrap();
    assert_eq!(selector.source(), "ul > li.active:first-child");
}

#[wasm_bindgen_test]
fn test_compiled_selector_invalid() {
    let result = scrape_wasm::CompiledSelector::compile("[[[invalid");
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_find_compiled() {
    let html = "<div class='item'>First</div><div class='item'>Second</div>";
    let soup = Soup::new(html, None);
    let selector = scrape_wasm::CompiledSelector::compile(".item").unwrap();

    let result = soup.find_compiled(&selector);
    assert!(result.is_some());
    let tag = result.unwrap();
    assert_eq!(tag.text(), "First");
}

#[wasm_bindgen_test]
fn test_select_compiled() {
    let html = "<ul><li class='active'>A</li><li>B</li><li class='active'>C</li></ul>";
    let soup = Soup::new(html, None);
    let selector = scrape_wasm::CompiledSelector::compile("li.active").unwrap();

    let results = soup.select_compiled(&selector);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].text(), "A");
    assert_eq!(results[1].text(), "C");
}

#[wasm_bindgen_test]
fn test_compiled_selector_reuse() {
    let selector = scrape_wasm::CompiledSelector::compile("span.highlight").unwrap();

    let soup1 = Soup::new("<div><span class='highlight'>Doc 1</span></div>", None);
    let soup2 = Soup::new("<div><span class='highlight'>Doc 2</span></div>", None);

    let result1 = soup1.find_compiled(&selector).unwrap();
    let result2 = soup2.find_compiled(&selector).unwrap();

    assert_eq!(result1.text(), "Doc 1");
    assert_eq!(result2.text(), "Doc 2");
}

#[wasm_bindgen_test]
fn test_compiled_selector_not_found() {
    let soup = Soup::new("<div>No match</div>", None);
    let selector = scrape_wasm::CompiledSelector::compile(".nonexistent").unwrap();

    let result = soup.find_compiled(&selector);
    assert!(result.is_none());
}

#[wasm_bindgen_test]
fn test_compiled_selector_empty_results() {
    let soup = Soup::new("<div>No match</div>", None);
    let selector = scrape_wasm::CompiledSelector::compile("li").unwrap();

    let results = soup.select_compiled(&selector);
    assert_eq!(results.len(), 0);
}

// ==================== Phase 13b: Extraction Methods Tests ====================

#[wasm_bindgen_test]
fn test_select_text_single() {
    let html = "<div><span class='item'>Hello</span></div>";
    let soup = Soup::new(html, None);

    let texts = soup.select_text(".item").unwrap();
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0], "Hello");
}

#[wasm_bindgen_test]
fn test_select_text_multiple() {
    let html = "<ul><li class='item'>First</li><li class='item'>Second</li><li \
                class='item'>Third</li></ul>";
    let soup = Soup::new(html, None);

    let texts = soup.select_text(".item").unwrap();
    assert_eq!(texts.len(), 3);
    assert_eq!(texts[0], "First");
    assert_eq!(texts[1], "Second");
    assert_eq!(texts[2], "Third");
}

#[wasm_bindgen_test]
fn test_select_text_empty() {
    let soup = Soup::new("<div>No matches</div>", None);
    let texts = soup.select_text(".nonexistent").unwrap();
    assert_eq!(texts.len(), 0);
}

#[wasm_bindgen_test]
fn test_select_text_nested_content() {
    let html = "<div class='item'>Hello <b>World</b>!</div>";
    let soup = Soup::new(html, None);

    let texts = soup.select_text(".item").unwrap();
    assert_eq!(texts.len(), 1);
    assert!(texts[0].contains("Hello"));
    assert!(texts[0].contains("World"));
}

#[wasm_bindgen_test]
fn test_select_text_invalid_selector() {
    let soup = Soup::new("<div>Test</div>", None);
    let result = soup.select_text("[[[");
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_select_attr_single() {
    let html = "<a href='/link' class='link'>Click</a>";
    let soup = Soup::new(html, None);

    let hrefs = soup.select_attr("a", "href").unwrap();
    assert_eq!(hrefs.len(), 1);
    assert_eq!(hrefs[0], "/link");
}

#[wasm_bindgen_test]
fn test_select_attr_multiple() {
    let html = "<div><a href='/page1'>L1</a><a href='/page2'>L2</a><a href='/page3'>L3</a></div>";
    let soup = Soup::new(html, None);

    let hrefs = soup.select_attr("a", "href").unwrap();
    assert_eq!(hrefs.len(), 3);
    assert_eq!(hrefs[0], "/page1");
    assert_eq!(hrefs[1], "/page2");
    assert_eq!(hrefs[2], "/page3");
}

#[wasm_bindgen_test]
fn test_select_attr_missing() {
    let html = "<div><a href='/link'>Has</a><a>Missing</a></div>";
    let soup = Soup::new(html, None);

    let hrefs = soup.select_attr("a", "href").unwrap();
    assert_eq!(hrefs.len(), 2);
    assert_eq!(hrefs[0], "/link");
    assert!(hrefs[1].is_undefined());
}

#[wasm_bindgen_test]
fn test_select_attr_empty() {
    let soup = Soup::new("<div>No links</div>", None);
    let hrefs = soup.select_attr("a", "href").unwrap();
    assert_eq!(hrefs.len(), 0);
}

#[wasm_bindgen_test]
fn test_select_attr_invalid_selector() {
    let soup = Soup::new("<div>Test</div>", None);
    let result = soup.select_attr("[[[", "id");
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_select_attr_class() {
    let html = "<div><span class='tag-a'>A</span><span class='tag-b'>B</span><span \
                class='tag-c'>C</span></div>";
    let soup = Soup::new(html, None);

    let classes = soup.select_attr("span", "class").unwrap();
    assert_eq!(classes.len(), 3);
}

// ==================== Phase 13b: Text Nodes Iterator Tests ====================

#[wasm_bindgen_test]
fn test_text_nodes_simple() {
    let html = "<div>Hello World</div>";
    let soup = Soup::new(html, None);
    let div = soup.find("div").unwrap().unwrap();

    let text_nodes = div.text_nodes();
    assert_eq!(text_nodes.len(), 1);
    assert_eq!(text_nodes[0], "Hello World");
}

#[wasm_bindgen_test]
fn test_text_nodes_multiple() {
    let html = "<div>First<span>Middle</span>Last</div>";
    let soup = Soup::new(html, None);
    let div = soup.find("div").unwrap().unwrap();

    let text_nodes = div.text_nodes();
    let has_first = text_nodes.iter().any(|t| t.contains("First"));
    let has_last = text_nodes.iter().any(|t| t.contains("Last"));
    let has_middle = text_nodes.iter().any(|t| t.contains("Middle"));

    assert!(has_first);
    assert!(has_last);
    assert!(!has_middle); // Middle is inside span
}

#[wasm_bindgen_test]
fn test_text_nodes_empty() {
    let html = "<div><span>No direct text</span></div>";
    let soup = Soup::new(html, None);
    let div = soup.find("div").unwrap().unwrap();

    let text_nodes = div.text_nodes();
    let non_empty_count = text_nodes.iter().filter(|t| !t.trim().is_empty()).count();
    assert_eq!(non_empty_count, 0);
}

#[wasm_bindgen_test]
fn test_text_nodes_whitespace() {
    let html = "<div>  Text with spaces  </div>";
    let soup = Soup::new(html, None);
    let div = soup.find("div").unwrap().unwrap();

    let text_nodes = div.text_nodes();
    assert!(text_nodes.len() >= 1);
    assert!(text_nodes.iter().any(|node| node.contains("Text with spaces")));
}

#[wasm_bindgen_test]
fn test_text_nodes_mixed_content() {
    let html = "<p>Start <b>bold</b> middle <i>italic</i> end</p>";
    let soup = Soup::new(html, None);
    let p = soup.find("p").unwrap().unwrap();

    let text_nodes = p.text_nodes();
    assert!(text_nodes.iter().any(|node| node.contains("Start")));
    assert!(text_nodes.iter().any(|node| node.contains("middle")));
    assert!(text_nodes.iter().any(|node| node.contains("end")));
}

// ==================== Phase 13b: Filtered Iterators Tests ====================

#[wasm_bindgen_test]
fn test_children_by_name_single() {
    let html = "<div><span>A</span><p>B</p><span>C</span></div>";
    let soup = Soup::new(html, None);
    let div = soup.find("div").unwrap().unwrap();

    let spans = div.children_by_name("span");
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].text(), "A");
    assert_eq!(spans[1].text(), "C");
}

#[wasm_bindgen_test]
fn test_children_by_name_all_match() {
    let html = "<ul><li>A</li><li>B</li><li>C</li></ul>";
    let soup = Soup::new(html, None);
    let ul = soup.find("ul").unwrap().unwrap();

    let items = ul.children_by_name("li");
    assert_eq!(items.len(), 3);
}

#[wasm_bindgen_test]
fn test_children_by_name_none_match() {
    let html = "<div><span>A</span><span>B</span></div>";
    let soup = Soup::new(html, None);
    let div = soup.find("div").unwrap().unwrap();

    let items = div.children_by_name("li");
    assert_eq!(items.len(), 0);
}

#[wasm_bindgen_test]
fn test_children_by_name_empty() {
    let html = "<div></div>";
    let soup = Soup::new(html, None);
    let div = soup.find("div").unwrap().unwrap();

    let items = div.children_by_name("span");
    assert_eq!(items.len(), 0);
}

#[wasm_bindgen_test]
fn test_children_by_name_case_normalized() {
    let html = "<div><SPAN>A</SPAN><span>B</span></div>";
    let soup = Soup::new(html, None);
    let div = soup.find("div").unwrap().unwrap();

    let items = div.children_by_name("span");
    assert_eq!(items.len(), 2);
}

#[wasm_bindgen_test]
fn test_children_by_class_single() {
    let html = "<div><span class='item'>A</span><span>B</span></div>";
    let soup = Soup::new(html, None);
    let div = soup.find("div").unwrap().unwrap();

    let items = div.children_by_class("item");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].text(), "A");
}

#[wasm_bindgen_test]
fn test_children_by_class_multiple() {
    let html = "<div><span class='tag'>A</span><p class='tag'>B</p><div class='tag'>C</div></div>";
    let soup = Soup::new(html, None);
    let div = soup.find("div").unwrap().unwrap();

    let items = div.children_by_class("tag");
    assert_eq!(items.len(), 3);
}

#[wasm_bindgen_test]
fn test_children_by_class_multiple_classes() {
    let html = "<div><span class='item active'>A</span><span class='item'>B</span></div>";
    let soup = Soup::new(html, None);
    let div = soup.find("div").unwrap().unwrap();

    let items = div.children_by_class("item");
    assert_eq!(items.len(), 2);

    let active_items = div.children_by_class("active");
    assert_eq!(active_items.len(), 1);
    assert_eq!(active_items[0].text(), "A");
}

#[wasm_bindgen_test]
fn test_children_by_class_none_match() {
    let html = "<div><span class='a'>A</span><span class='b'>B</span></div>";
    let soup = Soup::new(html, None);
    let div = soup.find("div").unwrap().unwrap();

    let items = div.children_by_class("c");
    assert_eq!(items.len(), 0);
}

#[wasm_bindgen_test]
fn test_children_by_class_empty() {
    let html = "<div></div>";
    let soup = Soup::new(html, None);
    let div = soup.find("div").unwrap().unwrap();

    let items = div.children_by_class("item");
    assert_eq!(items.len(), 0);
}

#[wasm_bindgen_test]
fn test_children_by_class_no_class_attr() {
    let html = "<div><span>A</span><span>B</span></div>";
    let soup = Soup::new(html, None);
    let div = soup.find("div").unwrap().unwrap();

    let items = div.children_by_class("item");
    assert_eq!(items.len(), 0);
}

// ==================== Phase 13b: Scoped Extraction Tests ====================

#[wasm_bindgen_test]
fn test_tag_select_text() {
    let html = "<div class='container'><span class='item'>Inside</span></div><span \
                class='item'>Outside</span>";
    let soup = Soup::new(html, None);
    let container = soup.find(".container").unwrap().unwrap();

    let texts = container.select_text(".item").unwrap();
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0], "Inside");
}

#[wasm_bindgen_test]
fn test_tag_select_attr() {
    let html =
        "<div class='container'><a href='/inside'>Inside</a></div><a href='/outside'>Outside</a>";
    let soup = Soup::new(html, None);
    let container = soup.find(".container").unwrap().unwrap();

    let hrefs = container.select_attr("a", "href").unwrap();
    assert_eq!(hrefs.len(), 1);
}

#[wasm_bindgen_test]
fn test_tag_find_compiled() {
    let html = "<div><span class='target'>Found</span></div>";
    let soup = Soup::new(html, None);
    let div = soup.find("div").unwrap().unwrap();

    let selector = scrape_wasm::CompiledSelector::compile(".target").unwrap();
    let result = div.find_compiled(&selector);

    assert!(result.is_some());
    let tag = result.unwrap();
    assert_eq!(tag.text(), "Found");
}

#[wasm_bindgen_test]
fn test_tag_select_compiled() {
    let html = "<div><span class='item'>A</span><span class='item'>B</span></div>";
    let soup = Soup::new(html, None);
    let div = soup.find("div").unwrap().unwrap();

    let selector = scrape_wasm::CompiledSelector::compile(".item").unwrap();
    let results = div.select_compiled(&selector);

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].text(), "A");
    assert_eq!(results[1].text(), "B");
}

// ==================== Phase 13b: Edge Cases ====================

#[wasm_bindgen_test]
fn test_fragment_with_scripts() {
    let html = "<div>Before</div><script>alert('test');</script><div>After</div>";
    let soup = Soup::parse_fragment(html, None, None);
    let divs = soup.find_all("div").unwrap();
    assert_eq!(divs.len(), 2);
}

#[wasm_bindgen_test]
fn test_compiled_selector_pseudo_classes() {
    let html = "<ul><li>First</li><li>Second</li><li>Third</li></ul>";
    let soup = Soup::new(html, None);

    let selector = scrape_wasm::CompiledSelector::compile("li:first-child").unwrap();
    let result = soup.find_compiled(&selector).unwrap();

    assert_eq!(result.text(), "First");
}

#[wasm_bindgen_test]
fn test_select_text_empty_elements() {
    let html = "<div class='item'></div><div class='item'>Text</div>";
    let soup = Soup::new(html, None);

    let texts = soup.select_text(".item").unwrap();
    assert_eq!(texts.len(), 2);
    assert_eq!(texts[0], "");
    assert_eq!(texts[1], "Text");
}

#[wasm_bindgen_test]
fn test_select_attr_data_attributes() {
    let html = "<div><button data-id='1'>A</button><button data-id='2'>B</button><button \
                data-id='3'>C</button></div>";
    let soup = Soup::new(html, None);

    let ids = soup.select_attr("button", "data-id").unwrap();
    assert_eq!(ids.len(), 3);
}
