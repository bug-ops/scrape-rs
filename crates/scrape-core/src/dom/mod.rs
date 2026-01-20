//! DOM layer providing document structure and navigation.
//!
//! This module provides the core document object model:
//! - [`Document`] - The document container with arena allocation
//! - [`Node`] - Individual nodes in the tree
//! - [`NodeId`] - Lightweight handles for node references
//! - [`NodeKind`] - Node type discrimination
//!
//! # Architecture
//!
//! All nodes are stored contiguously in an arena allocator for cache efficiency.
//! Navigation is performed using [`NodeId`] handles, which are cheap to copy.
//!
//! # Navigation APIs
//!
//! The document provides both direct navigation methods and lazy iterators:
//!
//! - Direct: [`Document::parent`], [`Document::first_child`], [`Document::last_child`]
//! - Iterators: [`Document::children`], [`Document::ancestors`], [`Document::descendants`]

mod arena;
mod document;
mod index;
mod node;
pub mod node_type;
pub mod state;
mod tag_id;

#[allow(unused_imports)]
pub use document::{
    AncestorsIter, ChildrenIter, DescendantsIter, Document, DocumentImpl, ElementAncestorsIter,
    ElementChildrenIter, ElementDescendantsIter, ElementNextSiblingsIter, ElementPrevSiblingsIter,
    ElementSiblingsIter, NextSiblingsIter, PrevSiblingsIter, SiblingsIter,
};
pub use index::DocumentIndex;
pub use node::{Node, NodeId, NodeKind};
pub use node_type::{CommentMarker, ElementMarker, NodeType, TextMarker};
pub use state::{Building, DocumentState, MutableState, Queryable, QueryableState, Sealed};
pub use tag_id::TagId;
