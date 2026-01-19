//! Integration tests for navigation methods.

use scrape_core::Soup;

#[test]
fn navigation_chain_parent_to_root() {
    let html = r#"
        <html>
            <head><title>Test</title></head>
            <body>
                <div id="container">
                    <ul>
                        <li id="item1">Item 1</li>
                        <li id="item2">Item 2</li>
                        <li id="item3">Item 3</li>
                    </ul>
                </div>
            </body>
        </html>
    "#;

    let soup = Soup::parse(html);
    let item = soup.find("li#item2").unwrap().unwrap();

    // Navigate up
    let ancestors: Vec<_> = item.parents().filter_map(|t| t.name().map(String::from)).collect();
    assert!(ancestors.contains(&"ul".to_string()));
    assert!(ancestors.contains(&"div".to_string()));
    assert!(ancestors.contains(&"body".to_string()));
    assert!(ancestors.contains(&"html".to_string()));

    // Navigate laterally
    let prev: Vec<_> = item.prev_siblings().filter_map(|t| t.get("id").map(String::from)).collect();
    assert_eq!(prev, vec!["item1".to_string()]);

    let next: Vec<_> = item.next_siblings().filter_map(|t| t.get("id").map(String::from)).collect();
    assert_eq!(next, vec!["item3".to_string()]);

    // All siblings
    let all: Vec<_> = item.siblings().filter_map(|t| t.get("id").map(String::from)).collect();
    assert_eq!(all, vec!["item1".to_string(), "item3".to_string()]);
}

#[test]
fn closest_finds_nested_ancestor() {
    let html = r#"
        <div class="level1">
            <div class="level2">
                <div class="level3">
                    <span id="target">Target</span>
                </div>
            </div>
        </div>
    "#;

    let soup = Soup::parse(html);
    let target = soup.find("#target").unwrap().unwrap();

    let level3 = target.closest(".level3").unwrap().unwrap();
    assert!(level3.has_class("level3"));

    let level1 = target.closest(".level1").unwrap().unwrap();
    assert!(level1.has_class("level1"));

    let none = target.closest(".nonexistent").unwrap();
    assert!(none.is_none());
}

#[test]
fn navigation_with_mixed_content() {
    let html = r#"
        <div>
            <!-- comment -->
            <span id="a">A</span>
            Some text
            <span id="b">B</span>
            <!-- another comment -->
            <span id="c">C</span>
        </div>
    "#;

    let soup = Soup::parse(html);
    let b = soup.find("span#b").unwrap().unwrap();

    // Should skip text and comment nodes
    let prev: Vec<_> = b.prev_siblings().filter_map(|t| t.get("id").map(String::from)).collect();
    assert_eq!(prev, vec!["a".to_string()]);

    let next: Vec<_> = b.next_siblings().filter_map(|t| t.get("id").map(String::from)).collect();
    assert_eq!(next, vec!["c".to_string()]);

    let all: Vec<_> = b.siblings().filter_map(|t| t.get("id").map(String::from)).collect();
    assert_eq!(all, vec!["a".to_string(), "c".to_string()]);
}

#[test]
fn closest_with_complex_selector() {
    let html = r#"
        <div data-type="container" class="outer">
            <div data-type="wrapper" class="inner">
                <span id="target">Target</span>
            </div>
        </div>
    "#;

    let soup = Soup::parse(html);
    let target = soup.find("#target").unwrap().unwrap();

    let wrapper = target.closest("div[data-type='wrapper']").unwrap().unwrap();
    assert!(wrapper.has_class("inner"));

    let container = target.closest("div.outer[data-type='container']").unwrap().unwrap();
    assert!(container.has_class("outer"));
}

#[test]
fn navigation_empty_document() {
    let soup = Soup::parse("");
    assert!(soup.find("*").unwrap().is_none());
}

#[test]
fn navigation_single_element() {
    let soup = Soup::parse("<div></div>");
    let div = soup.find("div").unwrap().unwrap();

    // HTML5 parser adds <html> and <body> elements
    assert!(div.parents().count() >= 1);
    assert_eq!(div.siblings().count(), 0);
    assert_eq!(div.next_siblings().count(), 0);
    assert_eq!(div.prev_siblings().count(), 0);
}

#[test]
fn navigation_deeply_nested() {
    use std::fmt::Write;

    let mut html = String::new();
    for i in 0..100 {
        write!(&mut html, "<div id='d{i}'>").unwrap();
    }
    html.push_str("<span id='target'>Target</span>");
    for _ in 0..100 {
        html.push_str("</div>");
    }

    let soup = Soup::parse(&html);
    let target = soup.find("#target").unwrap().unwrap();

    // HTML5 parser adds <html> and <body>, so we have 100 divs + 2 implicit elements
    assert!(target.parents().count() >= 100);

    let closest = target.closest("div#d50").unwrap();
    assert!(closest.is_some());
}

#[test]
fn navigation_wide_siblings() {
    use std::fmt::Write;

    let mut html = String::from("<ul>");
    for i in 0..1000 {
        write!(&mut html, "<li id='i{i}'>Item {i}</li>").unwrap();
    }
    html.push_str("</ul>");

    let soup = Soup::parse(&html);
    let middle = soup.find("li#i500").unwrap().unwrap();

    assert_eq!(middle.prev_siblings().count(), 500);
    assert_eq!(middle.next_siblings().count(), 499);
    assert_eq!(middle.siblings().count(), 999);
}
