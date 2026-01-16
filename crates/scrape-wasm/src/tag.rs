//! Tag element wrapper for WASM.

use std::rc::Rc;

use js_sys::Object;
use scrape_core::{Document, NodeId, NodeKind, Soup as CoreSoup};
use wasm_bindgen::prelude::*;

/// An HTML element in the DOM tree.
///
/// Provides access to element content, attributes, and tree navigation.
///
/// @example
/// ```javascript
/// const soup = new Soup('<div class="test">Hello</div>');
/// const div = soup.find("div");
/// console.log(div.name);          // "div"
/// console.log(div.text);          // "Hello"
/// console.log(div.attr("class")); // "test"
/// ```
#[wasm_bindgen]
pub struct Tag {
    soup: Rc<CoreSoup>,
    id: NodeId,
}

impl Tag {
    /// Creates a new Tag from soup reference and node ID.
    #[must_use]
    pub fn new(soup: Rc<CoreSoup>, id: NodeId) -> Self {
        Self { soup, id }
    }

    /// Gets a reference to the document.
    fn doc(&self) -> &Document {
        self.soup.document()
    }
}

#[wasm_bindgen]
impl Tag {
    // ==================== Content Properties ====================

    /// Get the tag name (e.g., "div", "span").
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> Option<String> {
        self.doc().get(self.id).and_then(|n| n.kind.tag_name()).map(String::from)
    }

    /// Get the text content of this element and all descendants.
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> String {
        let mut result = String::new();
        collect_text(self.doc(), self.id, &mut result);
        result
    }

    /// Get the inner HTML content (excluding this element's tags).
    #[wasm_bindgen(getter, js_name = "innerHTML")]
    pub fn inner_html(&self) -> String {
        let mut result = String::new();
        for child_id in self.doc().children(self.id) {
            serialize_node(self.doc(), child_id, &mut result);
        }
        result
    }

    /// Get the outer HTML (including this element's tags).
    #[wasm_bindgen(getter, js_name = "outerHTML")]
    pub fn outer_html(&self) -> String {
        let mut result = String::new();
        serialize_node(self.doc(), self.id, &mut result);
        result
    }

    // ==================== Attribute Methods ====================

    /// Get an attribute value by name.
    ///
    /// @param name - The attribute name
    /// @returns The attribute value, or undefined if not present
    pub fn get(&self, name: &str) -> Option<String> {
        self.doc()
            .get(self.id)
            .and_then(|n| n.kind.attributes())
            .and_then(|attrs| attrs.get(name))
            .cloned()
    }

    /// Get an attribute value by name (alias for get).
    ///
    /// @param name - The attribute name
    /// @returns The attribute value, or undefined if not present
    pub fn attr(&self, name: &str) -> Option<String> {
        self.get(name)
    }

    /// Check if the element has an attribute.
    ///
    /// @param name - The attribute name
    /// @returns True if the attribute exists
    #[wasm_bindgen(js_name = "hasAttr")]
    pub fn has_attr(&self, name: &str) -> bool {
        self.doc()
            .get(self.id)
            .and_then(|n| n.kind.attributes())
            .is_some_and(|attrs| attrs.contains_key(name))
    }

    /// Get all attributes as an object.
    #[wasm_bindgen(getter)]
    pub fn attrs(&self) -> Object {
        let obj = Object::new();
        if let Some(node) = self.doc().get(self.id)
            && let Some(attrs) = node.kind.attributes()
        {
            for (k, v) in attrs {
                let _ = js_sys::Reflect::set(&obj, &k.into(), &v.into());
            }
        }
        obj
    }

    /// Check if the element has a specific class.
    ///
    /// @param className - The class name to check
    /// @returns True if the element has the class
    #[wasm_bindgen(js_name = "hasClass")]
    pub fn has_class(&self, class_name: &str) -> bool {
        self.get("class").is_some_and(|classes| classes.split_whitespace().any(|c| c == class_name))
    }

    /// Get all classes as an array.
    #[wasm_bindgen(getter)]
    pub fn classes(&self) -> Vec<String> {
        self.get("class")
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default()
    }

    // ==================== Navigation Properties ====================

    /// Get the parent element.
    #[wasm_bindgen(getter)]
    pub fn parent(&self) -> Option<Tag> {
        let doc = self.doc();
        doc.parent(self.id).and_then(|parent_id| {
            let node = doc.get(parent_id)?;
            if node.kind.is_element() {
                Some(Tag::new(Rc::clone(&self.soup), parent_id))
            } else {
                None
            }
        })
    }

    /// Get all direct child elements.
    #[wasm_bindgen(getter)]
    pub fn children(&self) -> Vec<Tag> {
        let doc = self.doc();
        doc.children(self.id)
            .filter_map(|child_id| {
                let node = doc.get(child_id)?;
                if node.kind.is_element() {
                    Some(Tag::new(Rc::clone(&self.soup), child_id))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the next sibling element.
    #[wasm_bindgen(getter, js_name = "nextSibling")]
    pub fn next_sibling(&self) -> Option<Tag> {
        let doc = self.doc();
        let mut current = doc.next_sibling(self.id);
        while let Some(sibling_id) = current {
            if let Some(node) = doc.get(sibling_id)
                && node.kind.is_element()
            {
                return Some(Tag::new(Rc::clone(&self.soup), sibling_id));
            }
            current = doc.next_sibling(sibling_id);
        }
        None
    }

    /// Get the previous sibling element.
    #[wasm_bindgen(getter, js_name = "prevSibling")]
    pub fn prev_sibling(&self) -> Option<Tag> {
        let doc = self.doc();
        let mut current = doc.prev_sibling(self.id);
        while let Some(sibling_id) = current {
            if let Some(node) = doc.get(sibling_id)
                && node.kind.is_element()
            {
                return Some(Tag::new(Rc::clone(&self.soup), sibling_id));
            }
            current = doc.prev_sibling(sibling_id);
        }
        None
    }

    /// Get all descendant elements.
    #[wasm_bindgen(getter)]
    pub fn descendants(&self) -> Vec<Tag> {
        let doc = self.doc();
        doc.descendants(self.id)
            .filter_map(|desc_id| {
                let node = doc.get(desc_id)?;
                if node.kind.is_element() {
                    Some(Tag::new(Rc::clone(&self.soup), desc_id))
                } else {
                    None
                }
            })
            .collect()
    }

    // ==================== Scoped Query Methods ====================

    /// Find the first descendant matching a CSS selector.
    ///
    /// @param selector - CSS selector string
    /// @returns The first matching Tag, or undefined if not found
    /// @throws Error if the selector syntax is invalid
    pub fn find(&self, selector: &str) -> Result<Option<Tag>, JsError> {
        scrape_core::query::find_within(self.doc(), self.id, selector)
            .map_err(|e| JsError::new(&e.to_string()))
            .map(|opt| opt.map(|id| Tag::new(Rc::clone(&self.soup), id)))
    }

    /// Find all descendants matching a CSS selector.
    ///
    /// @param selector - CSS selector string
    /// @returns Array of matching Tag instances
    /// @throws Error if the selector syntax is invalid
    #[wasm_bindgen(js_name = "findAll")]
    pub fn find_all(&self, selector: &str) -> Result<Vec<Tag>, JsError> {
        scrape_core::query::find_all_within(self.doc(), self.id, selector)
            .map_err(|e| JsError::new(&e.to_string()))
            .map(|ids| ids.into_iter().map(|id| Tag::new(Rc::clone(&self.soup), id)).collect())
    }

    /// Find all descendants matching a CSS selector (alias for findAll).
    ///
    /// @param selector - CSS selector string
    /// @returns Array of matching Tag instances
    pub fn select(&self, selector: &str) -> Result<Vec<Tag>, JsError> {
        self.find_all(selector)
    }

    /// Get the number of direct child elements.
    #[wasm_bindgen(getter)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn length(&self) -> u32 {
        self.doc()
            .children(self.id)
            .filter(|child_id| self.doc().get(*child_id).is_some_and(|n| n.kind.is_element()))
            .count() as u32
    }
}

impl Clone for Tag {
    fn clone(&self) -> Self {
        Self { soup: Rc::clone(&self.soup), id: self.id }
    }
}

// ==================== Helper Functions ====================

fn collect_text(doc: &Document, id: NodeId, result: &mut String) {
    let Some(node) = doc.get(id) else { return };

    match &node.kind {
        NodeKind::Text { content } => result.push_str(content),
        NodeKind::Element { .. } => {
            for child_id in doc.children(id) {
                collect_text(doc, child_id, result);
            }
        }
        NodeKind::Comment { .. } => {}
    }
}

fn serialize_node(doc: &Document, id: NodeId, result: &mut String) {
    let Some(node) = doc.get(id) else { return };

    match &node.kind {
        NodeKind::Element { name, attributes } => {
            result.push('<');
            result.push_str(name);
            for (attr_name, attr_value) in attributes {
                result.push(' ');
                result.push_str(attr_name);
                result.push_str("=\"");
                result.push_str(&escape_attr(attr_value));
                result.push('"');
            }
            result.push('>');

            if !is_void_element(name) {
                for child_id in doc.children(id) {
                    serialize_node(doc, child_id, result);
                }
                result.push_str("</");
                result.push_str(name);
                result.push('>');
            }
        }
        NodeKind::Text { content } => {
            result.push_str(&escape_text(content));
        }
        NodeKind::Comment { content } => {
            result.push_str("<!--");
            result.push_str(content);
            result.push_str("-->");
        }
    }
}

fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;").replace('>', "&gt;")
}

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
