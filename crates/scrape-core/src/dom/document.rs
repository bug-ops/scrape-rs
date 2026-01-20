//! Document container and tree operations.

use std::{collections::HashMap, marker::PhantomData};

use super::{
    arena::Arena,
    index::DocumentIndex,
    node::{Node, NodeId},
    state::{Building, DocumentState, MutableState, Queryable, QueryableState, Sealed},
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
///
/// # Typestate
///
/// Internally, the document uses typestate pattern to enforce lifecycle
/// guarantees at compile time. The type parameter `S` tracks whether
/// the document is being built, is queryable, or is sealed.
#[derive(Debug)]
pub struct DocumentImpl<S: DocumentState = Queryable> {
    arena: Arena<Node>,
    root: Option<NodeId>,
    index: Option<DocumentIndex>,
    _state: PhantomData<S>,
}

/// Public alias for backward compatibility.
///
/// The public `Document` type always refers to a queryable document.
/// Internally, we use `DocumentImpl<S>` for typestate enforcement.
pub type Document = DocumentImpl<Queryable>;

// ==================== Default Implementations ====================

impl Default for DocumentImpl<Building> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Document {
    fn default() -> Self {
        DocumentImpl::<Building>::new().build()
    }
}

// ==================== Building State ====================

impl DocumentImpl<Building> {
    /// Creates a new empty document in building state.
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
        Self { arena: Arena::with_capacity(capacity), root: None, index: None, _state: PhantomData }
    }

    /// Sets the root node ID.
    pub fn set_root(&mut self, id: NodeId) {
        self.root = Some(id);
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

    /// Transitions the document from Building to Queryable state.
    ///
    /// This is a one-way transition. Once built, the document structure
    /// cannot be modified.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use scrape_core::{Building, DocumentImpl};
    ///
    /// let mut doc = DocumentImpl::<Building>::new();
    /// let root = doc.create_element("div", HashMap::new());
    /// doc.set_root(root);
    ///
    /// // Transition to queryable state
    /// let doc = doc.build();
    /// ```
    #[must_use]
    pub fn build(self) -> DocumentImpl<Queryable> {
        DocumentImpl { arena: self.arena, root: self.root, index: self.index, _state: PhantomData }
    }
}

// ==================== Queryable State ====================

impl DocumentImpl<Queryable> {
    /// Creates a new empty document in queryable state.
    ///
    /// This is a convenience method for backward compatibility.
    /// Internally, it creates a Building document and immediately transitions to Queryable.
    #[must_use]
    pub fn new() -> Self {
        DocumentImpl::<Building>::new().build()
    }

    /// Creates a new empty document with the specified capacity in queryable state.
    ///
    /// This is a convenience method for backward compatibility.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        DocumentImpl::<Building>::with_capacity(capacity).build()
    }

    /// Sets the root node ID.
    ///
    /// Available on Queryable for backward compatibility with tests.
    pub fn set_root(&mut self, id: NodeId) {
        self.root = Some(id);
    }

    /// Creates a new element node and returns its ID.
    ///
    /// Available on Queryable for backward compatibility with tests.
    pub fn create_element(
        &mut self,
        name: impl Into<String>,
        attributes: HashMap<String, String>,
    ) -> NodeId {
        NodeId::new(self.arena.alloc(Node::element(name, attributes)))
    }

    /// Creates a new text node and returns its ID.
    ///
    /// Available on Queryable for backward compatibility with tests.
    pub fn create_text(&mut self, content: impl Into<String>) -> NodeId {
        NodeId::new(self.arena.alloc(Node::text(content)))
    }

    /// Creates a new comment node and returns its ID.
    ///
    /// Available on Queryable for backward compatibility with tests.
    pub fn create_comment(&mut self, content: impl Into<String>) -> NodeId {
        NodeId::new(self.arena.alloc(Node::comment(content)))
    }

    /// Appends a child node to a parent.
    ///
    /// Available on Queryable for backward compatibility with tests.
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

    /// Seals the document, preventing any future modifications.
    ///
    /// This is a one-way transition for when you need to guarantee
    /// the document will never change.
    #[must_use]
    pub fn seal(self) -> DocumentImpl<Sealed> {
        DocumentImpl { arena: self.arena, root: self.root, index: self.index, _state: PhantomData }
    }

    /// Sets the document index.
    ///
    /// Only available in Queryable state since index would be invalidated
    /// by structural modifications.
    pub fn set_index(&mut self, index: DocumentIndex) {
        self.index = Some(index);
    }
}

// ==================== Shared Methods (All States) ====================

impl<S: DocumentState> DocumentImpl<S> {
    /// Returns the root node ID, if any.
    #[must_use]
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// Returns a reference to the node with the given ID.
    #[inline]
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.arena.get(id.index())
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
}

// ==================== Mutable State Methods ====================

impl<S: MutableState> DocumentImpl<S> {
    /// Returns a mutable reference to the node with the given ID.
    ///
    /// Only available for documents in mutable states (Building).
    #[inline]
    #[must_use]
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.arena.get_mut(id.index())
    }
}

// ==================== Queryable State Methods ====================

impl<S: QueryableState> DocumentImpl<S> {
    /// Returns the document index, if built.
    ///
    /// Only available in queryable states (Queryable, Sealed).
    #[must_use]
    pub fn index(&self) -> Option<&DocumentIndex> {
        self.index.as_ref()
    }
}

// ==================== Navigation APIs ====================

impl<S: DocumentState> DocumentImpl<S> {
    // ==================== Navigation APIs ====================

    /// Returns the parent of a node.
    #[inline]
    #[must_use]
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.arena.get(id.index()).and_then(|n| n.parent)
    }

    /// Returns the first child of a node.
    #[inline]
    #[must_use]
    pub fn first_child(&self, id: NodeId) -> Option<NodeId> {
        self.arena.get(id.index()).and_then(|n| n.first_child)
    }

    /// Returns the last child of a node.
    #[inline]
    #[must_use]
    pub fn last_child(&self, id: NodeId) -> Option<NodeId> {
        self.arena.get(id.index()).and_then(|n| n.last_child)
    }

    /// Returns the next sibling of a node.
    #[inline]
    #[must_use]
    pub fn next_sibling(&self, id: NodeId) -> Option<NodeId> {
        self.arena.get(id.index()).and_then(|n| n.next_sibling)
    }

    /// Returns the previous sibling of a node.
    #[inline]
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
    /// use scrape_core::Document;
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
    pub fn children(&self, id: NodeId) -> ChildrenIter<'_, S> {
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
    /// use scrape_core::Document;
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
    pub fn ancestors(&self, id: NodeId) -> AncestorsIter<'_, S> {
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
    /// use scrape_core::Document;
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
    pub fn descendants(&self, id: NodeId) -> DescendantsIter<'_, S> {
        DescendantsIter { doc: self, root: id, stack: vec![id], started: false }
    }

    /// Returns an iterator over siblings following a node.
    ///
    /// Does not include the node itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use scrape_core::Document;
    ///
    /// let mut doc = Document::new();
    /// let parent = doc.create_element("ul", HashMap::new());
    /// let child1 = doc.create_element("li", HashMap::new());
    /// let child2 = doc.create_element("li", HashMap::new());
    /// let child3 = doc.create_element("li", HashMap::new());
    ///
    /// doc.append_child(parent, child1);
    /// doc.append_child(parent, child2);
    /// doc.append_child(parent, child3);
    ///
    /// let next: Vec<_> = doc.next_siblings(child1).collect();
    /// assert_eq!(next.len(), 2); // child2, child3
    /// ```
    #[must_use]
    pub fn next_siblings(&self, id: NodeId) -> NextSiblingsIter<'_, S> {
        NextSiblingsIter { doc: self, current: self.next_sibling(id) }
    }

    /// Returns an iterator over siblings preceding a node.
    ///
    /// Does not include the node itself. Iterates in reverse order
    /// (from immediate predecessor toward first sibling).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use scrape_core::Document;
    ///
    /// let mut doc = Document::new();
    /// let parent = doc.create_element("ul", HashMap::new());
    /// let child1 = doc.create_element("li", HashMap::new());
    /// let child2 = doc.create_element("li", HashMap::new());
    /// let child3 = doc.create_element("li", HashMap::new());
    ///
    /// doc.append_child(parent, child1);
    /// doc.append_child(parent, child2);
    /// doc.append_child(parent, child3);
    ///
    /// let prev: Vec<_> = doc.prev_siblings(child3).collect();
    /// assert_eq!(prev.len(), 2); // child2, child1 (reverse order)
    /// ```
    #[must_use]
    pub fn prev_siblings(&self, id: NodeId) -> PrevSiblingsIter<'_, S> {
        PrevSiblingsIter { doc: self, current: self.prev_sibling(id) }
    }

    /// Returns an iterator over all siblings of a node (excluding the node itself).
    ///
    /// Iterates in document order from first sibling to last.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use scrape_core::Document;
    ///
    /// let mut doc = Document::new();
    /// let parent = doc.create_element("ul", HashMap::new());
    /// let child1 = doc.create_element("li", HashMap::new());
    /// let child2 = doc.create_element("li", HashMap::new());
    /// let child3 = doc.create_element("li", HashMap::new());
    ///
    /// doc.append_child(parent, child1);
    /// doc.append_child(parent, child2);
    /// doc.append_child(parent, child3);
    ///
    /// let siblings: Vec<_> = doc.siblings(child2).collect();
    /// assert_eq!(siblings.len(), 2); // child1, child3
    /// ```
    #[must_use]
    pub fn siblings(&self, id: NodeId) -> SiblingsIter<'_, S> {
        // Find first sibling by going to parent's first child
        let first = self.parent(id).and_then(|p| self.first_child(p));
        SiblingsIter { doc: self, current: first, exclude: id }
    }
}

/// Iterator over direct children of a node.
///
/// Created by [`Document::children`].
#[derive(Debug)]
pub struct ChildrenIter<'a, S: DocumentState = Queryable> {
    doc: &'a DocumentImpl<S>,
    current: Option<NodeId>,
}

impl<S: DocumentState> Iterator for ChildrenIter<'_, S> {
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
pub struct AncestorsIter<'a, S: DocumentState = Queryable> {
    doc: &'a DocumentImpl<S>,
    current: Option<NodeId>,
}

impl<S: DocumentState> Iterator for AncestorsIter<'_, S> {
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
pub struct DescendantsIter<'a, S: DocumentState = Queryable> {
    doc: &'a DocumentImpl<S>,
    root: NodeId,
    stack: Vec<NodeId>,
    started: bool,
}

impl<S: DocumentState> Iterator for DescendantsIter<'_, S> {
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

/// Iterator over siblings following a node.
///
/// Created by [`Document::next_siblings`].
#[derive(Debug)]
pub struct NextSiblingsIter<'a, S: DocumentState = Queryable> {
    doc: &'a DocumentImpl<S>,
    current: Option<NodeId>,
}

impl<S: DocumentState> Iterator for NextSiblingsIter<'_, S> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = self.doc.next_sibling(current);
        Some(current)
    }
}

/// Iterator over siblings preceding a node.
///
/// Created by [`Document::prev_siblings`].
#[derive(Debug)]
pub struct PrevSiblingsIter<'a, S: DocumentState = Queryable> {
    doc: &'a DocumentImpl<S>,
    current: Option<NodeId>,
}

impl<S: DocumentState> Iterator for PrevSiblingsIter<'_, S> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = self.doc.prev_sibling(current);
        Some(current)
    }
}

/// Iterator over all siblings of a node (excluding the node itself).
///
/// Created by [`Document::siblings`].
/// Iterates in document order (from first sibling to last).
#[derive(Debug)]
pub struct SiblingsIter<'a, S: DocumentState = Queryable> {
    doc: &'a DocumentImpl<S>,
    current: Option<NodeId>,
    exclude: NodeId,
}

impl<S: DocumentState> Iterator for SiblingsIter<'_, S> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current = self.current?;
            self.current = self.doc.next_sibling(current);
            if current != self.exclude {
                return Some(current);
            }
        }
    }
}

// ==================== Iterator Extensions ====================

impl<'a, S: DocumentState> ChildrenIter<'a, S> {
    /// Returns an iterator over only element children (skipping text/comment nodes).
    ///
    /// This is more efficient than using `.filter()` externally because
    /// it avoids closure overhead and can be specialized.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div>text<span>A</span>more<span>B</span></div>");
    /// let div = soup.find("div").unwrap().unwrap();
    /// let doc = soup.document();
    ///
    /// // Only elements, skipping text nodes
    /// let count = doc.children(div.node_id()).elements().count();
    /// assert_eq!(count, 2); // span, span
    /// ```
    #[must_use]
    pub fn elements(self) -> ElementChildrenIter<'a, S> {
        ElementChildrenIter { inner: self }
    }
}

/// Iterator over element children only.
///
/// Created by [`ChildrenIter::elements`].
#[derive(Debug)]
pub struct ElementChildrenIter<'a, S: DocumentState = Queryable> {
    inner: ChildrenIter<'a, S>,
}

impl<S: DocumentState> Iterator for ElementChildrenIter<'_, S> {
    type Item = NodeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let id = self.inner.next()?;
            if self.inner.doc.get(id).is_some_and(|n| n.kind.is_element()) {
                return Some(id);
            }
        }
    }
}

impl<'a, S: DocumentState> DescendantsIter<'a, S> {
    /// Returns an iterator over only element descendants (skipping text/comment nodes).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div>text<p>para</p></div>");
    /// let div = soup.find("div").unwrap().unwrap();
    /// let doc = soup.document();
    ///
    /// let count = doc.descendants(div.node_id()).elements().count();
    /// assert_eq!(count, 1); // p only
    /// ```
    #[must_use]
    pub fn elements(self) -> ElementDescendantsIter<'a, S> {
        ElementDescendantsIter { inner: self }
    }
}

/// Iterator over element descendants only.
///
/// Created by [`DescendantsIter::elements`].
#[derive(Debug)]
pub struct ElementDescendantsIter<'a, S: DocumentState = Queryable> {
    inner: DescendantsIter<'a, S>,
}

impl<S: DocumentState> Iterator for ElementDescendantsIter<'_, S> {
    type Item = NodeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let id = self.inner.next()?;
            if self.inner.doc.get(id).is_some_and(|n| n.kind.is_element()) {
                return Some(id);
            }
        }
    }
}

impl<'a, S: DocumentState> AncestorsIter<'a, S> {
    /// Returns an iterator over only element ancestors (skipping non-element nodes).
    ///
    /// In practice, ancestors are almost always elements, but this method
    /// provides consistent API with other iterators.
    #[must_use]
    pub fn elements(self) -> ElementAncestorsIter<'a, S> {
        ElementAncestorsIter { inner: self }
    }
}

/// Iterator over element ancestors only.
///
/// Created by [`AncestorsIter::elements`].
#[derive(Debug)]
pub struct ElementAncestorsIter<'a, S: DocumentState = Queryable> {
    inner: AncestorsIter<'a, S>,
}

impl<S: DocumentState> Iterator for ElementAncestorsIter<'_, S> {
    type Item = NodeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let id = self.inner.next()?;
            if self.inner.doc.get(id).is_some_and(|n| n.kind.is_element()) {
                return Some(id);
            }
        }
    }
}

impl<'a, S: DocumentState> NextSiblingsIter<'a, S> {
    /// Returns an iterator over only element siblings (skipping text/comment nodes).
    #[must_use]
    pub fn elements(self) -> ElementNextSiblingsIter<'a, S> {
        ElementNextSiblingsIter { inner: self }
    }
}

/// Iterator over next element siblings only.
///
/// Created by [`NextSiblingsIter::elements`].
#[derive(Debug)]
pub struct ElementNextSiblingsIter<'a, S: DocumentState = Queryable> {
    inner: NextSiblingsIter<'a, S>,
}

impl<S: DocumentState> Iterator for ElementNextSiblingsIter<'_, S> {
    type Item = NodeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let id = self.inner.next()?;
            if self.inner.doc.get(id).is_some_and(|n| n.kind.is_element()) {
                return Some(id);
            }
        }
    }
}

impl<'a, S: DocumentState> PrevSiblingsIter<'a, S> {
    /// Returns an iterator over only element siblings (skipping text/comment nodes).
    #[must_use]
    pub fn elements(self) -> ElementPrevSiblingsIter<'a, S> {
        ElementPrevSiblingsIter { inner: self }
    }
}

/// Iterator over previous element siblings only.
///
/// Created by [`PrevSiblingsIter::elements`].
#[derive(Debug)]
pub struct ElementPrevSiblingsIter<'a, S: DocumentState = Queryable> {
    inner: PrevSiblingsIter<'a, S>,
}

impl<S: DocumentState> Iterator for ElementPrevSiblingsIter<'_, S> {
    type Item = NodeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let id = self.inner.next()?;
            if self.inner.doc.get(id).is_some_and(|n| n.kind.is_element()) {
                return Some(id);
            }
        }
    }
}

impl<'a, S: DocumentState> SiblingsIter<'a, S> {
    /// Returns an iterator over only element siblings (skipping text/comment nodes).
    #[must_use]
    pub fn elements(self) -> ElementSiblingsIter<'a, S> {
        ElementSiblingsIter { inner: self }
    }
}

/// Iterator over all element siblings (excluding self).
///
/// Created by [`SiblingsIter::elements`].
#[derive(Debug)]
pub struct ElementSiblingsIter<'a, S: DocumentState = Queryable> {
    inner: SiblingsIter<'a, S>,
}

impl<S: DocumentState> Iterator for ElementSiblingsIter<'_, S> {
    type Item = NodeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let id = self.inner.next()?;
            if self.inner.doc.get(id).is_some_and(|n| n.kind.is_element()) {
                return Some(id);
            }
        }
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
        assert_eq!(node.kind.tag_name(), Some("div"));
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

        assert_eq!(doc.get(first).unwrap().kind.tag_name(), Some("head"));
        assert_eq!(doc.get(last).unwrap().kind.tag_name(), Some("body"));
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

        let names: Vec<_> =
            doc.descendants(root).map(|id| doc.get(id).unwrap().kind.tag_name().unwrap()).collect();

        // Depth-first pre-order: a -> a1 -> a2 -> b
        assert_eq!(names, vec!["a", "a1", "a2", "b"]);
    }

    #[test]
    fn next_siblings_iteration() {
        let doc = create_test_doc();
        let root = doc.root().unwrap();
        let head = doc.first_child(root).unwrap();

        assert_eq!(doc.next_siblings(head).count(), 1); // body
    }

    #[test]
    fn next_siblings_empty_for_last() {
        let doc = create_test_doc();
        let root = doc.root().unwrap();
        let body = doc.last_child(root).unwrap();

        assert!(doc.next_siblings(body).next().is_none());
    }

    #[test]
    fn prev_siblings_iteration() {
        let doc = create_test_doc();
        let root = doc.root().unwrap();
        let body = doc.last_child(root).unwrap();

        assert_eq!(doc.prev_siblings(body).count(), 1); // head
    }

    #[test]
    fn prev_siblings_empty_for_first() {
        let doc = create_test_doc();
        let root = doc.root().unwrap();
        let head = doc.first_child(root).unwrap();

        assert!(doc.prev_siblings(head).next().is_none());
    }

    #[test]
    fn siblings_iteration() {
        let mut doc = Document::new();
        let parent = doc.create_element("ul", HashMap::new());
        let child1 = doc.create_element("li", HashMap::new());
        let child2 = doc.create_element("li", HashMap::new());
        let child3 = doc.create_element("li", HashMap::new());

        doc.append_child(parent, child1);
        doc.append_child(parent, child2);
        doc.append_child(parent, child3);

        let siblings: Vec<_> = doc.siblings(child2).collect();
        assert_eq!(siblings.len(), 2);
        assert_eq!(siblings[0], child1); // Document order
        assert_eq!(siblings[1], child3);
    }

    #[test]
    fn siblings_empty_for_only_child() {
        let mut doc = Document::new();
        let parent = doc.create_element("div", HashMap::new());
        let child = doc.create_element("span", HashMap::new());

        doc.append_child(parent, child);

        assert!(doc.siblings(child).next().is_none());
    }

    #[test]
    fn siblings_excludes_self() {
        let mut doc = Document::new();
        let parent = doc.create_element("ul", HashMap::new());
        let child1 = doc.create_element("li", HashMap::new());
        let child2 = doc.create_element("li", HashMap::new());

        doc.append_child(parent, child1);
        doc.append_child(parent, child2);

        let siblings1: Vec<_> = doc.siblings(child1).collect();
        assert_eq!(siblings1.len(), 1);
        assert_eq!(siblings1[0], child2);

        let siblings2: Vec<_> = doc.siblings(child2).collect();
        assert_eq!(siblings2.len(), 1);
        assert_eq!(siblings2[0], child1);
    }

    #[test]
    fn test_children_elements() {
        let mut doc = Document::new();
        let parent = doc.create_element("div", HashMap::new());
        let text = doc.create_text("text");
        let child1 = doc.create_element("span", HashMap::new());
        let child2 = doc.create_element("p", HashMap::new());

        doc.append_child(parent, text);
        doc.append_child(parent, child1);
        doc.append_child(parent, child2);

        let mut iter = doc.children(parent).elements();
        assert_eq!(iter.next(), Some(child1));
        assert_eq!(iter.next(), Some(child2));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_descendants_elements() {
        let mut doc = Document::new();
        let root = doc.create_element("div", HashMap::new());
        let text = doc.create_text("text");
        let child = doc.create_element("span", HashMap::new());
        let grandchild = doc.create_element("b", HashMap::new());

        doc.append_child(root, text);
        doc.append_child(root, child);
        doc.append_child(child, grandchild);

        assert_eq!(doc.descendants(root).elements().count(), 2); // span, b
    }

    #[test]
    fn test_ancestors_elements() {
        let mut doc = Document::new();
        let root = doc.create_element("html", HashMap::new());
        let body = doc.create_element("body", HashMap::new());
        let div = doc.create_element("div", HashMap::new());

        doc.set_root(root);
        doc.append_child(root, body);
        doc.append_child(body, div);

        assert_eq!(doc.ancestors(div).elements().count(), 2);
    }

    #[test]
    fn test_next_siblings_elements() {
        let mut doc = Document::new();
        let parent = doc.create_element("ul", HashMap::new());
        let li1 = doc.create_element("li", HashMap::new());
        let text = doc.create_text(" ");
        let li2 = doc.create_element("li", HashMap::new());

        doc.append_child(parent, li1);
        doc.append_child(parent, text);
        doc.append_child(parent, li2);

        let siblings: Vec<_> = doc.next_siblings(li1).elements().collect();
        assert_eq!(siblings.len(), 1);
        assert_eq!(siblings[0], li2);
    }

    #[test]
    fn test_prev_siblings_elements() {
        let mut doc = Document::new();
        let parent = doc.create_element("ul", HashMap::new());
        let li1 = doc.create_element("li", HashMap::new());
        let text = doc.create_text(" ");
        let li2 = doc.create_element("li", HashMap::new());

        doc.append_child(parent, li1);
        doc.append_child(parent, text);
        doc.append_child(parent, li2);

        let siblings: Vec<_> = doc.prev_siblings(li2).elements().collect();
        assert_eq!(siblings.len(), 1);
        assert_eq!(siblings[0], li1);
    }

    #[test]
    fn test_siblings_elements() {
        let mut doc = Document::new();
        let parent = doc.create_element("ul", HashMap::new());
        let li1 = doc.create_element("li", HashMap::new());
        let text = doc.create_text(" ");
        let li2 = doc.create_element("li", HashMap::new());
        let li3 = doc.create_element("li", HashMap::new());

        doc.append_child(parent, li1);
        doc.append_child(parent, text);
        doc.append_child(parent, li2);
        doc.append_child(parent, li3);

        assert_eq!(doc.siblings(li2).elements().count(), 2); // li1, li3
    }
}
