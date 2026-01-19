//! Python wrapper for Tag element.

use std::sync::Arc;

use pyo3::{exceptions::PyKeyError, prelude::*, types::PyDict};
use scrape_core::{Document, NodeId, NodeKind, Soup};

use crate::{error::IntoPyErr, selector::PyCompiledSelector};

/// An HTML element in the document.
///
/// Provides access to element content, attributes, and tree navigation.
///
/// Example:
///     >>> soup = Soup("<div class='test'>Hello</div>")
///     >>> div = soup.find("div")
///     >>> print(div.name)
///     div
///     >>> print(div.get("class"))
///     test
#[pyclass(name = "Tag")]
pub struct PyTag {
    soup: Arc<Soup>,
    id: NodeId,
}

impl PyTag {
    /// Create a new PyTag from soup reference and node ID.
    pub fn new(soup: Arc<Soup>, id: NodeId) -> Self {
        Self { soup, id }
    }

    /// Get the document reference.
    fn doc(&self) -> &Document {
        self.soup.document()
    }
}

#[pymethods]
impl PyTag {
    // ==================== Content Properties ====================

    /// Get the tag name (e.g., "div", "span").
    #[getter]
    fn name(&self) -> Option<String> {
        self.doc().get(self.id).and_then(|n| n.kind.tag_name()).map(String::from)
    }

    /// Get the text content of this element and all descendants.
    #[getter]
    fn text(&self) -> String {
        let mut result = String::new();
        collect_text(self.doc(), self.id, &mut result);
        result
    }

    /// Get the inner HTML content (excluding this element's tags).
    #[getter]
    fn inner_html(&self) -> String {
        let mut result = String::new();
        for child_id in self.doc().children(self.id) {
            serialize_node(self.doc(), child_id, &mut result);
        }
        result
    }

    /// Get the outer HTML (including this element's tags).
    #[getter]
    fn outer_html(&self) -> String {
        let mut result = String::new();
        serialize_node(self.doc(), self.id, &mut result);
        result
    }

    // ==================== Attribute Methods ====================

    /// Get an attribute value by name.
    ///
    /// Args:
    ///     name: The attribute name.
    ///
    /// Returns:
    ///     The attribute value, or None if not present.
    fn get(&self, name: &str) -> Option<String> {
        self.doc()
            .get(self.id)
            .and_then(|n| n.kind.attributes())
            .and_then(|attrs| attrs.get(name))
            .cloned()
    }

    /// Check if the element has an attribute.
    ///
    /// Args:
    ///     name: The attribute name.
    ///
    /// Returns:
    ///     True if the attribute exists.
    fn has_attr(&self, name: &str) -> bool {
        self.doc()
            .get(self.id)
            .and_then(|n| n.kind.attributes())
            .is_some_and(|attrs| attrs.contains_key(name))
    }

    /// Get all attributes as a dictionary.
    #[getter]
    fn attrs<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        if let Some(node) = self.doc().get(self.id)
            && let Some(attrs) = node.kind.attributes()
        {
            for (k, v) in attrs {
                dict.set_item(k, v)?;
            }
        }
        Ok(dict)
    }

    /// Check if the element has a specific class.
    ///
    /// Args:
    ///     class_name: The class name to check.
    ///
    /// Returns:
    ///     True if the element has the class.
    fn has_class(&self, class_name: &str) -> bool {
        self.get("class").is_some_and(|classes| classes.split_whitespace().any(|c| c == class_name))
    }

    /// Get all classes as a list.
    #[getter]
    fn classes(&self) -> Vec<String> {
        self.get("class")
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default()
    }

    // ==================== Navigation Properties ====================

    /// Get the parent element.
    #[getter]
    fn parent(&self) -> Option<PyTag> {
        let doc = self.doc();
        doc.parent(self.id).and_then(|parent_id| {
            let node = doc.get(parent_id)?;
            if node.kind.is_element() {
                Some(PyTag::new(Arc::clone(&self.soup), parent_id))
            } else {
                None
            }
        })
    }

    /// Get all child elements.
    #[getter]
    fn children(&self) -> Vec<PyTag> {
        let doc = self.doc();
        doc.children(self.id)
            .filter_map(|child_id| {
                let node = doc.get(child_id)?;
                if node.kind.is_element() {
                    Some(PyTag::new(Arc::clone(&self.soup), child_id))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the next sibling element.
    #[getter]
    fn next_sibling(&self) -> Option<PyTag> {
        let doc = self.doc();
        let mut current = doc.next_sibling(self.id);
        while let Some(sibling_id) = current {
            if let Some(node) = doc.get(sibling_id)
                && node.kind.is_element()
            {
                return Some(PyTag::new(Arc::clone(&self.soup), sibling_id));
            }
            current = doc.next_sibling(sibling_id);
        }
        None
    }

    /// Get the previous sibling element.
    #[getter]
    fn prev_sibling(&self) -> Option<PyTag> {
        let doc = self.doc();
        let mut current = doc.prev_sibling(self.id);
        while let Some(sibling_id) = current {
            if let Some(node) = doc.get(sibling_id)
                && node.kind.is_element()
            {
                return Some(PyTag::new(Arc::clone(&self.soup), sibling_id));
            }
            current = doc.prev_sibling(sibling_id);
        }
        None
    }

    /// Get all descendant elements.
    #[getter]
    fn descendants(&self) -> Vec<PyTag> {
        let doc = self.doc();
        doc.descendants(self.id)
            .filter_map(|desc_id| {
                let node = doc.get(desc_id)?;
                if node.kind.is_element() {
                    Some(PyTag::new(Arc::clone(&self.soup), desc_id))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all ancestor elements (from parent toward root).
    #[getter]
    fn parents(&self) -> Vec<PyTag> {
        let doc = self.doc();
        doc.ancestors(self.id)
            .filter_map(|ancestor_id| {
                let node = doc.get(ancestor_id)?;
                if node.kind.is_element() {
                    Some(PyTag::new(Arc::clone(&self.soup), ancestor_id))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all ancestor elements (alias for parents).
    #[getter]
    fn ancestors(&self) -> Vec<PyTag> {
        self.parents()
    }

    /// Find the nearest ancestor matching a CSS selector.
    ///
    /// Args:
    ///     selector: CSS selector string.
    ///
    /// Returns:
    ///     The nearest matching ancestor Tag, or None if not found.
    ///
    /// Raises:
    ///     ValueError: If the selector syntax is invalid.
    fn closest(&self, selector: &str) -> PyResult<Option<PyTag>> {
        use scrape_core::query::{matches_selector_list, parse_selector};

        let selector_list = parse_selector(selector).map_err(IntoPyErr::into_py_err)?;
        let doc = self.doc();

        for ancestor_id in doc.ancestors(self.id) {
            let Some(node) = doc.get(ancestor_id) else {
                continue;
            };
            if !node.kind.is_element() {
                continue;
            }

            if matches_selector_list(doc, ancestor_id, &selector_list) {
                return Ok(Some(PyTag::new(Arc::clone(&self.soup), ancestor_id)));
            }
        }

        Ok(None)
    }

    /// Get all following sibling elements.
    #[getter]
    fn next_siblings(&self) -> Vec<PyTag> {
        let doc = self.doc();
        doc.next_siblings(self.id)
            .filter_map(|sibling_id| {
                let node = doc.get(sibling_id)?;
                if node.kind.is_element() {
                    Some(PyTag::new(Arc::clone(&self.soup), sibling_id))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all preceding sibling elements (in reverse order).
    #[getter]
    fn prev_siblings(&self) -> Vec<PyTag> {
        let doc = self.doc();
        doc.prev_siblings(self.id)
            .filter_map(|sibling_id| {
                let node = doc.get(sibling_id)?;
                if node.kind.is_element() {
                    Some(PyTag::new(Arc::clone(&self.soup), sibling_id))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all sibling elements (excluding self, in document order).
    #[getter]
    fn siblings(&self) -> Vec<PyTag> {
        let doc = self.doc();
        doc.siblings(self.id)
            .filter_map(|sibling_id| {
                let node = doc.get(sibling_id)?;
                if node.kind.is_element() {
                    Some(PyTag::new(Arc::clone(&self.soup), sibling_id))
                } else {
                    None
                }
            })
            .collect()
    }

    // ==================== Scoped Query Methods ====================

    /// Find the first descendant matching a CSS selector.
    ///
    /// Args:
    ///     selector: CSS selector string.
    ///
    /// Returns:
    ///     The first matching Tag, or None if not found.
    ///
    /// Raises:
    ///     ValueError: If the selector syntax is invalid.
    fn find(&self, selector: &str) -> PyResult<Option<PyTag>> {
        scrape_core::query::find_within(self.doc(), self.id, selector)
            .map_err(IntoPyErr::into_py_err)
            .map(|opt| opt.map(|id| PyTag::new(Arc::clone(&self.soup), id)))
    }

    /// Find all descendants matching a CSS selector.
    ///
    /// Args:
    ///     selector: CSS selector string.
    ///
    /// Returns:
    ///     List of matching Tag instances.
    ///
    /// Raises:
    ///     ValueError: If the selector syntax is invalid.
    fn find_all(&self, selector: &str) -> PyResult<Vec<PyTag>> {
        scrape_core::query::find_all_within(self.doc(), self.id, selector)
            .map_err(IntoPyErr::into_py_err)
            .map(|ids| ids.into_iter().map(|id| PyTag::new(Arc::clone(&self.soup), id)).collect())
    }

    /// Find all descendants matching a CSS selector (alias for find_all).
    fn select(&self, selector: &str) -> PyResult<Vec<PyTag>> {
        self.find_all(selector)
    }

    /// Find the first descendant using a pre-compiled selector.
    ///
    /// Args:
    ///     selector: A CompiledSelector instance.
    ///
    /// Returns:
    ///     The first matching Tag, or None if not found.
    fn find_compiled(&self, selector: &PyCompiledSelector) -> Option<PyTag> {
        scrape_core::query::find_within_compiled(self.doc(), self.id, &selector.inner)
            .map(|id| PyTag::new(Arc::clone(&self.soup), id))
    }

    /// Find all descendants using a pre-compiled selector.
    ///
    /// Args:
    ///     selector: A CompiledSelector instance.
    ///
    /// Returns:
    ///     List of matching Tag instances.
    fn select_compiled(&self, selector: &PyCompiledSelector) -> Vec<PyTag> {
        scrape_core::query::find_all_within_compiled(self.doc(), self.id, &selector.inner)
            .into_iter()
            .map(|id| PyTag::new(Arc::clone(&self.soup), id))
            .collect()
    }

    /// Get all text nodes in this element's subtree.
    ///
    /// Returns:
    ///     List of text content strings.
    fn text_nodes(&self) -> Vec<String> {
        scrape_core::query::TextNodesIter::new(self.doc(), self.id).map(String::from).collect()
    }

    /// Get child elements filtered by tag name.
    ///
    /// Args:
    ///     name: Tag name to filter by.
    ///
    /// Returns:
    ///     List of matching child Tag instances.
    fn children_by_name(&self, name: &str) -> Vec<PyTag> {
        self.doc()
            .children(self.id)
            .filter_map(|child_id| {
                let node = self.doc().get(child_id)?;
                if let NodeKind::Element { name: tag_name, .. } = &node.kind
                    && tag_name.eq_ignore_ascii_case(name)
                {
                    return Some(PyTag::new(Arc::clone(&self.soup), child_id));
                }
                None
            })
            .collect()
    }

    /// Get child elements filtered by class name.
    ///
    /// Args:
    ///     class_name: Class name to filter by.
    ///
    /// Returns:
    ///     List of matching child Tag instances.
    fn children_by_class(&self, class_name: &str) -> Vec<PyTag> {
        self.doc()
            .children(self.id)
            .filter_map(|child_id| {
                let node = self.doc().get(child_id)?;
                if let NodeKind::Element { attributes, .. } = &node.kind
                    && let Some(classes) = attributes.get("class")
                    && classes.split_whitespace().any(|c| c == class_name)
                {
                    return Some(PyTag::new(Arc::clone(&self.soup), child_id));
                }
                None
            })
            .collect()
    }

    /// Extract text content from all descendants matching a CSS selector.
    ///
    /// Args:
    ///     selector: CSS selector string.
    ///
    /// Returns:
    ///     List of text strings, one per matching element.
    ///
    /// Raises:
    ///     ValueError: If the selector syntax is invalid.
    fn select_text(&self, selector: &str) -> PyResult<Vec<String>> {
        scrape_core::query::select_text_within(self.doc(), self.id, selector)
            .map_err(IntoPyErr::into_py_err)
    }

    /// Extract attribute values from all descendants matching a CSS selector.
    ///
    /// Args:
    ///     selector: CSS selector string.
    ///     attr: Attribute name to extract.
    ///
    /// Returns:
    ///     List of attribute values (None for missing attributes).
    ///
    /// Raises:
    ///     ValueError: If the selector syntax is invalid.
    fn select_attr(&self, selector: &str, attr: &str) -> PyResult<Vec<Option<String>>> {
        scrape_core::query::select_attr_within(self.doc(), self.id, selector, attr)
            .map_err(IntoPyErr::into_py_err)
    }

    // ==================== Python Special Methods ====================

    fn __repr__(&self) -> String {
        let name = self.name().unwrap_or_else(|| "?".to_string());
        format!("Tag('{name}')")
    }

    fn __str__(&self) -> String {
        self.outer_html()
    }

    /// Get attribute value using dict-like access.
    fn __getitem__(&self, name: &str) -> PyResult<String> {
        self.get(name).ok_or_else(|| PyKeyError::new_err(format!("Attribute '{name}' not found")))
    }

    /// Check if attribute exists: "href" in tag
    fn __contains__(&self, name: &str) -> bool {
        self.has_attr(name)
    }

    /// Get number of child elements.
    fn __len__(&self) -> usize {
        self.doc()
            .children(self.id)
            .filter(|child_id| self.doc().get(*child_id).is_some_and(|n| n.kind.is_element()))
            .count()
    }

    /// Iterate over child elements.
    fn __iter__(&self) -> PyTagIterator {
        PyTagIterator { children: self.children(), index: 0 }
    }

    /// Compare two tags for equality (same document, same node).
    fn __eq__(&self, other: &PyTag) -> bool {
        Arc::ptr_eq(&self.soup, &other.soup) && self.id == other.id
    }

    fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        Arc::as_ptr(&self.soup).hash(&mut hasher);
        self.id.hash(&mut hasher);
        hasher.finish()
    }
}

impl Clone for PyTag {
    fn clone(&self) -> Self {
        Self { soup: Arc::clone(&self.soup), id: self.id }
    }
}

/// Iterator over child elements.
#[pyclass]
pub struct PyTagIterator {
    children: Vec<PyTag>,
    index: usize,
}

#[pymethods]
impl PyTagIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<PyTag> {
        if self.index < self.children.len() {
            let tag = self.children[self.index].clone();
            self.index += 1;
            Some(tag)
        } else {
            None
        }
    }
}

// ==================== Helper Functions ====================

// TODO: extract helper functions to scrape-core to reduce duplication
// These functions are duplicated in scrape-py, scrape-node, and scrape-wasm
// Consider adding to scrape-core::utils or making them public methods on Document

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

// FIXME duplicate code
fn serialize_node(doc: &Document, id: NodeId, result: &mut String) {
    let Some(node) = doc.get(id) else { return };

    match &node.kind {
        NodeKind::Element { name, attributes, .. } => {
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
