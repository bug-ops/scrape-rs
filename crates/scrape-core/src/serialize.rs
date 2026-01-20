//! HTML serialization utilities.
//!
//! This module provides functions for serializing DOM nodes back to HTML
//! and extracting text content. These functions are used by all bindings
//! (Python, Node.js, WASM) to implement `inner_html`, `outer_html`, and `text`
//! properties.

use crate::{
    Document, NodeId, NodeKind,
    utils::{escape_attr, escape_text, is_void_element},
};

/// Serializes a DOM node and its subtree to HTML.
///
/// This function recursively serializes an element, its attributes, and all
/// descendant nodes to an HTML string. The output is appended to the provided
/// buffer.
///
/// # Serialization Rules
///
/// - **Elements**: Serialized as `<name attrs>children</name>` or `<name attrs>` for void elements
/// - **Text nodes**: Content is HTML-escaped using [`escape_text`]
/// - **Comments**: Serialized as `<!--content-->`
/// - **Attributes**: Values are HTML-escaped using [`escape_attr`]
///
/// # Examples
///
/// ```rust
/// use scrape_core::{Soup, serialize::serialize_node};
///
/// let soup = Soup::parse("<div class=\"test\"><span>Hello</span></div>");
/// let doc = soup.document();
/// let div_id = soup.find("div").unwrap().unwrap().node_id();
///
/// let mut html = String::new();
/// serialize_node(doc, div_id, &mut html);
/// assert!(html.contains("<div"));
/// assert!(html.contains("</div>"));
/// ```
pub fn serialize_node(doc: &Document, id: NodeId, buf: &mut String) {
    let Some(node) = doc.get(id) else { return };

    match &node.kind {
        NodeKind::Element { name, attributes, .. } => {
            buf.push('<');
            buf.push_str(name);

            for (attr_name, attr_value) in attributes {
                buf.push(' ');
                buf.push_str(attr_name);
                buf.push_str("=\"");
                buf.push_str(&escape_attr(attr_value));
                buf.push('"');
            }

            buf.push('>');

            if !is_void_element(name) {
                for child_id in doc.children(id) {
                    serialize_node(doc, child_id, buf);
                }
                buf.push_str("</");
                buf.push_str(name);
                buf.push('>');
            }
        }
        NodeKind::Text { content } => {
            buf.push_str(&escape_text(content));
        }
        NodeKind::Comment { content } => {
            buf.push_str("<!--");
            buf.push_str(content);
            buf.push_str("-->");
        }
    }
}

/// Serializes only the children of a node to HTML (inner HTML).
///
/// This is equivalent to calling [`serialize_node`] on each child and
/// concatenating the results.
///
/// # Examples
///
/// ```rust
/// use scrape_core::{Soup, serialize::serialize_inner_html};
///
/// let soup = Soup::parse("<div><span>A</span><span>B</span></div>");
/// let doc = soup.document();
/// let div_id = soup.find("div").unwrap().unwrap().node_id();
///
/// let mut html = String::new();
/// serialize_inner_html(doc, div_id, &mut html);
/// assert_eq!(html, "<span>A</span><span>B</span>");
/// ```
pub fn serialize_inner_html(doc: &Document, id: NodeId, buf: &mut String) {
    for child_id in doc.children(id) {
        serialize_node(doc, child_id, buf);
    }
}

/// Collects text content from a node and its descendants.
///
/// This function recursively traverses the DOM subtree and concatenates
/// all text node content into the provided buffer. Element and comment
/// nodes are skipped (only their text children are included).
///
/// # Examples
///
/// ```rust
/// use scrape_core::{Soup, serialize::collect_text};
///
/// let soup = Soup::parse("<div>Hello <b>World</b>!</div>");
/// let doc = soup.document();
/// let div_id = soup.find("div").unwrap().unwrap().node_id();
///
/// let mut text = String::new();
/// collect_text(doc, div_id, &mut text);
/// assert_eq!(text, "Hello World!");
/// ```
pub fn collect_text(doc: &Document, id: NodeId, buf: &mut String) {
    let Some(node) = doc.get(id) else { return };

    match &node.kind {
        NodeKind::Text { content } => buf.push_str(content),
        NodeKind::Element { .. } => {
            for child_id in doc.children(id) {
                collect_text(doc, child_id, buf);
            }
        }
        NodeKind::Comment { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Soup;

    #[test]
    fn test_serialize_node_element() {
        let soup = Soup::parse("<div>text</div>");
        let doc = soup.document();
        let div = soup.find("div").unwrap().unwrap();

        let mut buf = String::new();
        serialize_node(doc, div.node_id(), &mut buf);
        assert_eq!(buf, "<div>text</div>");
    }

    #[test]
    fn test_serialize_node_with_attributes() {
        let soup = Soup::parse("<a href=\"/page\" class=\"link\">click</a>");
        let doc = soup.document();
        let a = soup.find("a").unwrap().unwrap();

        let mut buf = String::new();
        serialize_node(doc, a.node_id(), &mut buf);
        assert!(buf.contains("href=\"/page\""));
        assert!(buf.contains("class=\"link\""));
        assert!(buf.contains(">click</a>"));
    }

    #[test]
    fn test_serialize_node_escapes_attr() {
        let soup = Soup::parse("<div data-value=\"a &amp; b\">text</div>");
        let doc = soup.document();
        let div = soup.find("div").unwrap().unwrap();

        let mut buf = String::new();
        serialize_node(doc, div.node_id(), &mut buf);
        assert!(buf.contains("data-value="));
    }

    #[test]
    fn test_serialize_node_void_element() {
        let soup = Soup::parse("<div><br><hr></div>");
        let doc = soup.document();
        let div = soup.find("div").unwrap().unwrap();

        let mut buf = String::new();
        serialize_node(doc, div.node_id(), &mut buf);
        assert!(buf.contains("<br>"));
        assert!(buf.contains("<hr>"));
        assert!(!buf.contains("</br>"));
        assert!(!buf.contains("</hr>"));
    }

    #[test]
    fn test_serialize_node_nested() {
        let soup = Soup::parse("<div><span><b>deep</b></span></div>");
        let doc = soup.document();
        let div = soup.find("div").unwrap().unwrap();

        let mut buf = String::new();
        serialize_node(doc, div.node_id(), &mut buf);
        assert_eq!(buf, "<div><span><b>deep</b></span></div>");
    }

    #[test]
    fn test_serialize_node_comment() {
        use crate::SoupConfig;

        let config = SoupConfig { include_comments: true, ..Default::default() };
        let soup = Soup::parse_with_config("<div>text<!-- comment -->more</div>", config);
        let doc = soup.document();
        let div = soup.find("div").unwrap().unwrap();

        let mut buf = String::new();
        serialize_node(doc, div.node_id(), &mut buf);
        assert!(buf.contains("<!-- comment -->"));
        assert!(buf.contains("text"));
        assert!(buf.contains("more"));
    }

    #[test]
    fn test_serialize_inner_html() {
        let soup = Soup::parse("<div><span>A</span><span>B</span></div>");
        let doc = soup.document();
        let div = soup.find("div").unwrap().unwrap();

        let mut buf = String::new();
        serialize_inner_html(doc, div.node_id(), &mut buf);
        assert_eq!(buf, "<span>A</span><span>B</span>");
    }

    #[test]
    fn test_collect_text_simple() {
        let soup = Soup::parse("<div>Hello World</div>");
        let doc = soup.document();
        let div = soup.find("div").unwrap().unwrap();

        let mut buf = String::new();
        collect_text(doc, div.node_id(), &mut buf);
        assert_eq!(buf, "Hello World");
    }

    #[test]
    fn test_collect_text_nested() {
        let soup = Soup::parse("<div>Hello <b>Bold</b> Text</div>");
        let doc = soup.document();
        let div = soup.find("div").unwrap().unwrap();

        let mut buf = String::new();
        collect_text(doc, div.node_id(), &mut buf);
        assert_eq!(buf, "Hello Bold Text");
    }

    #[test]
    fn test_collect_text_skips_comments() {
        let soup = Soup::parse("<div>text<!-- comment -->more</div>");
        let doc = soup.document();
        let div = soup.find("div").unwrap().unwrap();

        let mut buf = String::new();
        collect_text(doc, div.node_id(), &mut buf);
        assert_eq!(buf, "textmore");
    }

    #[test]
    fn test_collect_text_empty() {
        let soup = Soup::parse("<div></div>");
        let doc = soup.document();
        let div = soup.find("div").unwrap().unwrap();

        let mut buf = String::new();
        collect_text(doc, div.node_id(), &mut buf);
        assert_eq!(buf, "");
    }
}
