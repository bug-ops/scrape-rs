//! DOM node types and identifiers.

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
///
/// Nodes are linked via `first_child`/`last_child` for parent-child relationships
/// and `prev_sibling`/`next_sibling` for sibling relationships. This linked structure
/// eliminates per-node `Vec` allocations and enables O(1) append operations.
#[derive(Debug, Clone)]
pub struct Node {
    /// The kind of node (element, text, or comment).
    pub kind: NodeKind,
    /// Parent node, if any.
    pub parent: Option<NodeId>,
    /// First child node.
    pub first_child: Option<NodeId>,
    /// Last child node.
    pub last_child: Option<NodeId>,
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
            first_child: None,
            last_child: None,
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
            first_child: None,
            last_child: None,
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
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_equality() {
        let id1 = NodeId::new(42);
        let id2 = NodeId::new(42);
        let id3 = NodeId::new(43);
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn node_kind_element() {
        let kind = NodeKind::Element { name: "div".into(), attributes: HashMap::new() };
        assert!(kind.is_element());
        assert!(!kind.is_text());
        assert!(!kind.is_comment());
        assert_eq!(kind.as_element_name(), Some("div"));
    }

    #[test]
    fn node_kind_text() {
        let kind = NodeKind::Text { content: "Hello".into() };
        assert!(!kind.is_element());
        assert!(kind.is_text());
        assert!(!kind.is_comment());
        assert_eq!(kind.as_text(), Some("Hello"));
    }

    #[test]
    fn node_kind_comment() {
        let kind = NodeKind::Comment { content: "A comment".into() };
        assert!(!kind.is_element());
        assert!(!kind.is_text());
        assert!(kind.is_comment());
        assert_eq!(kind.as_comment(), Some("A comment"));
    }

    #[test]
    fn node_element_constructor() {
        let node = Node::element("div", HashMap::new());
        assert!(node.kind.is_element());
        assert!(node.parent.is_none());
        assert!(node.first_child.is_none());
        assert!(node.last_child.is_none());
        assert!(node.prev_sibling.is_none());
        assert!(node.next_sibling.is_none());
    }

    #[test]
    fn node_text_constructor() {
        let node = Node::text("Hello");
        assert!(node.kind.is_text());
        assert_eq!(node.kind.as_text(), Some("Hello"));
    }

    #[test]
    fn node_comment_constructor() {
        let node = Node::comment("A comment");
        assert!(node.kind.is_comment());
        assert_eq!(node.kind.as_comment(), Some("A comment"));
    }
}
