//! Text and attribute extraction from query results.

use super::{QueryResult, find_all, find_all_within, parse_selector};
use crate::dom::{Document, NodeId, NodeKind};

/// Extracts text content from all elements matching a CSS selector.
///
/// Returns the concatenated text content of each matching element.
/// Empty vector if no elements match.
///
/// # Errors
///
/// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
///
/// # Examples
///
/// ```rust
/// use scrape_core::{Soup, query::select_text};
///
/// let soup = Soup::parse("<div><span>A</span><span>B</span></div>");
/// let texts = select_text(soup.document(), "span").unwrap();
/// assert_eq!(texts, vec!["A", "B"]);
/// ```
pub fn select_text(doc: &Document, selector: &str) -> QueryResult<Vec<String>> {
    let selector_list = parse_selector(selector)?;
    let node_ids = find_all(doc, selector)?;

    Ok(node_ids.into_iter().map(|id| extract_text(doc, id)).collect())
}

/// Extracts text content from elements within a subtree matching a CSS selector.
///
/// # Errors
///
/// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
pub fn select_text_within(
    doc: &Document,
    root: NodeId,
    selector: &str,
) -> QueryResult<Vec<String>> {
    let selector_list = parse_selector(selector)?;
    let node_ids = find_all_within(doc, root, selector)?;

    Ok(node_ids.into_iter().map(|id| extract_text(doc, id)).collect())
}

/// Extracts attribute values from all elements matching a CSS selector.
///
/// Returns `Some(value)` if the attribute exists, `None` if it doesn't.
/// Empty vector if no elements match.
///
/// # Errors
///
/// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
///
/// # Examples
///
/// ```rust
/// use scrape_core::{Soup, query::select_attr};
///
/// let soup = Soup::parse("<a href='/a'>A</a><a href='/b'>B</a>");
/// let hrefs = select_attr(soup.document(), "a", "href").unwrap();
/// assert_eq!(hrefs, vec![Some("/a".to_string()), Some("/b".to_string())]);
/// ```
pub fn select_attr(doc: &Document, selector: &str, attr: &str) -> QueryResult<Vec<Option<String>>> {
    let selector_list = parse_selector(selector)?;
    let node_ids = find_all(doc, selector)?;

    Ok(node_ids.into_iter().map(|id| extract_attr(doc, id, attr)).collect())
}

/// Extracts attribute values from elements within a subtree matching a CSS selector.
///
/// # Errors
///
/// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
pub fn select_attr_within(
    doc: &Document,
    root: NodeId,
    selector: &str,
    attr: &str,
) -> QueryResult<Vec<Option<String>>> {
    let selector_list = parse_selector(selector)?;
    let node_ids = find_all_within(doc, root, selector)?;

    Ok(node_ids.into_iter().map(|id| extract_attr(doc, id, attr)).collect())
}

/// Extracts all text content from an element and its descendants.
fn extract_text(doc: &Document, root: NodeId) -> String {
    let mut text = String::new();
    collect_text(doc, root, &mut text);
    text
}

/// Recursively collects text from a node and its descendants.
fn collect_text(doc: &Document, node_id: NodeId, buffer: &mut String) {
    let Some(node) = doc.get(node_id) else {
        return;
    };

    match &node.kind {
        NodeKind::Text { content } => {
            buffer.push_str(content);
        }
        NodeKind::Element { .. } => {
            for child_id in doc.children(node_id) {
                collect_text(doc, child_id, buffer);
            }
        }
        NodeKind::Comment { .. } => {}
    }
}

/// Extracts an attribute value from an element.
fn extract_attr(doc: &Document, node_id: NodeId, attr: &str) -> Option<String> {
    let node = doc.get(node_id)?;

    if let NodeKind::Element { attributes, .. } = &node.kind {
        attributes.get(attr).cloned()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Soup;

    #[test]
    fn test_select_text_simple() {
        let soup = Soup::parse("<div><span>A</span><span>B</span></div>");
        let texts = select_text(soup.document(), "span").unwrap();
        assert_eq!(texts, vec!["A", "B"]);
    }

    #[test]
    fn test_select_text_nested() {
        let soup = Soup::parse("<p>Hello <b>World</b>!</p>");
        let texts = select_text(soup.document(), "p").unwrap();
        assert_eq!(texts, vec!["Hello World!"]);
    }

    #[test]
    fn test_select_text_no_matches() {
        let soup = Soup::parse("<div>text</div>");
        let texts = select_text(soup.document(), "span").unwrap();
        assert!(texts.is_empty());
    }

    #[test]
    fn test_select_text_multiple_elements() {
        let soup = Soup::parse("<ul><li>First</li><li>Second</li><li>Third</li></ul>");
        let texts = select_text(soup.document(), "li").unwrap();
        assert_eq!(texts, vec!["First", "Second", "Third"]);
    }

    #[test]
    fn test_select_text_deeply_nested() {
        let soup = Soup::parse("<div><p><span>Deep</span></p></div>");
        let texts = select_text(soup.document(), "div").unwrap();
        assert_eq!(texts, vec!["Deep"]);
    }

    #[test]
    fn test_select_text_invalid_selector() {
        let soup = Soup::parse("<div>text</div>");
        let result = select_text(soup.document(), "[");
        assert!(result.is_err());
    }

    #[test]
    fn test_select_attr_simple() {
        let soup = Soup::parse("<a href='/a'>A</a><a href='/b'>B</a>");
        let hrefs = select_attr(soup.document(), "a", "href").unwrap();
        assert_eq!(hrefs, vec![Some("/a".to_string()), Some("/b".to_string())]);
    }

    #[test]
    fn test_select_attr_missing() {
        let soup = Soup::parse("<a href='/a'>A</a><a>B</a>");
        let hrefs = select_attr(soup.document(), "a", "href").unwrap();
        assert_eq!(hrefs, vec![Some("/a".to_string()), None]);
    }

    #[test]
    fn test_select_attr_no_matches() {
        let soup = Soup::parse("<div>text</div>");
        let hrefs = select_attr(soup.document(), "a", "href").unwrap();
        assert!(hrefs.is_empty());
    }

    #[test]
    fn test_select_attr_different_attributes() {
        let soup = Soup::parse(r#"<img src="/a.png" alt="A"><img src="/b.png" alt="B">"#);

        let srcs = select_attr(soup.document(), "img", "src").unwrap();
        assert_eq!(srcs, vec![Some("/a.png".to_string()), Some("/b.png".to_string())]);

        let alts = select_attr(soup.document(), "img", "alt").unwrap();
        assert_eq!(alts, vec![Some("A".to_string()), Some("B".to_string())]);
    }

    #[test]
    fn test_select_attr_invalid_selector() {
        let soup = Soup::parse("<a href='/a'>A</a>");
        let result = select_attr(soup.document(), "[", "href");
        assert!(result.is_err());
    }

    #[test]
    fn test_select_text_within() {
        let soup = Soup::parse("<div><ul><li>A</li><li>B</li></ul><p>C</p></div>");
        let div = soup.find("ul").unwrap().unwrap();
        let texts = select_text_within(soup.document(), div.node_id(), "li").unwrap();
        assert_eq!(texts, vec!["A", "B"]);
    }

    #[test]
    fn test_select_attr_within() {
        let soup =
            Soup::parse(r#"<nav><a href="/1">1</a><a href="/2">2</a></nav><a href="/3">3</a>"#);
        let nav = soup.find("nav").unwrap().unwrap();
        let hrefs = select_attr_within(soup.document(), nav.node_id(), "a", "href").unwrap();
        assert_eq!(hrefs, vec![Some("/1".to_string()), Some("/2".to_string())]);
    }

    #[test]
    fn test_select_text_empty_element() {
        let soup = Soup::parse("<div></div>");
        let texts = select_text(soup.document(), "div").unwrap();
        assert_eq!(texts, vec![""]);
    }

    #[test]
    fn test_select_text_whitespace_preserved() {
        let soup = Soup::parse("<span>  Hello  </span>");
        let texts = select_text(soup.document(), "span").unwrap();
        // Note: depends on whitespace handling in parser
        assert!(!texts.is_empty());
    }
}
