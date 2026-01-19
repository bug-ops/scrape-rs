//! Find functions for locating elements in the DOM.
//!
//! This module provides functions to find elements using CSS selectors:
//!
//! - [`find`] - Find first matching element in the entire document
//! - [`find_all`] - Find all matching elements in the entire document
//! - [`find_within`] - Find first matching element within a subtree
//! - [`find_all_within`] - Find all matching elements within a subtree

use selectors::{context::SelectorCaches, parser::SelectorList};

use super::{
    CompiledSelector,
    error::QueryResult,
    selector::{ScrapeSelector, matches_selector_with_caches, parse_selector},
};
use crate::dom::{Document, NodeId};

/// Finds the first element matching a CSS selector.
///
/// # Errors
///
/// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
///
/// # Examples
///
/// ```rust
/// use scrape_core::{Html5everParser, Parser, query::find};
///
/// let parser = Html5everParser;
/// let doc = parser.parse("<div><span class=\"item\">text</span></div>").unwrap();
///
/// let result = find(&doc, "span.item").unwrap();
/// assert!(result.is_some());
/// ```
pub fn find(doc: &Document, selector: &str) -> QueryResult<Option<NodeId>> {
    // Fast path: simple ID selector
    if let Some(id) = selector.strip_prefix('#')
        && is_simple_selector(id)
        && let Some(index) = doc.index()
    {
        return Ok(index.get_by_id(id));
    }

    // Fast path: simple class selector
    if let Some(class) = selector.strip_prefix('.')
        && is_simple_selector(class)
        && let Some(index) = doc.index()
    {
        return Ok(index.get_by_class(class).first().copied());
    }

    // Fall back to full selector matching
    let selectors = parse_selector(selector)?;
    Ok(find_with_selector(doc, &selectors))
}

/// Finds all elements matching a CSS selector.
///
/// # Errors
///
/// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
///
/// # Examples
///
/// ```rust
/// use scrape_core::{Html5everParser, Parser, query::find_all};
///
/// let parser = Html5everParser;
/// let doc = parser.parse("<ul><li>A</li><li>B</li><li>C</li></ul>").unwrap();
///
/// let items = find_all(&doc, "li").unwrap();
/// assert_eq!(items.len(), 3);
/// ```
pub fn find_all(doc: &Document, selector: &str) -> QueryResult<Vec<NodeId>> {
    // Fast path: simple ID selector
    if let Some(id) = selector.strip_prefix('#')
        && is_simple_selector(id)
        && let Some(index) = doc.index()
    {
        return Ok(index.get_by_id(id).into_iter().collect());
    }

    // Fast path: simple class selector
    if let Some(class) = selector.strip_prefix('.')
        && is_simple_selector(class)
        && let Some(index) = doc.index()
    {
        return Ok(index.get_by_class(class).to_vec());
    }

    // Fall back to full selector matching
    let selectors = parse_selector(selector)?;
    Ok(find_all_with_selector(doc, &selectors))
}

/// Finds the first element matching a CSS selector within a subtree.
///
/// The search starts from the given scope node and only includes its descendants.
///
/// # Errors
///
/// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
///
/// # Examples
///
/// ```rust
/// use scrape_core::{Html5everParser, Parser, query::find_within};
///
/// let parser = Html5everParser;
/// let doc = parser
///     .parse("<div id=\"a\"><span>A</span></div><div id=\"b\"><span>B</span></div>")
///     .unwrap();
///
/// // Find div#a first
/// let scope = doc
///     .nodes()
///     .find(|(_, n)| n.kind.attributes().and_then(|a| a.get("id")) == Some(&"a".to_string()))
///     .map(|(id, _)| id)
///     .unwrap();
///
/// let result = find_within(&doc, scope, "span").unwrap();
/// assert!(result.is_some());
/// ```
pub fn find_within(doc: &Document, scope: NodeId, selector: &str) -> QueryResult<Option<NodeId>> {
    let selectors = parse_selector(selector)?;
    Ok(find_within_with_selector(doc, scope, &selectors))
}

/// Finds all elements matching a CSS selector within a subtree.
///
/// # Errors
///
/// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
pub fn find_all_within(doc: &Document, scope: NodeId, selector: &str) -> QueryResult<Vec<NodeId>> {
    let selectors = parse_selector(selector)?;
    Ok(find_all_within_with_selector(doc, scope, &selectors))
}

/// Finds the first element matching a pre-parsed selector.
///
/// Use this for repeated queries with the same selector to avoid re-parsing.
#[must_use]
pub fn find_with_selector(
    doc: &Document,
    selectors: &SelectorList<ScrapeSelector>,
) -> Option<NodeId> {
    let root = doc.root()?;
    let mut caches = SelectorCaches::default();

    // Check root first
    if matches_selector_with_caches(doc, root, selectors, &mut caches) {
        return Some(root);
    }

    // Then check descendants
    for id in doc.descendants(root) {
        if let Some(node) = doc.get(id)
            && node.kind.is_element()
            && matches_selector_with_caches(doc, id, selectors, &mut caches)
        {
            return Some(id);
        }
    }

    None
}

/// Finds all elements matching a pre-parsed selector.
#[must_use]
pub fn find_all_with_selector(
    doc: &Document,
    selectors: &SelectorList<ScrapeSelector>,
) -> Vec<NodeId> {
    let mut results = Vec::new();

    let Some(root) = doc.root() else {
        return results;
    };

    let mut caches = SelectorCaches::default();

    // Check root first
    if matches_selector_with_caches(doc, root, selectors, &mut caches) {
        results.push(root);
    }

    // Then check descendants
    for id in doc.descendants(root) {
        if let Some(node) = doc.get(id)
            && node.kind.is_element()
            && matches_selector_with_caches(doc, id, selectors, &mut caches)
        {
            results.push(id);
        }
    }

    results
}

/// Finds the first element matching a selector within a subtree.
#[must_use]
pub fn find_within_with_selector(
    doc: &Document,
    scope: NodeId,
    selectors: &SelectorList<ScrapeSelector>,
) -> Option<NodeId> {
    let mut caches = SelectorCaches::default();

    for id in doc.descendants(scope) {
        if let Some(node) = doc.get(id)
            && node.kind.is_element()
            && matches_selector_with_caches(doc, id, selectors, &mut caches)
        {
            return Some(id);
        }
    }
    None
}

/// Finds all elements matching a selector within a subtree.
#[must_use]
pub fn find_all_within_with_selector(
    doc: &Document,
    scope: NodeId,
    selectors: &SelectorList<ScrapeSelector>,
) -> Vec<NodeId> {
    let mut results = Vec::new();
    let mut caches = SelectorCaches::default();

    for id in doc.descendants(scope) {
        if let Some(node) = doc.get(id)
            && node.kind.is_element()
            && matches_selector_with_caches(doc, id, selectors, &mut caches)
        {
            results.push(id);
        }
    }

    results
}

/// Finds the first element matching a compiled selector.
///
/// # Examples
///
/// ```rust
/// use scrape_core::{
///     Html5everParser, Parser,
///     query::{CompiledSelector, find_compiled},
/// };
///
/// let parser = Html5everParser;
/// let doc = parser.parse("<div><span class=\"item\">text</span></div>").unwrap();
/// let selector = CompiledSelector::compile("span.item").unwrap();
///
/// let result = find_compiled(&doc, &selector);
/// assert!(result.is_some());
/// ```
#[must_use]
pub fn find_compiled(doc: &Document, selector: &CompiledSelector) -> Option<NodeId> {
    find_with_selector(doc, selector.selector_list())
}

/// Finds all elements matching a compiled selector.
///
/// # Examples
///
/// ```rust
/// use scrape_core::{
///     Html5everParser, Parser,
///     query::{CompiledSelector, find_all_compiled},
/// };
///
/// let parser = Html5everParser;
/// let doc = parser.parse("<ul><li>A</li><li>B</li><li>C</li></ul>").unwrap();
/// let selector = CompiledSelector::compile("li").unwrap();
///
/// let items = find_all_compiled(&doc, &selector);
/// assert_eq!(items.len(), 3);
/// ```
#[must_use]
pub fn find_all_compiled(doc: &Document, selector: &CompiledSelector) -> Vec<NodeId> {
    find_all_with_selector(doc, selector.selector_list())
}

/// Finds the first element matching a compiled selector within a subtree.
///
/// # Examples
///
/// ```rust
/// use scrape_core::{
///     Html5everParser, Parser,
///     query::{CompiledSelector, find_within_compiled},
/// };
///
/// let parser = Html5everParser;
/// let doc = parser
///     .parse("<div id=\"a\"><span>A</span></div><div id=\"b\"><span>B</span></div>")
///     .unwrap();
/// let selector = CompiledSelector::compile("span").unwrap();
///
/// // Find div#a first
/// let scope = doc
///     .nodes()
///     .find(|(_, n)| n.kind.attributes().and_then(|a| a.get("id")) == Some(&"a".to_string()))
///     .map(|(id, _)| id)
///     .unwrap();
///
/// let result = find_within_compiled(&doc, scope, &selector);
/// assert!(result.is_some());
/// ```
#[must_use]
pub fn find_within_compiled(
    doc: &Document,
    scope: NodeId,
    selector: &CompiledSelector,
) -> Option<NodeId> {
    find_within_with_selector(doc, scope, selector.selector_list())
}

/// Finds all elements matching a compiled selector within a subtree.
#[must_use]
pub fn find_all_within_compiled(
    doc: &Document,
    scope: NodeId,
    selector: &CompiledSelector,
) -> Vec<NodeId> {
    find_all_within_with_selector(doc, scope, selector.selector_list())
}

/// Checks if a selector string is simple (no combinators or complex syntax).
///
/// A simple selector is one that contains only alphanumeric characters, hyphens,
/// and underscores. It does not contain combinators (>, +, ~, space), attribute
/// selectors, pseudo-classes, or multiple selectors.
#[inline]
fn is_simple_selector(s: &str) -> bool {
    !s.is_empty() && !s.contains(['.', '#', '[', ']', ':', ' ', '>', '+', '~', ',', '*', '(', ')'])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Html5everParser, Parser};

    fn parse_doc(html: &str) -> Document {
        Html5everParser.parse(html).unwrap()
    }

    #[test]
    fn test_find_by_tag() {
        let doc = parse_doc("<div><span>text</span></div>");
        let result = find(&doc, "span").unwrap();
        assert!(result.is_some());

        let span_id = result.unwrap();
        assert_eq!(doc.get(span_id).unwrap().kind.tag_name(), Some("span"));
    }

    #[test]
    fn test_find_by_class() {
        let doc = parse_doc("<div class=\"container\"><span class=\"item\">text</span></div>");
        let result = find(&doc, ".item").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_find_by_id() {
        let doc = parse_doc("<div id=\"main\">text</div>");
        let result = find(&doc, "#main").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_find_returns_none_when_not_found() {
        let doc = parse_doc("<div>text</div>");
        let result = find(&doc, "span").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_find_invalid_selector() {
        let doc = parse_doc("<div>text</div>");
        let result = find(&doc, "[");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_all_by_tag() {
        let doc = parse_doc("<ul><li>A</li><li>B</li><li>C</li></ul>");
        let results = find_all(&doc, "li").unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_find_all_returns_empty_when_not_found() {
        let doc = parse_doc("<div>text</div>");
        let results = find_all(&doc, "span").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_find_all_by_class() {
        let doc =
            parse_doc("<div class=\"a\">1</div><div class=\"b\">2</div><div class=\"a\">3</div>");
        let results = find_all(&doc, ".a").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_find_with_compound_selector() {
        let doc =
            parse_doc("<div class=\"foo\" id=\"bar\">match</div><div class=\"foo\">no id</div>");
        let result = find(&doc, "div.foo#bar").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_find_with_descendant_combinator() {
        let doc = parse_doc("<div><ul><li>item</li></ul></div>");
        let result = find(&doc, "div li").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_find_with_child_combinator() {
        let doc =
            parse_doc("<div><span>direct</span></div><div><ul><span>nested</span></ul></div>");
        let results = find_all(&doc, "div > span").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_find_within_scope() {
        let doc = parse_doc("<div id=\"a\"><span>A</span></div><div id=\"b\"><span>B</span></div>");

        // Find div#b
        let scope = doc
            .nodes()
            .find(|(_, n)| {
                n.kind.attributes().and_then(|a| a.get("id")).is_some_and(|id| id == "b")
            })
            .map(|(id, _)| id)
            .unwrap();

        // Find span within div#b only
        let result = find_within(&doc, scope, "span").unwrap();
        assert!(result.is_some());

        // Verify it's the correct span (child of scope)
        let span_id = result.unwrap();
        let span_parent = doc.parent(span_id).unwrap();
        assert_eq!(span_parent, scope);
    }

    #[test]
    fn test_find_all_within_scope() {
        let doc = parse_doc("<ul id=\"list\"><li>1</li><li>2</li></ul><li>outside</li>");

        // Find ul#list
        let scope = doc
            .nodes()
            .find(|(_, n)| {
                n.kind.attributes().and_then(|a| a.get("id")).is_some_and(|id| id == "list")
            })
            .map(|(id, _)| id)
            .unwrap();

        let results = find_all_within(&doc, scope, "li").unwrap();
        assert_eq!(results.len(), 2); // Only li elements inside ul#list
    }

    #[test]
    fn test_find_returns_first_match() {
        let doc = parse_doc(
            "<div class=\"item\" id=\"first\">1</div><div class=\"item\" id=\"second\">2</div>",
        );
        let result = find(&doc, ".item").unwrap();
        assert!(result.is_some());

        let id = result.unwrap();
        let attrs = doc.get(id).unwrap().kind.attributes().unwrap();
        assert_eq!(attrs.get("id"), Some(&"first".to_string()));
    }

    #[test]
    fn test_find_all_preserves_order() {
        let doc = parse_doc("<ul><li id=\"a\">A</li><li id=\"b\">B</li><li id=\"c\">C</li></ul>");
        let results = find_all(&doc, "li").unwrap();

        let ids: Vec<_> = results
            .iter()
            .map(|id| {
                doc.get(*id).and_then(|n| n.kind.attributes()).and_then(|a| a.get("id").cloned())
            })
            .collect();

        assert_eq!(ids, vec![Some("a".into()), Some("b".into()), Some("c".into())]);
    }

    #[test]
    fn test_find_empty_document() {
        let doc = Document::new();
        let result = find(&doc, "div").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_find_all_empty_document() {
        let doc = Document::new();
        let results = find_all(&doc, "div").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_find_with_attribute_selector() {
        let doc = parse_doc("<input type=\"text\"><input type=\"password\">");
        let result = find(&doc, "input[type=\"text\"]").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_find_all_multiple_selectors() {
        let doc = parse_doc("<div>a</div><span>b</span><p>c</p>");
        let results = find_all(&doc, "div, span").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_find_universal_selector() {
        let doc = parse_doc("<div><span>text</span></div>");
        let results = find_all(&doc, "*").unwrap();
        // Should match html, head, body, div, span (and possibly more from html5ever)
        assert!(results.len() >= 2);
    }

    #[test]
    fn test_is_simple_selector() {
        assert!(is_simple_selector("main"));
        assert!(is_simple_selector("my-id"));
        assert!(is_simple_selector("my_class"));
        assert!(is_simple_selector("id123"));

        assert!(!is_simple_selector(""));
        assert!(!is_simple_selector("foo bar"));
        assert!(!is_simple_selector("foo.bar"));
        assert!(!is_simple_selector("foo#bar"));
        assert!(!is_simple_selector("foo[attr]"));
        assert!(!is_simple_selector("foo:hover"));
        assert!(!is_simple_selector("foo>bar"));
        assert!(!is_simple_selector("foo+bar"));
        assert!(!is_simple_selector("foo~bar"));
        assert!(!is_simple_selector("foo,bar"));
        assert!(!is_simple_selector("*"));
        assert!(!is_simple_selector("foo(bar)"));
    }

    #[test]
    fn test_fast_path_id_selector() {
        let doc = parse_doc("<div id='main'><span id='inner'>text</span></div>");

        let main = find(&doc, "#main").unwrap();
        assert!(main.is_some());
        let main_id = main.unwrap();
        assert_eq!(doc.get(main_id).unwrap().kind.tag_name(), Some("div"));

        let inner = find(&doc, "#inner").unwrap();
        assert!(inner.is_some());
    }

    #[test]
    fn test_fast_path_class_selector_find() {
        let doc = parse_doc("<div class='item'>A</div><div class='item'>B</div>");

        let first = find(&doc, ".item").unwrap();
        assert!(first.is_some());
    }

    #[test]
    fn test_fast_path_class_selector_find_all() {
        let doc = parse_doc(
            "<div class='item'>A</div><div class='item'>B</div><div class='item'>C</div>",
        );

        let items = find_all(&doc, ".item").unwrap();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_fast_path_id_not_found() {
        let doc = parse_doc("<div id='main'>text</div>");

        let result = find(&doc, "#notfound").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_fast_path_class_not_found() {
        let doc = parse_doc("<div class='foo'>text</div>");

        let results = find_all(&doc, ".notfound").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_complex_selector_fallback() {
        let doc = parse_doc("<div id='main' class='container'>text</div>");

        let result = find(&doc, "#main.container").unwrap();
        assert!(result.is_some());

        let result = find(&doc, "div#main").unwrap();
        assert!(result.is_some());

        let result = find(&doc, "div > #main").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_fast_path_duplicate_ids() {
        let doc = parse_doc("<div id='dup'>First</div><div id='dup'>Second</div>");

        let found = find(&doc, "#dup").unwrap();
        assert!(found.is_some());
    }

    #[test]
    fn test_fast_path_multiple_classes() {
        let doc = parse_doc("<div class='foo bar'>A</div><div class='bar baz'>B</div>");

        let items = find_all(&doc, ".bar").unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_fast_path_with_no_index() {
        let mut doc = Document::new();
        #[allow(clippy::default_trait_access)]
        let root_id = doc.create_element("html".to_string(), Default::default());
        doc.set_root(root_id);
        #[allow(clippy::default_trait_access)]
        let elem = doc.create_element("div".to_string(), Default::default());
        doc.append_child(root_id, elem);

        let result = find(&doc, "#test").unwrap();
        assert!(result.is_none());

        let results = find_all(&doc, ".test").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_fast_path_unicode_selectors() {
        let doc = parse_doc("<div id='日本語'>Japanese</div><div class='中文'>Chinese</div>");

        let result = find(&doc, "#日本語").unwrap();
        assert!(result.is_some());

        let results = find_all(&doc, ".中文").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_fast_path_very_long_selector() {
        let long_id = "a".repeat(1000);
        let html = format!("<div id='{long_id}'>text</div>");
        let doc = parse_doc(&html);

        let result = find(&doc, &format!("#{long_id}")).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_fast_path_empty_class_attribute() {
        let doc = parse_doc("<div class=''>Empty</div><div>No class</div>");

        let results = find_all(&doc, ".foo").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_fast_path_special_chars_in_selector() {
        let doc = parse_doc("<div id='test:id'>Colon</div><div class='foo.bar'>Dot</div>");

        let result = find(&doc, "#test\\:id").unwrap();
        assert!(result.is_some());

        let results = find_all(&doc, ".foo\\.bar").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_fast_path_vs_fallback_consistency() {
        let doc =
            parse_doc("<div id='main' class='container'>A</div><div class='container'>B</div>");

        let fast_result = find(&doc, "#main").unwrap();
        let fallback_result = find(&doc, "[id='main']").unwrap();
        assert_eq!(fast_result, fallback_result);

        let fast_results = find_all(&doc, ".container").unwrap();
        let fallback_results = find_all(&doc, "[class~='container']").unwrap();
        assert_eq!(fast_results.len(), fallback_results.len());
    }
}
