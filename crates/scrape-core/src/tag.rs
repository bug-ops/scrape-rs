//! HTML element type.
//!
//! The [`Tag`] struct represents a reference to an element in the DOM tree,
//! providing navigation and content extraction methods.

use std::{borrow::Cow, collections::HashMap};

use crate::{
    dom::{Document, NodeId, NodeKind},
    query::{QueryResult, find_all_within, find_within},
};

/// A reference to an element in the document.
///
/// `Tag` provides navigation and content extraction methods. It borrows from
/// the underlying [`Document`], ensuring the tag remains valid while in use.
///
/// # Design
///
/// - `Copy` trait enables cheap passing without ownership concerns
/// - Lifetime `'a` tied to Document prevents dangling references
/// - [`NodeId`] enables O(1) node access via arena
///
/// # Examples
///
/// ## Accessing Attributes
///
/// ```rust
/// use scrape_core::Soup;
///
/// let soup = Soup::parse("<a href=\"https://example.com\" class=\"link\">Link</a>");
/// if let Ok(Some(link)) = soup.find("a") {
///     assert_eq!(link.get("href"), Some("https://example.com"));
///     assert!(link.has_class("link"));
/// }
/// ```
///
/// ## Tree Navigation
///
/// ```rust
/// use scrape_core::Soup;
///
/// let soup = Soup::parse("<div><span>Child</span></div>");
/// if let Ok(Some(span)) = soup.find("span") {
///     if let Some(parent) = span.parent() {
///         assert_eq!(parent.name(), Some("div"));
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Tag<'a> {
    doc: &'a Document,
    id: NodeId,
}

impl<'a> Tag<'a> {
    /// Creates a new Tag reference.
    #[must_use]
    pub(crate) fn new(doc: &'a Document, id: NodeId) -> Self {
        Self { doc, id }
    }

    /// Returns the node ID.
    #[must_use]
    pub fn node_id(&self) -> NodeId {
        self.id
    }

    /// Returns the tag name (e.g., "div", "span", "a").
    ///
    /// Returns `None` if this is not an element node.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div></div>");
    /// if let Ok(Some(div)) = soup.find("div") {
    ///     assert_eq!(div.name(), Some("div"));
    /// }
    /// ```
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.doc.get(self.id).and_then(|n| n.kind.tag_name())
    }

    /// Returns the value of an attribute, if present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<a href=\"/page\">Link</a>");
    /// if let Ok(Some(link)) = soup.find("a") {
    ///     assert_eq!(link.get("href"), Some("/page"));
    ///     assert_eq!(link.get("class"), None);
    /// }
    /// ```
    #[must_use]
    pub fn get(&self, attr: &str) -> Option<&str> {
        self.doc
            .get(self.id)
            .and_then(|n| n.kind.attributes())
            .and_then(|attrs| attrs.get(attr).map(String::as_str))
    }

    /// Checks if this element has the specified attribute.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<input disabled type=\"text\">");
    /// if let Ok(Some(input)) = soup.find("input") {
    ///     assert!(input.has_attr("disabled"));
    ///     assert!(input.has_attr("type"));
    ///     assert!(!input.has_attr("value"));
    /// }
    /// ```
    #[must_use]
    pub fn has_attr(&self, attr: &str) -> bool {
        self.doc
            .get(self.id)
            .and_then(|n| n.kind.attributes())
            .is_some_and(|attrs| attrs.contains_key(attr))
    }

    /// Returns all attributes on this element.
    ///
    /// Returns `None` if this is not an element node.
    #[must_use]
    pub fn attrs(&self) -> Option<&HashMap<String, String>> {
        self.doc.get(self.id).and_then(|n| n.kind.attributes())
    }

    /// Checks if this element has the specified class.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div class=\"foo bar\"></div>");
    /// if let Ok(Some(div)) = soup.find("div") {
    ///     assert!(div.has_class("foo"));
    ///     assert!(div.has_class("bar"));
    ///     assert!(!div.has_class("baz"));
    /// }
    /// ```
    #[must_use]
    pub fn has_class(&self, class: &str) -> bool {
        self.get("class").is_some_and(|classes| classes.split_whitespace().any(|c| c == class))
    }

    /// Returns all classes on this element.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div class=\"foo bar baz\"></div>");
    /// if let Ok(Some(div)) = soup.find("div") {
    ///     let classes: Vec<_> = div.classes().collect();
    ///     assert_eq!(classes, vec!["foo", "bar", "baz"]);
    /// }
    /// ```
    pub fn classes(&self) -> impl Iterator<Item = &str> {
        self.get("class").map(|s| s.split_whitespace()).into_iter().flatten()
    }

    /// Returns the text content of this element and its descendants.
    ///
    /// HTML tags are stripped and only text nodes are included.
    /// Text from multiple nodes is concatenated with no separator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div>Hello <b>World</b>!</div>");
    /// if let Ok(Some(div)) = soup.find("div") {
    ///     assert_eq!(div.text(), "Hello World!");
    /// }
    /// ```
    #[must_use]
    pub fn text(&self) -> String {
        let mut result = String::new();
        self.collect_text(&mut result);
        result
    }

    fn collect_text(&self, buf: &mut String) {
        let Some(node) = self.doc.get(self.id) else { return };

        match &node.kind {
            NodeKind::Text { content } => buf.push_str(content),
            NodeKind::Element { .. } => {
                for child_id in self.doc.children(self.id) {
                    Tag::new(self.doc, child_id).collect_text(buf);
                }
            }
            NodeKind::Comment { .. } => {}
        }
    }

    /// Returns the inner HTML of this element.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div><span>Hello</span></div>");
    /// if let Ok(Some(div)) = soup.find("div") {
    ///     assert_eq!(div.inner_html(), "<span>Hello</span>");
    /// }
    /// ```
    #[must_use]
    pub fn inner_html(&self) -> String {
        let mut result = String::new();
        for child_id in self.doc.children(self.id) {
            Tag::new(self.doc, child_id).serialize_to(&mut result);
        }
        result
    }

    /// Returns the outer HTML of this element (including the tag itself).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div><span>Hello</span></div>");
    /// if let Ok(Some(span)) = soup.find("span") {
    ///     assert_eq!(span.outer_html(), "<span>Hello</span>");
    /// }
    /// ```
    #[must_use]
    pub fn outer_html(&self) -> String {
        let mut result = String::new();
        self.serialize_to(&mut result);
        result
    }

    fn serialize_to(&self, buf: &mut String) {
        let Some(node) = self.doc.get(self.id) else { return };

        match &node.kind {
            NodeKind::Element { name, attributes } => {
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
                    for child_id in self.doc.children(self.id) {
                        Tag::new(self.doc, child_id).serialize_to(buf);
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

    // ==================== Navigation ====================

    /// Returns the parent element, if any.
    ///
    /// Returns `None` for the root element or if the parent is not an element.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div><span>text</span></div>");
    /// if let Ok(Some(span)) = soup.find("span") {
    ///     let parent = span.parent().unwrap();
    ///     assert_eq!(parent.name(), Some("div"));
    /// }
    /// ```
    #[must_use]
    pub fn parent(&self) -> Option<Tag<'a>> {
        let parent_id = self.doc.parent(self.id)?;
        let parent_node = self.doc.get(parent_id)?;
        if parent_node.kind.is_element() { Some(Tag::new(self.doc, parent_id)) } else { None }
    }

    /// Returns an iterator over direct child elements.
    ///
    /// Only element nodes are included (text and comments are skipped).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<ul><li>A</li><li>B</li><li>C</li></ul>");
    /// if let Ok(Some(ul)) = soup.find("ul") {
    ///     let children: Vec<_> = ul.children().collect();
    ///     assert_eq!(children.len(), 3);
    /// }
    /// ```
    pub fn children(&self) -> impl Iterator<Item = Tag<'a>> {
        let doc = self.doc;
        self.doc
            .children(self.id)
            .filter(move |id| doc.get(*id).is_some_and(|n| n.kind.is_element()))
            .map(move |id| Tag::new(doc, id))
    }

    /// Returns the next sibling element.
    ///
    /// Skips text and comment nodes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<ul><li id=\"a\">A</li><li id=\"b\">B</li></ul>");
    /// if let Ok(Some(first)) = soup.find("li") {
    ///     if let Some(next) = first.next_sibling() {
    ///         assert_eq!(next.get("id"), Some("b"));
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn next_sibling(&self) -> Option<Tag<'a>> {
        let mut current = self.doc.next_sibling(self.id);
        while let Some(sibling_id) = current {
            if let Some(node) = self.doc.get(sibling_id)
                && node.kind.is_element()
            {
                return Some(Tag::new(self.doc, sibling_id));
            }
            current = self.doc.next_sibling(sibling_id);
        }
        None
    }

    /// Returns the previous sibling element.
    ///
    /// Skips text and comment nodes.
    #[must_use]
    pub fn prev_sibling(&self) -> Option<Tag<'a>> {
        let mut current = self.doc.prev_sibling(self.id);
        while let Some(sibling_id) = current {
            if let Some(node) = self.doc.get(sibling_id)
                && node.kind.is_element()
            {
                return Some(Tag::new(self.doc, sibling_id));
            }
            current = self.doc.prev_sibling(sibling_id);
        }
        None
    }

    /// Returns an iterator over all descendant elements.
    ///
    /// Only element nodes are included (text and comments are skipped).
    pub fn descendants(&self) -> impl Iterator<Item = Tag<'a>> {
        let doc = self.doc;
        self.doc
            .descendants(self.id)
            .filter(move |id| doc.get(*id).is_some_and(|n| n.kind.is_element()))
            .map(move |id| Tag::new(doc, id))
    }

    // ==================== Scoped Queries ====================

    /// Finds the first descendant matching the selector.
    ///
    /// # Errors
    ///
    /// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div><ul><li class=\"item\">text</li></ul></div>");
    /// if let Ok(Some(div)) = soup.find("div") {
    ///     let item = div.find(".item").unwrap();
    ///     assert!(item.is_some());
    /// }
    /// ```
    pub fn find(&self, selector: &str) -> QueryResult<Option<Tag<'a>>> {
        find_within(self.doc, self.id, selector).map(|opt| opt.map(|id| Tag::new(self.doc, id)))
    }

    /// Finds all descendants matching the selector.
    ///
    /// # Errors
    ///
    /// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<ul><li>A</li><li>B</li><li>C</li></ul>");
    /// if let Ok(Some(ul)) = soup.find("ul") {
    ///     let items = ul.find_all("li").unwrap();
    ///     assert_eq!(items.len(), 3);
    /// }
    /// ```
    pub fn find_all(&self, selector: &str) -> QueryResult<Vec<Tag<'a>>> {
        find_all_within(self.doc, self.id, selector)
            .map(|ids| ids.into_iter().map(|id| Tag::new(self.doc, id)).collect())
    }

    /// Selects descendants using a CSS selector.
    ///
    /// Alias for [`Tag::find_all`].
    ///
    /// # Errors
    ///
    /// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
    pub fn select(&self, selector: &str) -> QueryResult<Vec<Tag<'a>>> {
        self.find_all(selector)
    }
}

impl PartialEq for Tag<'_> {
    fn eq(&self, other: &Self) -> bool {
        // Document equality via pointer comparison ensures tags from different documents
        // are never considered equal, maintaining correctness for cross-document operations.
        // NodeId equality alone is insufficient since different documents may have nodes
        // with the same ID but different content.
        std::ptr::eq(self.doc, other.doc) && self.id == other.id
    }
}

impl Eq for Tag<'_> {}

/// Escapes special characters for HTML text content.
///
/// Returns borrowed input when no escaping is needed (common case),
/// avoiding allocation overhead.
fn escape_text(s: &str) -> Cow<'_, str> {
    if !s.contains(['&', '<', '>']) {
        return Cow::Borrowed(s);
    }

    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(c),
        }
    }
    Cow::Owned(result)
}

/// Escapes special characters for HTML attribute values.
///
/// Returns borrowed input when no escaping is needed (common case),
/// avoiding allocation overhead.
fn escape_attr(s: &str) -> Cow<'_, str> {
    if !s.contains(['&', '"', '<', '>']) {
        return Cow::Borrowed(s);
    }

    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(c),
        }
    }
    Cow::Owned(result)
}

/// Returns true if the element is a void element (no closing tag).
fn is_void_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

#[cfg(test)]
mod tests {
    use crate::Soup;

    #[test]
    fn test_tag_name() {
        let soup = Soup::parse("<div>text</div>");
        let tag = soup.find("div").unwrap().unwrap();
        assert_eq!(tag.name(), Some("div"));
    }

    #[test]
    fn test_tag_get_attribute() {
        let soup = Soup::parse("<a href=\"/page\" class=\"link\">text</a>");
        let tag = soup.find("a").unwrap().unwrap();
        assert_eq!(tag.get("href"), Some("/page"));
        assert_eq!(tag.get("class"), Some("link"));
        assert_eq!(tag.get("title"), None);
    }

    #[test]
    fn test_tag_has_attr() {
        let soup = Soup::parse("<input disabled type=\"text\">");
        let tag = soup.find("input").unwrap().unwrap();
        assert!(tag.has_attr("disabled"));
        assert!(tag.has_attr("type"));
        assert!(!tag.has_attr("value"));
    }

    #[test]
    fn test_tag_has_class() {
        let soup = Soup::parse("<div class=\"foo bar\">text</div>");
        let tag = soup.find("div").unwrap().unwrap();
        assert!(tag.has_class("foo"));
        assert!(tag.has_class("bar"));
        assert!(!tag.has_class("baz"));
    }

    #[test]
    fn test_tag_classes() {
        let soup = Soup::parse("<div class=\"foo bar baz\">text</div>");
        let tag = soup.find("div").unwrap().unwrap();
        let classes: Vec<_> = tag.classes().collect();
        assert_eq!(classes, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn test_tag_text() {
        let soup = Soup::parse("<div>Hello <b>World</b>!</div>");
        let tag = soup.find("div").unwrap().unwrap();
        assert_eq!(tag.text(), "Hello World!");
    }

    #[test]
    fn test_tag_text_nested() {
        let soup = Soup::parse("<div><p>First</p><p>Second</p></div>");
        let tag = soup.find("div").unwrap().unwrap();
        assert_eq!(tag.text(), "FirstSecond");
    }

    #[test]
    fn test_tag_inner_html() {
        let soup = Soup::parse("<div><span>Hello</span></div>");
        let tag = soup.find("div").unwrap().unwrap();
        assert_eq!(tag.inner_html(), "<span>Hello</span>");
    }

    #[test]
    fn test_tag_outer_html() {
        let soup = Soup::parse("<div><span>Hello</span></div>");
        let tag = soup.find("span").unwrap().unwrap();
        assert_eq!(tag.outer_html(), "<span>Hello</span>");
    }

    #[test]
    fn test_tag_outer_html_with_attrs() {
        let soup = Soup::parse("<a href=\"/page\" class=\"link\">text</a>");
        let tag = soup.find("a").unwrap().unwrap();
        let html = tag.outer_html();
        assert!(html.contains("<a "));
        assert!(html.contains("href=\"/page\""));
        assert!(html.contains("class=\"link\""));
        assert!(html.contains(">text</a>"));
    }

    #[test]
    fn test_tag_parent() {
        let soup = Soup::parse("<div><span>text</span></div>");
        let span = soup.find("span").unwrap().unwrap();
        let parent = span.parent().unwrap();
        assert_eq!(parent.name(), Some("div"));
    }

    #[test]
    fn test_tag_children() {
        let soup = Soup::parse("<ul><li>A</li><li>B</li><li>C</li></ul>");
        let ul = soup.find("ul").unwrap().unwrap();
        let children: Vec<_> = ul.children().collect();
        assert_eq!(children.len(), 3);
        for child in &children {
            assert_eq!(child.name(), Some("li"));
        }
    }

    #[test]
    fn test_tag_next_sibling() {
        let soup = Soup::parse("<ul><li id=\"a\">A</li><li id=\"b\">B</li></ul>");
        let first = soup.find("li").unwrap().unwrap();
        let second = first.next_sibling().unwrap();
        assert_eq!(second.get("id"), Some("b"));
    }

    #[test]
    fn test_tag_prev_sibling() {
        let soup = Soup::parse("<ul><li id=\"a\">A</li><li id=\"b\">B</li></ul>");
        let second = soup.find("li#b").unwrap().unwrap();
        let first = second.prev_sibling().unwrap();
        assert_eq!(first.get("id"), Some("a"));
    }

    #[test]
    fn test_tag_find_within() {
        let soup = Soup::parse("<div><ul><li class=\"item\">text</li></ul></div>");
        let div = soup.find("div").unwrap().unwrap();
        let item = div.find(".item").unwrap().unwrap();
        assert_eq!(item.name(), Some("li"));
    }

    #[test]
    fn test_tag_find_all_within() {
        let soup = Soup::parse("<div><span>1</span><span>2</span></div><span>3</span>");
        let div = soup.find("div").unwrap().unwrap();
        let spans = div.find_all("span").unwrap();
        assert_eq!(spans.len(), 2);
    }

    #[test]
    fn test_tag_copy() {
        let soup = Soup::parse("<div>text</div>");
        let tag1 = soup.find("div").unwrap().unwrap();
        let tag2 = tag1; // Copy
        assert_eq!(tag1, tag2);
    }

    #[test]
    fn test_tag_equality() {
        let soup = Soup::parse("<div><span id=\"a\">A</span><span id=\"b\">B</span></div>");
        let a1 = soup.find("#a").unwrap().unwrap();
        let a2 = soup.find("#a").unwrap().unwrap();
        let b = soup.find("#b").unwrap().unwrap();

        assert_eq!(a1, a2);
        assert_ne!(a1, b);
    }

    #[test]
    fn test_tag_descendants() {
        let soup = Soup::parse("<div><ul><li>A</li><li>B</li></ul></div>");
        let div = soup.find("div").unwrap().unwrap();
        assert!(div.descendants().count() >= 3); // ul, li, li at minimum
    }

    #[test]
    fn test_escape_text() {
        let soup = Soup::parse("<div>&lt;script&gt;</div>");
        let div = soup.find("div").unwrap().unwrap();
        let text = div.text();
        assert_eq!(text, "<script>");
    }

    #[test]
    fn test_void_element_serialization() {
        let soup = Soup::parse("<div><br><hr></div>");
        let div = soup.find("div").unwrap().unwrap();
        let html = div.inner_html();
        assert!(html.contains("<br>"));
        assert!(html.contains("<hr>"));
        assert!(!html.contains("</br>"));
    }

    #[test]
    fn test_tag_attrs() {
        let soup =
            Soup::parse("<div id=\"main\" class=\"container\" data-value=\"123\">text</div>");
        let div = soup.find("div").unwrap().unwrap();
        let attrs = div.attrs().unwrap();
        assert_eq!(attrs.get("id"), Some(&"main".to_string()));
        assert_eq!(attrs.get("class"), Some(&"container".to_string()));
        assert_eq!(attrs.get("data-value"), Some(&"123".to_string()));
    }
}
