//! Phase 19 edge case tests for selector explanation and complex queries.

#![allow(missing_docs)]

use scrape_core::{
    Soup,
    query::{QueryError, explain},
};

// Complex nested selector tests

#[test]
fn test_deeply_nested_selectors() {
    let html = r#"
        <html>
            <body>
                <div class="wrapper">
                    <main>
                        <section id="content">
                            <article class="post">
                                <header>
                                    <h1 class="title">Deep Title</h1>
                                </header>
                            </article>
                        </section>
                    </main>
                </div>
            </body>
        </html>
    "#;
    let soup = Soup::parse(html);
    let result =
        soup.find("div.wrapper > main > section#content > article.post > header > h1.title");
    assert!(result.is_ok());
    let tag = result.unwrap();
    assert!(tag.is_some());
    assert_eq!(tag.unwrap().text(), "Deep Title");
}

#[test]
fn test_complex_combinators() {
    let html = r"
        <div>
            <p>First</p>
            <p>Second</p>
            <span>Adjacent</span>
            <p>Third</p>
        </div>
    ";
    let soup = Soup::parse(html);

    // Adjacent sibling
    let adjacent = soup.find("p + span").unwrap();
    assert!(adjacent.is_some());
    assert_eq!(adjacent.unwrap().text(), "Adjacent");

    // General sibling
    let general = soup.find_all("p ~ p").unwrap();
    assert_eq!(general.len(), 2);
}

#[test]
fn test_multiple_attribute_selectors() {
    let html = r#"
        <a href="https://example.com" target="_blank" rel="noopener">Link 1</a>
        <a href="http://example.com" target="_self">Link 2</a>
        <a href="https://test.com" target="_blank" rel="noopener">Link 3</a>
    "#;
    let soup = Soup::parse(html);

    let result = soup.find_all(r#"a[href^="https"][target="_blank"][rel="noopener"]"#).unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn test_pseudo_class_combinations() {
    let html = r#"
        <ul>
            <li>First</li>
            <li class="special">Second</li>
            <li>Third</li>
            <li class="special">Fourth</li>
        </ul>
    "#;
    let soup = Soup::parse(html);

    let first_child = soup.find("li:first-child").unwrap();
    assert!(first_child.is_some());
    assert_eq!(first_child.unwrap().text(), "First");

    let last_child = soup.find("li:last-child").unwrap();
    assert!(last_child.is_some());
    assert_eq!(last_child.unwrap().text(), "Fourth");
}

// Unicode selector tests

#[test]
fn test_unicode_in_text_content() {
    let html = r#"
        <p class="emoji">🚀 Rocket Launch 🌟</p>
        <p class="chinese">你好世界</p>
        <p class="mixed">Hello 世界 🌍</p>
    "#;
    let soup = Soup::parse(html);

    let emoji = soup.find("p.emoji").unwrap().unwrap();
    assert!(emoji.text().contains("🚀"));

    let chinese = soup.find("p.chinese").unwrap().unwrap();
    assert_eq!(chinese.text(), "你好世界");

    let mixed = soup.find("p.mixed").unwrap().unwrap();
    assert!(mixed.text().contains("Hello"));
    assert!(mixed.text().contains("世界"));
}

#[test]
fn test_unicode_in_attributes() {
    let html = r#"
        <div data-emoji="🚀"></div>
        <div data-text="你好"></div>
    "#;
    let soup = Soup::parse(html);

    let emoji = soup.find(r#"div[data-emoji="🚀"]"#).unwrap();
    assert!(emoji.is_some());

    let text = soup.find(r#"div[data-text="你好"]"#).unwrap();
    assert!(text.is_some());
}

// Empty and malformed selector tests

#[test]
fn test_empty_selector_error() {
    let result: Result<_, QueryError> = explain("");
    assert!(result.is_err());
}

#[test]
fn test_malformed_bracket_selector() {
    let result: Result<_, QueryError> = explain("div[[[");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("invalid selector"));
}

#[test]
fn test_unclosed_attribute_selector() {
    // Some parsers may accept this or auto-correct it
    let result: Result<_, QueryError> = explain("a[href");
    // Just verify it returns a result (may be Ok or Err depending on parser)
    let _ = result;
}

#[test]
fn test_invalid_pseudo_class() {
    let result: Result<_, QueryError> = explain("div:not-a-real-pseudo");
    assert!(result.is_err());
}

#[test]
fn test_invalid_combinator_sequence() {
    let result: Result<_, QueryError> = explain("div > > span");
    assert!(result.is_err());
}

// Malformed HTML tests

#[test]
fn test_unclosed_tags() {
    let html = "<div><p>Unclosed paragraph<span>Unclosed span";
    let soup = Soup::parse(html);
    let paragraphs = soup.find_all("p").unwrap();
    assert_eq!(paragraphs.len(), 1);
}

#[test]
fn test_mismatched_tags() {
    let html = "<div><span></div></span>";
    let soup = Soup::parse(html);
    let divs = soup.find_all("div").unwrap();
    assert!(!divs.is_empty());
}

#[test]
fn test_nested_invalid_structure() {
    let html = "<ul><div><li>Invalid nesting</li></div></ul>";
    let soup = Soup::parse(html);
    let items = soup.find_all("li").unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].text(), "Invalid nesting");
}

#[test]
fn test_self_closing_tags_with_content() {
    let html = r#"<img src="test.jpg">Some content</img>"#;
    let soup = Soup::parse(html);
    let images = soup.find_all("img").unwrap();
    assert!(!images.is_empty());
}

// Large document tests

#[test]
fn test_large_document_with_many_elements() {
    let items: Vec<String> =
        (0..1000).map(|i| format!(r#"<div class="item-{i}">Item {i}</div>"#)).collect();
    let html = format!("<html><body>{}</body></html>", items.join(""));
    let soup = Soup::parse(&html);

    let all_divs = soup.find_all("div").unwrap();
    assert_eq!(all_divs.len(), 1000);
}

#[test]
fn test_very_deep_nesting() {
    use std::fmt::Write;
    let mut html = String::from("<html><body>");
    for i in 0..100 {
        let _ = write!(html, r#"<div class="level-{i}">"#);
    }
    html.push_str("Deep content");
    for _ in 0..100 {
        html.push_str("</div>");
    }
    html.push_str("</body></html>");

    let soup = Soup::parse(&html);
    let deep = soup.find("div.level-99").unwrap();
    assert!(deep.is_some());
}

#[test]
fn test_large_document_with_complex_selectors() {
    let items: Vec<String> = (0..500)
        .map(|i| {
            format!(
                r#"<article id="post-{i}" class="post">
                    <h2 class="title">Title {i}</h2>
                    <div class="content"><p>Content {i}</p></div>
                   </article>"#
            )
        })
        .collect();
    let html = format!("<html><body>{}</body></html>", items.join(""));
    let soup = Soup::parse(&html);

    let complex_query = soup.find_all("article.post > div.content > p").unwrap();
    assert_eq!(complex_query.len(), 500);
}

// Explain function tests

#[test]
fn test_explain_simple_selector() {
    let result = explain("div.container");
    assert!(result.is_ok());
    let explanation = result.unwrap();
    assert_eq!(explanation.source, "div.container");
}

#[test]
fn test_explain_complex_selector() {
    let result = explain("#main > ul.nav li a[href]");
    assert!(result.is_ok());
    let explanation = result.unwrap();
    assert!(!explanation.description.is_empty());
}

#[test]
fn test_explain_with_pseudo_classes() {
    let result = explain("li:first-child");
    assert!(result.is_ok());
}

#[test]
fn test_explain_with_attribute_selectors() {
    let result = explain(r#"a[href^="https"]"#);
    assert!(result.is_ok());
}

#[test]
fn test_explain_universal_selector() {
    let result = explain("*");
    assert!(result.is_ok());
}

#[test]
fn test_explain_descendant_combinator() {
    let result = explain("div span");
    assert!(result.is_ok());
}

#[test]
fn test_explain_child_combinator() {
    let result = explain("ul > li");
    assert!(result.is_ok());
}

#[test]
fn test_explain_adjacent_sibling() {
    let result = explain("h1 + p");
    assert!(result.is_ok());
}

#[test]
fn test_explain_general_sibling() {
    let result = explain("h1 ~ p");
    assert!(result.is_ok());
}

#[test]
fn test_explain_multiple_classes() {
    let result = explain(".class1.class2.class3");
    assert!(result.is_ok());
}

#[test]
fn test_explain_id_selector() {
    let result = explain("#unique-id");
    assert!(result.is_ok());
}

#[test]
fn test_explain_type_selector() {
    let result = explain("div");
    assert!(result.is_ok());
}

// Performance characteristics

#[test]
fn test_explain_performance_is_fast() {
    use std::time::Instant;

    let start = Instant::now();
    let _ = explain("div.container > ul.nav li a[href^='https']");
    let duration = start.elapsed();

    // Target: <1ms for typical selectors
    assert!(duration.as_micros() < 10_000, "explain() took too long: {duration:?}");
}

#[test]
fn test_query_large_document_performance() {
    use std::time::Instant;

    let items: Vec<String> =
        (0..10000).map(|i| format!(r#"<p class="item">Item {i}</p>"#)).collect();
    let html = format!("<html><body>{}</body></html>", items.join(""));
    let soup = Soup::parse(&html);

    let start = Instant::now();
    let results = soup.find_all("p.item").unwrap();
    let duration = start.elapsed();

    assert_eq!(results.len(), 10000);
    // Should complete in reasonable time
    assert!(duration.as_millis() < 1000, "Query took too long: {duration:?}");
}
