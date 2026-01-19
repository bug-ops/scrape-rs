//! Document indexing for fast element lookup.

use std::collections::HashMap;

use super::NodeId;

/// Index for fast element lookup by ID and class.
#[derive(Debug, Default, Clone)]
pub struct DocumentIndex {
    by_id: HashMap<String, NodeId>,
    by_class: HashMap<String, Vec<NodeId>>,
}

impl DocumentIndex {
    /// Creates a new empty index.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an element's ID.
    ///
    /// Per HTML spec, first occurrence wins if duplicate IDs exist.
    pub fn register_id(&mut self, id: String, node_id: NodeId) {
        self.by_id.entry(id).or_insert(node_id);
    }

    /// Registers an element's classes.
    pub fn register_classes(&mut self, classes: &str, node_id: NodeId) {
        for class in classes.split_whitespace() {
            self.by_class.entry(class.to_string()).or_default().push(node_id);
        }
    }

    /// Looks up an element by ID.
    #[must_use]
    pub fn get_by_id(&self, id: &str) -> Option<NodeId> {
        self.by_id.get(id).copied()
    }

    /// Looks up elements by class.
    #[must_use]
    pub fn get_by_class(&self, class: &str) -> &[NodeId] {
        self.by_class.get(class).map_or(&[], Vec::as_slice)
    }

    /// Returns whether the index is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty() && self.by_class.is_empty()
    }

    /// Returns the number of indexed IDs.
    #[must_use]
    pub fn id_count(&self) -> usize {
        self.by_id.len()
    }

    /// Returns the number of indexed classes.
    #[must_use]
    pub fn class_count(&self) -> usize {
        self.by_class.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_register_id() {
        let mut index = DocumentIndex::new();
        let node1 = NodeId::new(1);
        let node2 = NodeId::new(2);

        index.register_id("main".to_string(), node1);
        assert_eq!(index.get_by_id("main"), Some(node1));

        index.register_id("main".to_string(), node2);
        assert_eq!(index.get_by_id("main"), Some(node1));
    }

    #[test]
    fn test_index_register_classes() {
        let mut index = DocumentIndex::new();
        let node1 = NodeId::new(1);
        let node2 = NodeId::new(2);

        index.register_classes("foo bar", node1);
        index.register_classes("bar baz", node2);

        assert_eq!(index.get_by_class("foo"), &[node1]);
        assert_eq!(index.get_by_class("bar"), &[node1, node2]);
        assert_eq!(index.get_by_class("baz"), &[node2]);
        assert_eq!(index.get_by_class("qux"), &[]);
    }

    #[test]
    fn test_index_empty() {
        let index = DocumentIndex::new();
        assert!(index.is_empty());

        let mut index = DocumentIndex::new();
        index.register_id("test".to_string(), NodeId::new(1));
        assert!(!index.is_empty());
    }

    #[test]
    fn test_index_counts() {
        let mut index = DocumentIndex::new();
        assert_eq!(index.id_count(), 0);
        assert_eq!(index.class_count(), 0);

        index.register_id("id1".to_string(), NodeId::new(1));
        index.register_id("id2".to_string(), NodeId::new(2));
        index.register_classes("class1 class2", NodeId::new(3));

        assert_eq!(index.id_count(), 2);
        assert_eq!(index.class_count(), 2);
    }

    #[test]
    fn test_index_empty_class_string() {
        let mut index = DocumentIndex::new();
        let node = NodeId::new(1);

        index.register_classes("", node);
        assert_eq!(index.class_count(), 0);
        assert!(index.is_empty());
    }

    #[test]
    fn test_index_whitespace_only_class() {
        let mut index = DocumentIndex::new();
        let node = NodeId::new(1);

        index.register_classes("   ", node);
        index.register_classes("\t\n", node);
        assert_eq!(index.class_count(), 0);
        assert!(index.is_empty());
    }

    #[test]
    fn test_index_large_scale() {
        let mut index = DocumentIndex::new();

        for i in 0..10_000 {
            index.register_id(format!("id-{i}"), NodeId::new(i));
            index.register_classes(&format!("class-{i} shared"), NodeId::new(i));
        }

        assert_eq!(index.id_count(), 10_000);
        assert_eq!(index.class_count(), 10_001);

        assert_eq!(index.get_by_id("id-5000"), Some(NodeId::new(5000)));
        assert_eq!(index.get_by_class("class-5000"), &[NodeId::new(5000)]);
        assert_eq!(index.get_by_class("shared").len(), 10_000);
    }

    #[test]
    fn test_index_unicode_ids_and_classes() {
        let mut index = DocumentIndex::new();
        let node1 = NodeId::new(1);
        let node2 = NodeId::new(2);

        index.register_id("日本語".to_string(), node1);
        index.register_classes("emoji-😀 中文", node2);

        assert_eq!(index.get_by_id("日本語"), Some(node1));
        assert_eq!(index.get_by_class("emoji-😀"), &[node2]);
        assert_eq!(index.get_by_class("中文"), &[node2]);
    }

    #[test]
    fn test_index_special_characters() {
        let mut index = DocumentIndex::new();
        let node = NodeId::new(1);

        index.register_id("id-with-dash_and_underscore123".to_string(), node);
        index.register_classes("class:with:colons foo.bar", node);

        assert_eq!(index.get_by_id("id-with-dash_and_underscore123"), Some(node));
        assert_eq!(index.get_by_class("class:with:colons"), &[node]);
        assert_eq!(index.get_by_class("foo.bar"), &[node]);
    }
}
