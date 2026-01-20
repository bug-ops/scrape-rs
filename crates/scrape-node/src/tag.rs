//! Node.js wrapper for Tag element.

use std::{collections::HashMap, sync::Arc};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use scrape_core::{Document, NodeId, NodeKind, Soup as CoreSoup};

use crate::{error::IntoNapiError, selector::CompiledSelector};

/// An HTML element in the document.
///
/// Provides access to element content, attributes, and tree navigation.
///
/// @example
/// ```javascript
/// const soup = new Soup('<div class="test">Hello</div>');
/// const div = soup.find("div");
/// console.log(div.name);         // "div"
/// console.log(div.text);         // "Hello"
/// console.log(div.attr("class")); // "test"
/// ```
#[napi]
pub struct Tag {
    soup: Arc<CoreSoup>,
    id: NodeId,
}

impl Tag {
    /// Create a new Tag from soup reference and node ID.
    pub fn new(soup: Arc<CoreSoup>, id: NodeId) -> Self {
        Self { soup, id }
    }

    /// Get the document reference.
    fn doc(&self) -> &Document {
        self.soup.document()
    }
}

#[napi]
impl Tag {
    // ==================== Content Properties ====================

    /// Get the tag name (e.g., "div", "span").
    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.doc().get(self.id).and_then(|n| n.kind.tag_name()).map(String::from)
    }

    /// Get the text content of this element and all descendants.
    #[napi(getter)]
    pub fn text(&self) -> String {
        let mut result = String::new();
        scrape_core::serialize::collect_text(self.doc(), self.id, &mut result);
        result
    }

    /// Get the inner HTML content (excluding this element's tags).
    #[napi(getter, js_name = "innerHTML")]
    pub fn inner_html(&self) -> String {
        let mut result = String::new();
        scrape_core::serialize::serialize_inner_html(self.doc(), self.id, &mut result);
        result
    }

    /// Get the outer HTML (including this element's tags).
    #[napi(getter, js_name = "outerHTML")]
    pub fn outer_html(&self) -> String {
        let mut result = String::new();
        scrape_core::serialize::serialize_node(self.doc(), self.id, &mut result);
        result
    }

    // ==================== Attribute Methods ====================

    /// Get an attribute value by name.
    ///
    /// @param name - The attribute name
    /// @returns The attribute value, or null if not present
    #[napi(js_name = "attr")]
    pub fn get_attr(&self, name: String) -> Option<String> {
        self.doc()
            .get(self.id)
            .and_then(|n| n.kind.attributes())
            .and_then(|attrs| attrs.get(&name))
            .cloned()
    }

    /// Get an attribute value by name (alias for attr).
    ///
    /// @param name - The attribute name
    /// @returns The attribute value, or null if not present
    #[napi]
    pub fn get(&self, name: String) -> Option<String> {
        self.get_attr(name)
    }

    /// Check if the element has an attribute.
    ///
    /// @param name - The attribute name
    /// @returns True if the attribute exists
    #[napi(js_name = "hasAttr")]
    pub fn has_attr(&self, name: String) -> bool {
        self.doc()
            .get(self.id)
            .and_then(|n| n.kind.attributes())
            .is_some_and(|attrs| attrs.contains_key(&name))
    }

    /// Get all attributes as an object.
    #[napi(getter)]
    pub fn attrs(&self) -> HashMap<String, String> {
        self.doc().get(self.id).and_then(|n| n.kind.attributes()).cloned().unwrap_or_default()
    }

    /// Check if the element has a specific class.
    ///
    /// @param className - The class name to check
    /// @returns True if the element has the class
    #[napi(js_name = "hasClass")]
    pub fn has_class(&self, class_name: String) -> bool {
        self.get_attr("class".to_string())
            .is_some_and(|classes| classes.split_whitespace().any(|c| c == class_name))
    }

    /// Get all classes as an array.
    #[napi(getter)]
    pub fn classes(&self) -> Vec<String> {
        self.get_attr("class".to_string())
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default()
    }

    // ==================== Navigation Properties ====================

    /// Get the parent element.
    #[napi(getter)]
    pub fn parent(&self) -> Option<Tag> {
        let doc = self.doc();
        doc.parent(self.id).and_then(|parent_id| {
            let node = doc.get(parent_id)?;
            if node.kind.is_element() {
                Some(Tag::new(Arc::clone(&self.soup), parent_id))
            } else {
                None
            }
        })
    }

    /// Get all direct child elements.
    #[napi(getter)]
    pub fn children(&self) -> Vec<Tag> {
        self.doc()
            .children(self.id)
            .elements()
            .map(|id| Tag::new(Arc::clone(&self.soup), id))
            .collect()
    }

    /// Get the next sibling element.
    #[napi(getter, js_name = "nextSibling")]
    pub fn next_sibling(&self) -> Option<Tag> {
        let doc = self.doc();
        let mut current = doc.next_sibling(self.id);
        while let Some(sibling_id) = current {
            if let Some(node) = doc.get(sibling_id)
                && node.kind.is_element()
            {
                return Some(Tag::new(Arc::clone(&self.soup), sibling_id));
            }
            current = doc.next_sibling(sibling_id);
        }
        None
    }

    /// Get the previous sibling element.
    #[napi(getter, js_name = "prevSibling")]
    pub fn prev_sibling(&self) -> Option<Tag> {
        let doc = self.doc();
        let mut current = doc.prev_sibling(self.id);
        while let Some(sibling_id) = current {
            if let Some(node) = doc.get(sibling_id)
                && node.kind.is_element()
            {
                return Some(Tag::new(Arc::clone(&self.soup), sibling_id));
            }
            current = doc.prev_sibling(sibling_id);
        }
        None
    }

    /// Get all descendant elements.
    #[napi(getter)]
    pub fn descendants(&self) -> Vec<Tag> {
        self.doc()
            .descendants(self.id)
            .elements()
            .map(|id| Tag::new(Arc::clone(&self.soup), id))
            .collect()
    }

    /// Get all ancestor elements (from parent toward root).
    #[napi(getter)]
    pub fn parents(&self) -> Vec<Tag> {
        self.doc()
            .ancestors(self.id)
            .elements()
            .map(|id| Tag::new(Arc::clone(&self.soup), id))
            .collect()
    }

    /// Get all ancestor elements (alias for parents).
    #[napi(getter)]
    pub fn ancestors(&self) -> Vec<Tag> {
        self.parents()
    }

    /// Find the nearest ancestor matching a CSS selector.
    ///
    /// @param selector - CSS selector string
    /// @returns The nearest matching ancestor Tag, or null if not found
    /// @throws Error if the selector syntax is invalid
    #[napi]
    pub fn closest(&self, selector: String) -> Result<Option<Tag>> {
        use scrape_core::query::{matches_selector_list, parse_selector};

        let selector_list = parse_selector(&selector).map_err(IntoNapiError::into_napi_error)?;
        let doc = self.doc();

        for ancestor_id in doc.ancestors(self.id) {
            let Some(node) = doc.get(ancestor_id) else {
                continue;
            };
            if !node.kind.is_element() {
                continue;
            }

            if matches_selector_list(doc, ancestor_id, &selector_list) {
                return Ok(Some(Tag::new(Arc::clone(&self.soup), ancestor_id)));
            }
        }

        Ok(None)
    }

    /// Get all following sibling elements.
    #[napi(getter, js_name = "nextSiblings")]
    pub fn next_siblings(&self) -> Vec<Tag> {
        self.doc()
            .next_siblings(self.id)
            .elements()
            .map(|id| Tag::new(Arc::clone(&self.soup), id))
            .collect()
    }

    /// Get all preceding sibling elements (in reverse order).
    #[napi(getter, js_name = "prevSiblings")]
    pub fn prev_siblings(&self) -> Vec<Tag> {
        self.doc()
            .prev_siblings(self.id)
            .elements()
            .map(|id| Tag::new(Arc::clone(&self.soup), id))
            .collect()
    }

    /// Get all sibling elements (excluding self, in document order).
    #[napi(getter)]
    pub fn siblings(&self) -> Vec<Tag> {
        self.doc()
            .siblings(self.id)
            .elements()
            .map(|id| Tag::new(Arc::clone(&self.soup), id))
            .collect()
    }

    // ==================== Scoped Query Methods ====================

    /// Find the first descendant matching a CSS selector.
    ///
    /// @param selector - CSS selector string
    /// @returns The first matching Tag, or null if not found
    /// @throws Error if the selector syntax is invalid
    #[napi]
    pub fn find(&self, selector: String) -> Result<Option<Tag>> {
        scrape_core::query::find_within(self.doc(), self.id, &selector)
            .map_err(IntoNapiError::into_napi_error)
            .map(|opt| opt.map(|id| Tag::new(Arc::clone(&self.soup), id)))
    }

    /// Find all descendants matching a CSS selector.
    ///
    /// @param selector - CSS selector string
    /// @returns Array of matching Tag instances
    /// @throws Error if the selector syntax is invalid
    #[napi(js_name = "findAll")]
    pub fn find_all(&self, selector: String) -> Result<Vec<Tag>> {
        scrape_core::query::find_all_within(self.doc(), self.id, &selector)
            .map_err(IntoNapiError::into_napi_error)
            .map(|ids| ids.into_iter().map(|id| Tag::new(Arc::clone(&self.soup), id)).collect())
    }

    /// Find all descendants matching a CSS selector (alias for findAll).
    #[napi]
    pub fn select(&self, selector: String) -> Result<Vec<Tag>> {
        self.find_all(selector)
    }

    /// Get the number of direct child elements.
    #[napi(getter)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn length(&self) -> u32 {
        self.doc()
            .children(self.id)
            .filter(|child_id| self.doc().get(*child_id).is_some_and(|n| n.kind.is_element()))
            .count() as u32
    }

    // ==================== Compiled Selector Methods ====================

    /// Find the first descendant matching a compiled selector.
    ///
    /// @param selector - A compiled CSS selector
    /// @returns The first matching Tag, or null if not found
    #[napi(js_name = "findCompiled")]
    pub fn find_compiled(&self, selector: &CompiledSelector) -> Option<Tag> {
        scrape_core::query::find_within_compiled(self.doc(), self.id, &selector.inner)
            .map(|id| Tag::new(Arc::clone(&self.soup), id))
    }

    /// Find all descendants matching a compiled selector.
    ///
    /// @param selector - A compiled CSS selector
    /// @returns Array of matching Tag instances
    #[napi(js_name = "selectCompiled")]
    pub fn select_compiled(&self, selector: &CompiledSelector) -> Vec<Tag> {
        scrape_core::query::find_all_within_compiled(self.doc(), self.id, &selector.inner)
            .into_iter()
            .map(|id| Tag::new(Arc::clone(&self.soup), id))
            .collect()
    }

    // ==================== Text and Iterator Methods ====================

    /// Get all direct text nodes (excluding descendants).
    ///
    /// @returns Array of text content strings
    ///
    /// @example
    /// ```javascript
    /// const soup = new Soup("<div>Text1<span>Inner</span>Text2</div>");
    /// const div = soup.find("div");
    /// const texts = div.textNodes;
    /// // texts: ["Text1", "Text2"]
    /// ```
    #[napi(getter, js_name = "textNodes")]
    pub fn text_nodes(&self) -> Vec<String> {
        self.doc()
            .children(self.id)
            .filter_map(|child_id| {
                let node = self.doc().get(child_id)?;
                match &node.kind {
                    NodeKind::Text { content } => Some(content.clone()),
                    _ => None,
                }
            })
            .collect()
    }

    /// Get all direct child elements with a specific tag name.
    ///
    /// @param name - The tag name to filter by
    /// @returns Array of matching child Tag instances
    ///
    /// @example
    /// ```javascript
    /// const soup = new Soup("<div><p>A</p><span>B</span><p>C</p></div>");
    /// const div = soup.find("div");
    /// const paras = div.childrenByName("p");
    /// // paras.length: 2
    /// ```
    #[napi(js_name = "childrenByName")]
    pub fn children_by_name(&self, name: String) -> Vec<Tag> {
        let doc = self.doc();
        doc.children(self.id)
            .filter_map(|child_id| {
                let node = doc.get(child_id)?;
                if node.kind.is_element() && node.kind.tag_name()? == name {
                    Some(Tag::new(Arc::clone(&self.soup), child_id))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all direct child elements with a specific class.
    ///
    /// @param className - The class name to filter by
    /// @returns Array of matching child Tag instances
    ///
    /// @example
    /// ```javascript
    /// const soup = new Soup("<div><p class='item'>A</p><span>B</span><p class='item'>C</p></div>");
    /// const div = soup.find("div");
    /// const items = div.childrenByClass("item");
    /// // items.length: 2
    /// ```
    #[napi(js_name = "childrenByClass")]
    pub fn children_by_class(&self, class_name: String) -> Vec<Tag> {
        let doc = self.doc();
        doc.children(self.id)
            .filter_map(|child_id| {
                let node = doc.get(child_id)?;
                if node.kind.is_element() {
                    let attrs = node.kind.attributes()?;
                    let classes = attrs.get("class")?;
                    if classes.split_whitespace().any(|c| c == class_name) {
                        return Some(Tag::new(Arc::clone(&self.soup), child_id));
                    }
                }
                None
            })
            .collect()
    }

    // ==================== Extraction Methods ====================

    /// Extract text content from all descendants matching a selector.
    ///
    /// @param selector - CSS selector string
    /// @returns Array of text content strings
    /// @throws Error if the selector syntax is invalid
    ///
    /// @example
    /// ```javascript
    /// const soup = new Soup("<div><p>A</p><p>B</p></div>");
    /// const div = soup.find("div");
    /// const texts = div.selectText("p");
    /// // texts: ["A", "B"]
    /// ```
    #[napi(js_name = "selectText")]
    pub fn select_text(&self, selector: String) -> Result<Vec<String>> {
        scrape_core::query::select_text_within(self.doc(), self.id, &selector)
            .map_err(IntoNapiError::into_napi_error)
    }

    /// Extract attribute values from all descendants matching a selector.
    ///
    /// @param selector - CSS selector string
    /// @param attr - Attribute name to extract
    /// @returns Array of attribute values (null if attribute is missing)
    /// @throws Error if the selector syntax is invalid
    ///
    /// @example
    /// ```javascript
    /// const soup = new Soup("<div><a href='/a'>A</a><a href='/b'>B</a></div>");
    /// const div = soup.find("div");
    /// const hrefs = div.selectAttr("a", "href");
    /// // hrefs: ["/a", "/b"]
    /// ```
    #[napi(js_name = "selectAttr")]
    pub fn select_attr(&self, selector: String, attr: String) -> Result<Vec<Option<String>>> {
        scrape_core::query::select_attr_within(self.doc(), self.id, &selector, &attr)
            .map_err(IntoNapiError::into_napi_error)
    }
}

impl Clone for Tag {
    fn clone(&self) -> Self {
        Self { soup: Arc::clone(&self.soup), id: self.id }
    }
}
