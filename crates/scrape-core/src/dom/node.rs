//! DOM node types and document container.

use std::collections::HashMap;

/// A node ID in the DOM tree.
///
/// This is an opaque handle to a node in the document.
/// The inner value is `pub(crate)` to allow internal indexing while
/// preventing external construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub(crate) usize);

impl NodeId {
    /// Creates a new node ID.
    #[must_use]
    pub(crate) const fn new(id: usize) -> Self {
        Self(id)
    }

    /// Returns the raw ID value (for internal use).
    #[must_use]
    pub(crate) const fn index(self) -> usize {
        self.0
    }
}

/// Types of nodes in the DOM tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    /// Element node (e.g., `<div>`, `<span>`).
    Element {
        /// Tag name (lowercase).
        name: String,
        /// Element attributes.
        attributes: HashMap<String, String>,
    },
    /// Text node.
    Text {
        /// Text content.
        content: String,
    },
    /// Comment node.
    Comment {
        /// Comment content.
        content: String,
    },
}

impl NodeKind {
    /// Returns the tag name if this is an element node.
    #[must_use]
    pub fn as_element_name(&self) -> Option<&str> {
        match self {
            Self::Element { name, .. } => Some(name),
            _ => None,
        }
    }

    /// Returns the text content if this is a text node.
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { content } => Some(content),
            _ => None,
        }
    }

    /// Returns the comment content if this is a comment node.
    #[must_use]
    pub fn as_comment(&self) -> Option<&str> {
        match self {
            Self::Comment { content } => Some(content),
            _ => None,
        }
    }

    /// Returns `true` if this is an element node.
    #[must_use]
    pub const fn is_element(&self) -> bool {
        matches!(self, Self::Element { .. })
    }

    /// Returns `true` if this is a text node.
    #[must_use]
    pub const fn is_text(&self) -> bool {
        matches!(self, Self::Text { .. })
    }

    /// Returns `true` if this is a comment node.
    #[must_use]
    pub const fn is_comment(&self) -> bool {
        matches!(self, Self::Comment { .. })
    }
}

/// A node in the DOM tree.
#[derive(Debug, Clone)]
pub struct Node {
    /// The kind of node (element, text, or comment).
    pub kind: NodeKind,
    /// Parent node, if any.
    pub parent: Option<NodeId>,
    /// Child nodes.
    pub children: Vec<NodeId>,
    /// Previous sibling.
    pub prev_sibling: Option<NodeId>,
    /// Next sibling.
    pub next_sibling: Option<NodeId>,
}

impl Node {
    /// Creates a new element node.
    #[must_use]
    pub fn element(name: impl Into<String>, attributes: HashMap<String, String>) -> Self {
        Self {
            kind: NodeKind::Element { name: name.into(), attributes },
            parent: None,
            children: Vec::new(),
            prev_sibling: None,
            next_sibling: None,
        }
    }

    /// Creates a new text node.
    #[must_use]
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            kind: NodeKind::Text { content: content.into() },
            parent: None,
            children: Vec::new(),
            prev_sibling: None,
            next_sibling: None,
        }
    }

    /// Creates a new comment node.
    #[must_use]
    pub fn comment(content: impl Into<String>) -> Self {
        Self {
            kind: NodeKind::Comment { content: content.into() },
            parent: None,
            children: Vec::new(),
            prev_sibling: None,
            next_sibling: None,
        }
    }
}

/// A parsed HTML document.
///
/// The document stores all nodes in a flat `Vec` and maintains
/// parent/child/sibling relationships through [`NodeId`] references.
#[derive(Debug)]
pub struct Document {
    nodes: Vec<Node>,
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
        Self { nodes: Vec::with_capacity(capacity), root: None }
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
        self.nodes.get(id.index())
    }

    /// Returns a mutable reference to the node with the given ID.
    #[must_use]
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id.index())
    }

    /// Creates a new element node and returns its ID.
    pub fn create_element(
        &mut self,
        name: impl Into<String>,
        attributes: HashMap<String, String>,
    ) -> NodeId {
        let id = NodeId::new(self.nodes.len());
        self.nodes.push(Node::element(name, attributes));
        id
    }

    /// Creates a new text node and returns its ID.
    pub fn create_text(&mut self, content: impl Into<String>) -> NodeId {
        let id = NodeId::new(self.nodes.len());
        self.nodes.push(Node::text(content));
        id
    }

    /// Creates a new comment node and returns its ID.
    pub fn create_comment(&mut self, content: impl Into<String>) -> NodeId {
        let id = NodeId::new(self.nodes.len());
        self.nodes.push(Node::comment(content));
        id
    }

    /// Appends a child node to a parent, maintaining sibling links.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `parent_id` or `child_id` refer to non-existent nodes.
    pub fn append_child(&mut self, parent_id: NodeId, child_id: NodeId) {
        debug_assert!(parent_id.index() < self.nodes.len(), "Invalid parent_id");
        debug_assert!(child_id.index() < self.nodes.len(), "Invalid child_id");

        // Update previous sibling link if parent has children
        if let Some(last_child_id) = self.nodes[parent_id.index()].children.last().copied() {
            self.nodes[last_child_id.index()].next_sibling = Some(child_id);
            self.nodes[child_id.index()].prev_sibling = Some(last_child_id);
        }

        self.nodes[child_id.index()].parent = Some(parent_id);
        self.nodes[parent_id.index()].children.push(child_id);
    }

    /// Returns the number of nodes in the document.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the document has no nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns an iterator over all nodes.
    pub fn nodes(&self) -> impl Iterator<Item = (NodeId, &Node)> {
        self.nodes.iter().enumerate().map(|(i, node)| (NodeId::new(i), node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_equality() {
        let id1 = NodeId::new(42);
        let id2 = NodeId::new(42);
        let id3 = NodeId::new(43);
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_node_kind_element() {
        let kind = NodeKind::Element { name: "div".into(), attributes: HashMap::new() };
        assert!(kind.is_element());
        assert!(!kind.is_text());
        assert!(!kind.is_comment());
        assert_eq!(kind.as_element_name(), Some("div"));
    }

    #[test]
    fn test_node_kind_text() {
        let kind = NodeKind::Text { content: "Hello".into() };
        assert!(!kind.is_element());
        assert!(kind.is_text());
        assert!(!kind.is_comment());
        assert_eq!(kind.as_text(), Some("Hello"));
    }

    #[test]
    fn test_node_kind_comment() {
        let kind = NodeKind::Comment { content: "A comment".into() };
        assert!(!kind.is_element());
        assert!(!kind.is_text());
        assert!(kind.is_comment());
        assert_eq!(kind.as_comment(), Some("A comment"));
    }

    #[test]
    fn test_document_create_element() {
        let mut doc = Document::new();
        let id = doc.create_element("div", HashMap::new());
        assert_eq!(doc.len(), 1);

        let node = doc.get(id).unwrap();
        assert!(node.kind.is_element());
        assert_eq!(node.kind.as_element_name(), Some("div"));
    }

    #[test]
    fn test_document_create_text() {
        let mut doc = Document::new();
        let id = doc.create_text("Hello World");

        let node = doc.get(id).unwrap();
        assert!(node.kind.is_text());
        assert_eq!(node.kind.as_text(), Some("Hello World"));
    }

    #[test]
    fn test_document_parent_child_relationship() {
        let mut doc = Document::new();
        let parent_id = doc.create_element("div", HashMap::new());
        let child_id = doc.create_element("span", HashMap::new());

        doc.append_child(parent_id, child_id);

        let parent = doc.get(parent_id).unwrap();
        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0], child_id);

        let child = doc.get(child_id).unwrap();
        assert_eq!(child.parent, Some(parent_id));
    }

    #[test]
    fn test_document_sibling_links() {
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
    fn test_document_root() {
        let mut doc = Document::new();
        assert!(doc.root().is_none());

        let root_id = doc.create_element("html", HashMap::new());
        doc.set_root(root_id);

        assert_eq!(doc.root(), Some(root_id));
    }
}
