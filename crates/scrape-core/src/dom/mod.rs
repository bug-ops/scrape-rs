//! DOM layer providing document structure and navigation.
//!
//! This module provides the internal DOM tree structure used by scrape-core.
//!
//! # Architecture
//!
//! - **Document**: Container for all nodes with root reference
//! - **Node**: Individual DOM node with parent/child/sibling links
//! - **`NodeId`**: Opaque handle to a node in the document
//! - **`NodeKind`**: Element, Text, or Comment node variants

mod node;

pub use node::{Document, Node, NodeId, NodeKind};
