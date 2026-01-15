//! Document container and tree operations.

use std::collections::HashMap;

use super::{
    arena::Arena,
    node::{Node, NodeId},
};

/// An HTML document containing a tree of nodes.
///
/// The document owns all nodes via an arena allocator, ensuring
/// cache-friendly contiguous storage. Navigation is performed
/// using [`NodeId`] handles.
///
/// # Architecture
///
/// Nodes are stored in a single contiguous `Arena<Node>`. Parent-child
/// relationships use `first_child`/`last_child` links (O(1) append),
/// and siblings are doubly linked via `prev_sibling`/`next_sibling`.
///
/// # Navigation
///
/// The document provides both direct navigation methods and lazy iterators:
///
/// - [`parent`](Document::parent), [`first_child`](Document::first_child),
///   [`last_child`](Document::last_child) - direct links
/// - [`children`](Document::children) - iterate over direct children
/// - [`ancestors`](Document::ancestors) - iterate from parent to root
/// - [`descendants`](Document::descendants) - depth-first subtree traversal
#[derive(Debug)]
pub struct Document {
    arena: Arena<Node>,
    root: Option<NodeId>,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    /// Creates a new empty document with default capacity.
    ///
    /// The default capacity is 256 nodes, which is sufficient for typical HTML pages
    /// and reduces reallocations during parsing.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(256)
    }

    /// Creates a new empty document with the specified capacity.
    ///
    /// Use this when you know the approximate number of nodes to avoid reallocations.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { arena: Arena::with_capacity(capacity), root: None }
    }

    /// Returns the root node ID, if any.
    #[must_use]
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// Sets the root node ID.
    pub fn set_root(&mut self, id: NodeId) {
        self.root = Some(id);
    }

    /// Returns a reference to the node with the given ID.
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.arena.get(id.index())
    }

    /// Returns a mutable reference to the node with the given ID.
    #[must_use]
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.arena.get_mut(id.index())
    }

    /// Creates a new element node and returns its ID.
    pub fn create_element(
        &mut self,
        name: impl Into<String>,
        attributes: HashMap<String, String>,
    ) -> NodeId {
        NodeId::new(self.arena.alloc(Node::element(name, attributes)))
    }

    /// Creates a new text node and returns its ID.
    pub fn create_text(&mut self, content: impl Into<String>) -> NodeId {
        NodeId::new(self.arena.alloc(Node::text(content)))
    }

    /// Creates a new comment node and returns its ID.
    pub fn create_comment(&mut self, content: impl Into<String>) -> NodeId {
        NodeId::new(self.arena.alloc(Node::comment(content)))
    }

    /// Appends a child node to a parent.
    ///
    /// Updates parent, `first_child`, `last_child`, and sibling links.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `parent_id` or `child_id` are invalid.
    pub fn append_child(&mut self, parent_id: NodeId, child_id: NodeId) {
        debug_assert!(parent_id.index() < self.arena.len(), "Invalid parent_id");
        debug_assert!(child_id.index() < self.arena.len(), "Invalid child_id");

        // Get current last child of parent
        let prev_last = self.arena.get(parent_id.index()).and_then(|p| p.last_child);

        // Update child's parent and prev_sibling
        if let Some(child) = self.arena.get_mut(child_id.index()) {
            child.parent = Some(parent_id);
            child.prev_sibling = prev_last;
            child.next_sibling = None;
        }

        // Update previous last child's next_sibling
        if let Some(prev_id) = prev_last
            && let Some(prev) = self.arena.get_mut(prev_id.index())
        {
            prev.next_sibling = Some(child_id);
        }

        // Update parent's first_child (if first) and last_child
        if let Some(parent) = self.arena.get_mut(parent_id.index()) {
            if parent.first_child.is_none() {
                parent.first_child = Some(child_id);
            }
            parent.last_child = Some(child_id);
        }
    }

    /// Returns the number of nodes in the document.
    #[must_use]
    pub fn len(&self) -> usize {
        self.arena.len()
    }

    /// Returns `true` if the document has no nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.arena.is_empty()
    }

    /// Returns an iterator over all nodes.
    pub fn nodes(&self) -> impl Iterator<Item = (NodeId, &Node)> {
        self.arena.iter().map(|(i, node)| (NodeId::new(i), node))
    }

    // ==================== Navigation APIs ====================

    /// Returns the parent of a node.
    #[must_use]
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.arena.get(id.index()).and_then(|n| n.parent)
    }

    /// Returns the first child of a node.
    #[must_use]
    pub fn first_child(&self, id: NodeId) -> Option<NodeId> {
        self.arena.get(id.index()).and_then(|n| n.first_child)
    }

    /// Returns the last child of a node.
    #[must_use]
    pub fn last_child(&self, id: NodeId) -> Option<NodeId> {
        self.arena.get(id.index()).and_then(|n| n.last_child)
    }

    /// Returns the next sibling of a node.
    #[must_use]
    pub fn next_sibling(&self, id: NodeId) -> Option<NodeId> {
        self.arena.get(id.index()).and_then(|n| n.next_sibling)
    }

    /// Returns the previous sibling of a node.
    #[must_use]
    pub fn prev_sibling(&self, id: NodeId) -> Option<NodeId> {
        self.arena.get(id.index()).and_then(|n| n.prev_sibling)
    }

    /// Returns an iterator over children of a node.
    ///
    /// The iterator yields children in order from first to last.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use scrape_core::dom::Document;
    ///
    /// let mut doc = Document::new();
    /// let parent = doc.create_element("div", HashMap::new());
    /// let child1 = doc.create_element("span", HashMap::new());
    /// let child2 = doc.create_element("span", HashMap::new());
    ///
    /// doc.append_child(parent, child1);
    /// doc.append_child(parent, child2);
    ///
    /// let children: Vec<_> = doc.children(parent).collect();
    /// assert_eq!(children.len(), 2);
    /// ```
    #[must_use]
    pub fn children(&self, id: NodeId) -> ChildrenIter<'_> {
        ChildrenIter { doc: self, current: self.first_child(id) }
    }

    /// Returns an iterator over ancestors of a node.
    ///
    /// The iterator yields ancestors from parent to root (does not include the node itself).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use scrape_core::dom::Document;
    ///
    /// let mut doc = Document::new();
    /// let grandparent = doc.create_element("html", HashMap::new());
    /// let parent = doc.create_element("body", HashMap::new());
    /// let child = doc.create_element("div", HashMap::new());
    ///
    /// doc.append_child(grandparent, parent);
    /// doc.append_child(parent, child);
    ///
    /// let ancestors: Vec<_> = doc.ancestors(child).collect();
    /// assert_eq!(ancestors.len(), 2); // parent, grandparent
    /// ```
    #[must_use]
    pub fn ancestors(&self, id: NodeId) -> AncestorsIter<'_> {
        AncestorsIter { doc: self, current: self.parent(id) }
    }

    /// Returns an iterator over descendants in depth-first pre-order.
    ///
    /// Does not include the starting node itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use scrape_core::dom::Document;
    ///
    /// let mut doc = Document::new();
    /// let root = doc.create_element("html", HashMap::new());
    /// let child1 = doc.create_element("head", HashMap::new());
    /// let child2 = doc.create_element("body", HashMap::new());
    /// let grandchild = doc.create_element("div", HashMap::new());
    ///
    /// doc.append_child(root, child1);
    /// doc.append_child(root, child2);
    /// doc.append_child(child2, grandchild);
    ///
    /// let descendants: Vec<_> = doc.descendants(root).collect();
    /// assert_eq!(descendants.len(), 3); // head, body, div
    /// ```
    #[must_use]
    pub fn descendants(&self, id: NodeId) -> DescendantsIter<'_> {
        DescendantsIter { doc: self, root: id, stack: vec![id], started: false }
    }
}

/// Iterator over direct children of a node.
///
/// Created by [`Document::children`].
#[derive(Debug)]
pub struct ChildrenIter<'a> {
    doc: &'a Document,
    current: Option<NodeId>,
}

impl Iterator for ChildrenIter<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = self.doc.next_sibling(current);
        Some(current)
    }
}

/// Iterator over ancestors of a node (parent, grandparent, ...).
///
/// Created by [`Document::ancestors`].
#[derive(Debug)]
pub struct AncestorsIter<'a> {
    doc: &'a Document,
    current: Option<NodeId>,
}

impl Iterator for AncestorsIter<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = self.doc.parent(current);
        Some(current)
    }
}

/// Iterator over descendants in depth-first pre-order.
///
/// Created by [`Document::descendants`].
#[derive(Debug)]
pub struct DescendantsIter<'a> {
    doc: &'a Document,
    root: NodeId,
    stack: Vec<NodeId>,
    started: bool,
}

impl Iterator for DescendantsIter<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.started {
            self.started = true;
            if let Some(first) = self.doc.first_child(self.root) {
                self.stack.clear();
                self.stack.push(first);
            } else {
                return None;
            }
        }

        let current = self.stack.pop()?;

        // Push next sibling first (so it's processed after children)
        if let Some(next) = self.doc.next_sibling(current) {
            self.stack.push(next);
        }

        // Push first child (will be processed next - depth-first)
        if let Some(child) = self.doc.first_child(current) {
            self.stack.push(child);
        }

        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_doc() -> Document {
        let mut doc = Document::new();

        let html = doc.create_element("html", HashMap::new());
        doc.set_root(html);

        let head = doc.create_element("head", HashMap::new());
        let body = doc.create_element("body", HashMap::new());
        let div = doc.create_element("div", HashMap::new());
        let text = doc.create_text("Hello");

        doc.append_child(html, head);
        doc.append_child(html, body);
        doc.append_child(body, div);
        doc.append_child(div, text);

        doc
    }

    #[test]
    fn document_create_element() {
        let mut doc = Document::new();
        let id = doc.create_element("div", HashMap::new());
        assert_eq!(doc.len(), 1);

        let node = doc.get(id).unwrap();
        assert!(node.kind.is_element());
        assert_eq!(node.kind.as_element_name(), Some("div"));
    }

    #[test]
    fn document_create_text() {
        let mut doc = Document::new();
        let id = doc.create_text("Hello World");

        let node = doc.get(id).unwrap();
        assert!(node.kind.is_text());
        assert_eq!(node.kind.as_text(), Some("Hello World"));
    }

    #[test]
    fn document_root() {
        let mut doc = Document::new();
        assert!(doc.root().is_none());

        let root_id = doc.create_element("html", HashMap::new());
        doc.set_root(root_id);

        assert_eq!(doc.root(), Some(root_id));
    }

    #[test]
    fn parent_navigation() {
        let doc = create_test_doc();
        let root = doc.root().unwrap();

        let body = doc.children(root).nth(1).unwrap();
        assert_eq!(doc.parent(body), Some(root));
    }

    #[test]
    fn children_iteration() {
        let doc = create_test_doc();
        let root = doc.root().unwrap();

        assert_eq!(doc.children(root).count(), 2);
    }

    #[test]
    fn sibling_navigation() {
        let doc = create_test_doc();
        let root = doc.root().unwrap();

        let head = doc.first_child(root).unwrap();
        let body = doc.next_sibling(head).unwrap();

        assert_eq!(doc.prev_sibling(body), Some(head));
        assert!(doc.next_sibling(body).is_none());
    }

    #[test]
    fn descendants_iteration() {
        let doc = create_test_doc();
        let root = doc.root().unwrap();

        assert_eq!(doc.descendants(root).count(), 4);
    }

    #[test]
    fn ancestors_iteration() {
        let doc = create_test_doc();
        let root = doc.root().unwrap();

        let body = doc.children(root).nth(1).unwrap();
        let div = doc.first_child(body).unwrap();
        let text = doc.first_child(div).unwrap();

        assert_eq!(doc.ancestors(text).count(), 3);
    }

    #[test]
    fn first_and_last_child() {
        let doc = create_test_doc();
        let root = doc.root().unwrap();

        let first = doc.first_child(root).unwrap();
        let last = doc.last_child(root).unwrap();

        assert_eq!(doc.get(first).unwrap().kind.as_element_name(), Some("head"));
        assert_eq!(doc.get(last).unwrap().kind.as_element_name(), Some("body"));
    }

    #[test]
    fn children_empty_for_leaf_nodes() {
        let doc = create_test_doc();
        let root = doc.root().unwrap();

        let body = doc.children(root).nth(1).unwrap();
        let div = doc.first_child(body).unwrap();
        let text = doc.first_child(div).unwrap();

        assert!(doc.children(text).next().is_none());
    }

    #[test]
    fn descendants_empty_for_leaf_nodes() {
        let doc = create_test_doc();
        let root = doc.root().unwrap();

        let body = doc.children(root).nth(1).unwrap();
        let div = doc.first_child(body).unwrap();
        let text = doc.first_child(div).unwrap();

        assert!(doc.descendants(text).next().is_none());
    }

    #[test]
    fn ancestors_empty_for_root() {
        let doc = create_test_doc();
        let root = doc.root().unwrap();

        assert!(doc.ancestors(root).next().is_none());
    }

    #[test]
    fn document_parent_child_relationship() {
        let mut doc = Document::new();
        let parent_id = doc.create_element("div", HashMap::new());
        let child_id = doc.create_element("span", HashMap::new());

        doc.append_child(parent_id, child_id);

        let parent = doc.get(parent_id).unwrap();
        assert_eq!(parent.first_child, Some(child_id));
        assert_eq!(parent.last_child, Some(child_id));

        let child = doc.get(child_id).unwrap();
        assert_eq!(child.parent, Some(parent_id));
    }

    #[test]
    fn document_sibling_links() {
        let mut doc = Document::new();
        let parent_id = doc.create_element("div", HashMap::new());
        let child1_id = doc.create_element("span", HashMap::new());
        let child2_id = doc.create_element("span", HashMap::new());
        let child3_id = doc.create_element("span", HashMap::new());

        doc.append_child(parent_id, child1_id);
        doc.append_child(parent_id, child2_id);
        doc.append_child(parent_id, child3_id);

        let child1 = doc.get(child1_id).unwrap();
        assert_eq!(child1.prev_sibling, None);
        assert_eq!(child1.next_sibling, Some(child2_id));

        let child2 = doc.get(child2_id).unwrap();
        assert_eq!(child2.prev_sibling, Some(child1_id));
        assert_eq!(child2.next_sibling, Some(child3_id));

        let child3 = doc.get(child3_id).unwrap();
        assert_eq!(child3.prev_sibling, Some(child2_id));
        assert_eq!(child3.next_sibling, None);
    }

    #[test]
    fn descendants_order_depth_first() {
        let mut doc = Document::new();
        let root = doc.create_element("root", HashMap::new());
        let a = doc.create_element("a", HashMap::new());
        let b = doc.create_element("b", HashMap::new());
        let a1 = doc.create_element("a1", HashMap::new());
        let a2 = doc.create_element("a2", HashMap::new());

        doc.set_root(root);
        doc.append_child(root, a);
        doc.append_child(root, b);
        doc.append_child(a, a1);
        doc.append_child(a, a2);

        let names: Vec<_> = doc
            .descendants(root)
            .map(|id| doc.get(id).unwrap().kind.as_element_name().unwrap())
            .collect();

        // Depth-first pre-order: a -> a1 -> a2 -> b
        assert_eq!(names, vec!["a", "a1", "a2", "b"]);
    }
}
