//! HTML element type.
//!
//! The [`Tag`] struct represents a reference to an element in the DOM tree,
//! providing navigation and content extraction methods.

use std::collections::HashMap;

use crate::{
    dom::{Document, NodeId},
    query::{
        CompiledSelector, QueryResult, TextNodesIter, find_all_within, find_all_within_compiled,
        find_within, find_within_compiled, select_attr_within, select_text_within,
    },
    serialize::{collect_text as serialize_collect_text, serialize_node},
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

    /// Returns a reference to the document containing this tag.
    ///
    /// This method is primarily useful for advanced operations that need
    /// direct document access, such as custom serialization or traversal.
    #[must_use]
    pub fn document(&self) -> &'a Document {
        self.doc
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
    #[inline]
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
    #[inline]
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
    #[inline]
    #[must_use]
    pub fn has_class(&self, class: &str) -> bool {
        self.get("class").is_some_and(|classes| {
            #[cfg(feature = "simd")]
            {
                crate::simd::contains_class(classes, class)
            }
            #[cfg(not(feature = "simd"))]
            {
                classes.split_whitespace().any(|c| c == class)
            }
        })
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
        self.text_into(&mut result);
        result
    }

    /// Collects text content into the provided buffer.
    ///
    /// This method allows buffer reuse for repeated text extraction,
    /// avoiding allocations in performance-critical paths.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div>Hello</div><div>World</div>");
    /// let mut buffer = String::new();
    ///
    /// for div in soup.find_all("div").unwrap() {
    ///     buffer.clear();
    ///     div.text_into(&mut buffer);
    ///     println!("{}", buffer);
    /// }
    /// ```
    pub fn text_into(&self, buf: &mut String) {
        self.collect_text(buf);
    }

    fn collect_text(&self, buf: &mut String) {
        serialize_collect_text(self.doc, self.id, buf);
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
        serialize_node(self.doc, self.id, buf);
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

    /// Returns an iterator over all ancestor elements.
    ///
    /// Iterates from parent toward root (does not include the element itself).
    /// Only element nodes are included (text and comments are skipped).
    ///
    /// # Complexity
    ///
    /// - Time: `O(depth)` - iterates from node to root
    /// - Space: `O(1)` - lazy evaluation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<html><body><div><span>text</span></div></body></html>");
    /// if let Ok(Some(span)) = soup.find("span") {
    ///     let names: Vec<_> = span.parents().filter_map(|t| t.name().map(String::from)).collect();
    ///     assert_eq!(names, vec!["div", "body", "html"]);
    /// }
    /// ```
    pub fn parents(&self) -> impl Iterator<Item = Tag<'a>> {
        let doc = self.doc;
        self.doc
            .ancestors(self.id)
            .filter(move |id| doc.get(*id).is_some_and(|n| n.kind.is_element()))
            .map(move |id| Tag::new(doc, id))
    }

    /// Returns an iterator over all ancestor elements.
    ///
    /// Alias for [`parents`](Self::parents).
    pub fn ancestors(&self) -> impl Iterator<Item = Tag<'a>> {
        self.parents()
    }

    /// Finds the nearest ancestor matching the CSS selector.
    ///
    /// Iterates from parent toward root, returning the first match.
    /// Returns `Ok(None)` if no ancestor matches. Does not match the element itself.
    ///
    /// # Complexity
    ///
    /// - Time: `O(depth × selector_complexity)` - tests each ancestor against selector
    /// - Space: `O(1)` - no allocation
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
    /// let soup = Soup::parse("<div class='outer'><div class='inner'><span>text</span></div></div>");
    /// if let Ok(Some(span)) = soup.find("span") {
    ///     let inner = span.closest("div.inner").unwrap();
    ///     assert!(inner.is_some());
    ///     assert!(inner.unwrap().has_class("inner"));
    ///
    ///     let outer = span.closest("div.outer").unwrap();
    ///     assert!(outer.is_some());
    ///     assert!(outer.unwrap().has_class("outer"));
    ///
    ///     let none = span.closest("section").unwrap();
    ///     assert!(none.is_none());
    /// }
    /// ```
    pub fn closest(&self, selector: &str) -> QueryResult<Option<Tag<'a>>> {
        use crate::query::{matches_selector_list, parse_selector};

        // Parse selector
        let selector_list = parse_selector(selector)?;

        // Iterate ancestors and test each
        for ancestor_id in self.doc.ancestors(self.id) {
            let Some(node) = self.doc.get(ancestor_id) else {
                continue;
            };
            if !node.kind.is_element() {
                continue;
            }

            // Test if ancestor matches selector
            if matches_selector_list(self.doc, ancestor_id, &selector_list) {
                return Ok(Some(Tag::new(self.doc, ancestor_id)));
            }
        }

        Ok(None)
    }

    /// Returns an iterator over following sibling elements.
    ///
    /// Does not include the element itself. Only element nodes are included.
    ///
    /// # Complexity
    ///
    /// - Time: `O(width)` - iterates through siblings until end
    /// - Space: `O(1)` - lazy evaluation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<ul><li id='a'>A</li><li id='b'>B</li><li id='c'>C</li></ul>");
    /// if let Ok(Some(first)) = soup.find("li") {
    ///     let ids: Vec<_> =
    ///         first.next_siblings().filter_map(|t| t.get("id").map(String::from)).collect();
    ///     assert_eq!(ids, vec!["b", "c"]);
    /// }
    /// ```
    pub fn next_siblings(&self) -> impl Iterator<Item = Tag<'a>> {
        let doc = self.doc;
        self.doc
            .next_siblings(self.id)
            .filter(move |id| doc.get(*id).is_some_and(|n| n.kind.is_element()))
            .map(move |id| Tag::new(doc, id))
    }

    /// Returns an iterator over preceding sibling elements.
    ///
    /// Does not include the element itself. Only element nodes are included.
    /// Iterates in reverse order (from immediate predecessor toward first sibling).
    ///
    /// # Complexity
    ///
    /// - Time: `O(width)` - iterates through siblings until start
    /// - Space: `O(1)` - lazy evaluation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<ul><li id='a'>A</li><li id='b'>B</li><li id='c'>C</li></ul>");
    /// if let Ok(Some(last)) = soup.find("li#c") {
    ///     let ids: Vec<_> =
    ///         last.prev_siblings().filter_map(|t| t.get("id").map(String::from)).collect();
    ///     assert_eq!(ids, vec!["b", "a"]); // Reverse order
    /// }
    /// ```
    pub fn prev_siblings(&self) -> impl Iterator<Item = Tag<'a>> {
        let doc = self.doc;
        self.doc
            .prev_siblings(self.id)
            .filter(move |id| doc.get(*id).is_some_and(|n| n.kind.is_element()))
            .map(move |id| Tag::new(doc, id))
    }

    /// Returns an iterator over all sibling elements (excluding self).
    ///
    /// Iterates in document order (from first sibling to last).
    /// Only element nodes are included.
    ///
    /// # Complexity
    ///
    /// - Time: `O(width)` - iterates through all siblings
    /// - Space: `O(1)` - lazy evaluation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<ul><li id='a'>A</li><li id='b'>B</li><li id='c'>C</li></ul>");
    /// if let Ok(Some(middle)) = soup.find("li#b") {
    ///     let ids: Vec<_> = middle.siblings().filter_map(|t| t.get("id").map(String::from)).collect();
    ///     assert_eq!(ids, vec!["a", "c"]); // Document order
    /// }
    /// ```
    pub fn siblings(&self) -> impl Iterator<Item = Tag<'a>> {
        let doc = self.doc;
        self.doc
            .siblings(self.id)
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

    /// Finds the first descendant using a pre-compiled selector.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::{Soup, query::CompiledSelector};
    ///
    /// let selector = CompiledSelector::compile(".item").unwrap();
    /// let soup = Soup::parse("<div><ul><li class=\"item\">text</li></ul></div>");
    /// if let Ok(Some(div)) = soup.find("div") {
    ///     let item = div.find_compiled(&selector);
    ///     assert!(item.is_some());
    /// }
    /// ```
    #[must_use]
    pub fn find_compiled(&self, selector: &CompiledSelector) -> Option<Tag<'a>> {
        find_within_compiled(self.doc, self.id, selector).map(|id| Tag::new(self.doc, id))
    }

    /// Finds all descendants using a pre-compiled selector.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::{Soup, query::CompiledSelector};
    ///
    /// let selector = CompiledSelector::compile("li").unwrap();
    /// let soup = Soup::parse("<ul><li>A</li><li>B</li><li>C</li></ul>");
    /// if let Ok(Some(ul)) = soup.find("ul") {
    ///     let items = ul.select_compiled(&selector);
    ///     assert_eq!(items.len(), 3);
    /// }
    /// ```
    #[must_use]
    pub fn select_compiled(&self, selector: &CompiledSelector) -> Vec<Tag<'a>> {
        find_all_within_compiled(self.doc, self.id, selector)
            .into_iter()
            .map(|id| Tag::new(self.doc, id))
            .collect()
    }

    /// Extracts text content from all descendants matching a CSS selector.
    ///
    /// Returns the concatenated text content of each matching element within
    /// this element's subtree.
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
    /// let soup = Soup::parse("<div><ul><li>First</li><li>Second</li></ul></div>");
    /// if let Ok(Some(div)) = soup.find("div") {
    ///     let texts = div.select_text("li").unwrap();
    ///     assert_eq!(texts, vec!["First", "Second"]);
    /// }
    /// ```
    pub fn select_text(&self, selector: &str) -> QueryResult<Vec<String>> {
        select_text_within(self.doc, self.id, selector)
    }

    /// Extracts attribute values from all descendants matching a CSS selector.
    ///
    /// Returns `Some(value)` if the attribute exists, `None` if it doesn't.
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
    /// let soup = Soup::parse("<nav><a href='/1'>1</a><a href='/2'>2</a></nav>");
    /// if let Ok(Some(nav)) = soup.find("nav") {
    ///     let hrefs = nav.select_attr("a", "href").unwrap();
    ///     assert_eq!(hrefs, vec![Some("/1".to_string()), Some("/2".to_string())]);
    /// }
    /// ```
    pub fn select_attr(&self, selector: &str, attr: &str) -> QueryResult<Vec<Option<String>>> {
        select_attr_within(self.doc, self.id, selector, attr)
    }

    /// Returns an iterator over all text nodes in this subtree.
    ///
    /// Only text node content is returned; element tags and comments are skipped.
    /// Iterates in depth-first order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div>Hello <b>World</b>!</div>");
    /// if let Ok(Some(div)) = soup.find("div") {
    ///     let texts: Vec<_> = div.text_nodes().collect();
    ///     assert_eq!(texts, vec!["Hello ", "World", "!"]);
    /// }
    /// ```
    #[must_use]
    pub fn text_nodes(&self) -> TextNodesIter<'a> {
        TextNodesIter::new(self.doc, self.id)
    }

    /// Returns an iterator over child elements with the given tag name.
    ///
    /// Only direct children are included (not descendants).
    /// Tag name matching is case-insensitive.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<ul><li>A</li><span>X</span><li>B</li></ul>");
    /// if let Ok(Some(ul)) = soup.find("ul") {
    ///     let lis: Vec<_> = ul.children_by_name("li").collect();
    ///     assert_eq!(lis.len(), 2);
    /// }
    /// ```
    pub fn children_by_name(&self, name: &'a str) -> impl Iterator<Item = Tag<'a>> + 'a {
        let doc = self.doc;
        let id = self.id;
        doc.children(id).filter_map(move |child_id| {
            let node = doc.get(child_id)?;
            let tag_name = node.kind.tag_name()?;
            if tag_name.eq_ignore_ascii_case(name) { Some(Tag::new(doc, child_id)) } else { None }
        })
    }

    /// Returns an iterator over child elements with the given class.
    ///
    /// Only direct children are included (not descendants).
    /// Elements are matched if they have the class in their class attribute.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div><span class=\"a\">A</span><span class=\"b\">B</span></div>");
    /// if let Ok(Some(div)) = soup.find("div") {
    ///     let results: Vec<_> = div.children_by_class("a").collect();
    ///     assert_eq!(results.len(), 1);
    /// }
    /// ```
    pub fn children_by_class(&self, class: &'a str) -> impl Iterator<Item = Tag<'a>> + 'a {
        let doc = self.doc;
        let id = self.id;
        doc.children(id).filter_map(move |child_id| {
            let node = doc.get(child_id)?;
            let attrs = node.kind.attributes()?;
            let classes = attrs.get("class")?;

            #[cfg(feature = "simd")]
            let matches = crate::simd::contains_class(classes, class);
            #[cfg(not(feature = "simd"))]
            let matches = classes.split_whitespace().any(|c| c == class);

            if matches { Some(Tag::new(doc, child_id)) } else { None }
        })
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

    #[test]
    fn test_parents() {
        let soup = Soup::parse("<html><body><div><span>text</span></div></body></html>");
        let span = soup.find("span").unwrap().unwrap();

        let parents: Vec<_> = span.parents().collect();
        assert_eq!(parents.len(), 3);
        assert_eq!(parents[0].name(), Some("div"));
        assert_eq!(parents[1].name(), Some("body"));
        assert_eq!(parents[2].name(), Some("html"));
    }

    #[test]
    fn test_parents_empty_for_root() {
        let soup = Soup::parse("<html><body></body></html>");
        let html = soup.find("html").unwrap().unwrap();

        assert_eq!(html.parents().count(), 0);
    }

    #[test]
    fn test_ancestors_alias() {
        let soup = Soup::parse("<div><span>text</span></div>");
        let span = soup.find("span").unwrap().unwrap();

        assert_eq!(span.parents().count(), span.ancestors().count());
    }

    #[test]
    fn test_closest_basic() {
        let soup =
            Soup::parse("<div class='outer'><div class='inner'><span>text</span></div></div>");
        let span = soup.find("span").unwrap().unwrap();

        let inner = span.closest("div.inner").unwrap().unwrap();
        assert!(inner.has_class("inner"));

        let outer = span.closest("div.outer").unwrap().unwrap();
        assert!(outer.has_class("outer"));
    }

    #[test]
    fn test_closest_not_found() {
        let soup = Soup::parse("<div><span>text</span></div>");
        let span = soup.find("span").unwrap().unwrap();

        let result = span.closest("section").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_closest_invalid_selector() {
        let soup = Soup::parse("<div><span>text</span></div>");
        let span = soup.find("span").unwrap().unwrap();

        // Use a truly invalid selector
        let result = span.closest("::invalid-pseudo");
        assert!(result.is_err());
    }

    #[test]
    fn test_closest_does_not_match_self() {
        let soup = Soup::parse("<div><span class='target'>text</span></div>");
        let span = soup.find("span").unwrap().unwrap();

        let result = span.closest("span.target").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_next_siblings() {
        let soup = Soup::parse("<ul><li id='a'>A</li><li id='b'>B</li><li id='c'>C</li></ul>");
        let first = soup.find("li").unwrap().unwrap();

        let siblings: Vec<_> = first.next_siblings().collect();
        assert_eq!(siblings.len(), 2);
        assert_eq!(siblings[0].get("id"), Some("b"));
        assert_eq!(siblings[1].get("id"), Some("c"));
    }

    #[test]
    fn test_next_siblings_empty() {
        let soup = Soup::parse("<ul><li id='a'>A</li><li id='b'>B</li></ul>");
        let last = soup.find("li#b").unwrap().unwrap();

        assert_eq!(last.next_siblings().count(), 0);
    }

    #[test]
    fn test_next_siblings_skips_text() {
        let soup = Soup::parse("<ul><li id='a'>A</li> text <li id='b'>B</li></ul>");
        let first = soup.find("li").unwrap().unwrap();

        let siblings: Vec<_> = first.next_siblings().collect();
        assert_eq!(siblings.len(), 1);
        assert_eq!(siblings[0].get("id"), Some("b"));
    }

    #[test]
    fn test_prev_siblings() {
        let soup = Soup::parse("<ul><li id='a'>A</li><li id='b'>B</li><li id='c'>C</li></ul>");
        let last = soup.find("li#c").unwrap().unwrap();

        let siblings: Vec<_> = last.prev_siblings().collect();
        assert_eq!(siblings.len(), 2);
        assert_eq!(siblings[0].get("id"), Some("b"));
        assert_eq!(siblings[1].get("id"), Some("a"));
    }

    #[test]
    fn test_prev_siblings_empty() {
        let soup = Soup::parse("<ul><li id='a'>A</li><li id='b'>B</li></ul>");
        let first = soup.find("li").unwrap().unwrap();

        assert_eq!(first.prev_siblings().count(), 0);
    }

    #[test]
    fn test_siblings() {
        let soup = Soup::parse("<ul><li id='a'>A</li><li id='b'>B</li><li id='c'>C</li></ul>");
        let middle = soup.find("li#b").unwrap().unwrap();

        let siblings: Vec<_> = middle.siblings().collect();
        assert_eq!(siblings.len(), 2);
        assert_eq!(siblings[0].get("id"), Some("a"));
        assert_eq!(siblings[1].get("id"), Some("c"));
    }

    #[test]
    fn test_siblings_only_child() {
        let soup = Soup::parse("<div><span>only</span></div>");
        let span = soup.find("span").unwrap().unwrap();

        assert_eq!(span.siblings().count(), 0);
    }

    #[test]
    fn test_siblings_skips_text_and_comments() {
        let soup = Soup::parse("<ul><li id='a'>A</li><!-- comment --> text <li id='b'>B</li></ul>");
        let first = soup.find("li").unwrap().unwrap();

        let siblings: Vec<_> = first.siblings().collect();
        assert_eq!(siblings.len(), 1);
        assert_eq!(siblings[0].get("id"), Some("b"));
    }

    #[test]
    fn test_tag_find_compiled() {
        use crate::query::CompiledSelector;

        let selector = CompiledSelector::compile(".item").unwrap();
        let soup = Soup::parse("<div><ul><li class=\"item\">text</li></ul></div>");
        let div = soup.find("div").unwrap().unwrap();
        let item = div.find_compiled(&selector);
        assert!(item.is_some());
        assert_eq!(item.unwrap().name(), Some("li"));
    }

    #[test]
    fn test_tag_select_compiled() {
        use crate::query::CompiledSelector;

        let selector = CompiledSelector::compile("li").unwrap();
        let soup = Soup::parse("<ul><li>A</li><li>B</li><li>C</li></ul>");
        let ul = soup.find("ul").unwrap().unwrap();
        let items = ul.select_compiled(&selector);
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_children_by_name() {
        let soup = Soup::parse("<ul><li>A</li><span>X</span><li>B</li></ul>");
        let ul = soup.find("ul").unwrap().unwrap();
        let lis: Vec<_> = ul.children_by_name("li").collect();
        assert_eq!(lis.len(), 2);
        assert_eq!(lis[0].text(), "A");
        assert_eq!(lis[1].text(), "B");
    }

    #[test]
    fn test_children_by_name_case_insensitive() {
        let soup = Soup::parse("<div><LI>A</LI><li>B</li></div>");
        let div = soup.find("div").unwrap().unwrap();
        assert_eq!(div.children_by_name("li").count(), 2);
    }

    #[test]
    fn test_children_by_name_none_matching() {
        let soup = Soup::parse("<div><span>A</span></div>");
        let div = soup.find("div").unwrap().unwrap();
        assert!(div.children_by_name("li").next().is_none());
    }

    #[test]
    fn test_children_by_name_only_direct_children() {
        let soup = Soup::parse("<div><span><li>Nested</li></span></div>");
        let div = soup.find("div").unwrap().unwrap();
        assert!(div.children_by_name("li").next().is_none());
    }

    #[test]
    fn test_children_by_class() {
        let soup = Soup::parse("<div><span class=\"a\">A</span><span class=\"b\">B</span></div>");
        let div = soup.find("div").unwrap().unwrap();
        let class_a: Vec<_> = div.children_by_class("a").collect();
        assert_eq!(class_a.len(), 1);
        assert_eq!(class_a[0].text(), "A");
    }

    #[test]
    fn test_children_by_class_multiple_classes() {
        let soup =
            Soup::parse("<div><span class=\"a b\">AB</span><span class=\"a c\">AC</span></div>");
        let div = soup.find("div").unwrap().unwrap();
        assert_eq!(div.children_by_class("a").count(), 2);
    }

    #[test]
    fn test_children_by_class_exact_match() {
        let soup = Soup::parse("<div><span class=\"abc\">X</span></div>");
        let div = soup.find("div").unwrap().unwrap();
        assert!(div.children_by_class("a").next().is_none());
    }

    #[test]
    fn test_children_by_class_none_matching() {
        let soup = Soup::parse("<div><span class=\"b\">B</span></div>");
        let div = soup.find("div").unwrap().unwrap();
        assert!(div.children_by_class("a").next().is_none());
    }

    #[test]
    fn test_children_by_class_only_direct_children() {
        let soup = Soup::parse("<div><ul><li class=\"item\">Nested</li></ul></div>");
        let div = soup.find("div").unwrap().unwrap();
        assert!(div.children_by_class("item").next().is_none());
    }

    #[test]
    fn test_text_into_empty_buffer() {
        let soup = Soup::parse("<div>Hello World</div>");
        let div = soup.find("div").unwrap().unwrap();
        let mut buffer = String::new();
        div.text_into(&mut buffer);
        assert_eq!(buffer, "Hello World");
    }

    #[test]
    fn test_text_into_existing_content() {
        let soup = Soup::parse("<div>Hello</div>");
        let div = soup.find("div").unwrap().unwrap();
        let mut buffer = String::from("Start: ");
        div.text_into(&mut buffer);
        assert_eq!(buffer, "Start: Hello");
    }

    #[test]
    fn test_text_into_buffer_reuse() {
        let soup = Soup::parse("<div>First</div><div>Second</div><div>Third</div>");
        let divs = soup.find_all("div").unwrap();
        let mut buffer = String::new();

        for (i, div) in divs.iter().enumerate() {
            if i > 0 {
                buffer.clear();
            }
            div.text_into(&mut buffer);
            match i {
                0 => assert_eq!(buffer, "First"),
                1 => assert_eq!(buffer, "Second"),
                2 => assert_eq!(buffer, "Third"),
                _ => panic!("unexpected iteration"),
            }
        }
    }

    #[test]
    fn test_text_into_nested_elements() {
        let soup = Soup::parse("<div>Hello <b>Bold</b> <i>Italic</i>!</div>");
        let div = soup.find("div").unwrap().unwrap();
        let mut buffer = String::new();
        div.text_into(&mut buffer);
        assert_eq!(buffer, "Hello BoldItalic!");
    }

    #[test]
    fn test_text_into_empty_element() {
        let soup = Soup::parse("<div></div>");
        let div = soup.find("div").unwrap().unwrap();
        let mut buffer = String::from("prefix");
        div.text_into(&mut buffer);
        assert_eq!(buffer, "prefix");
    }

    #[test]
    fn test_text_into_deeply_nested() {
        let soup = Soup::parse("<div><p><span><b>Deep</b></span></p></div>");
        let div = soup.find("div").unwrap().unwrap();
        let mut buffer = String::new();
        div.text_into(&mut buffer);
        assert_eq!(buffer, "Deep");
    }

    #[test]
    fn test_text_into_multiple_text_nodes() {
        let soup = Soup::parse("<div>First<span>Middle</span>Last</div>");
        let div = soup.find("div").unwrap().unwrap();
        let mut buffer = String::new();
        div.text_into(&mut buffer);
        assert_eq!(buffer, "FirstMiddleLast");
    }

    #[test]
    fn test_text_into_no_allocations_on_reuse() {
        let soup = Soup::parse("<div>Test</div>");
        let div = soup.find("div").unwrap().unwrap();

        let mut buffer = String::with_capacity(100);
        div.text_into(&mut buffer);
        let capacity_after_first = buffer.capacity();

        buffer.clear();
        div.text_into(&mut buffer);
        let capacity_after_second = buffer.capacity();

        assert_eq!(capacity_after_first, capacity_after_second);
        assert_eq!(buffer, "Test");
    }
}
