//! Sealed node type markers.
//!
//! This module defines compile-time node type markers using the sealed trait pattern.
//! The sealed trait prevents external code from implementing new node types while
//! allowing the library to add new types in the future without breaking changes.
//!
//! # Available Node Types
//!
//! - [`ElementMarker`] - Element nodes (`<div>`, `<span>`, etc.)
//! - [`TextMarker`] - Text content nodes
//! - [`CommentMarker`] - Comment nodes (`<!-- ... -->`)

mod private {
    pub trait Sealed {}
}

/// Trait implemented by all node type markers.
///
/// This trait is sealed - it cannot be implemented outside this crate.
/// This allows the library to add new node types without breaking
/// external code.
pub trait NodeType: private::Sealed {
    /// Returns the string name of this node type.
    fn type_name() -> &'static str;
}

/// Marker type for element nodes.
///
/// Represents element nodes in the DOM tree (e.g., `<div>`, `<span>`, `<p>`).
#[derive(Debug, Clone, Copy)]
pub struct ElementMarker;

impl private::Sealed for ElementMarker {}
impl NodeType for ElementMarker {
    fn type_name() -> &'static str {
        "element"
    }
}

/// Marker type for text nodes.
///
/// Represents text content nodes in the DOM tree.
#[derive(Debug, Clone, Copy)]
pub struct TextMarker;

impl private::Sealed for TextMarker {}
impl NodeType for TextMarker {
    fn type_name() -> &'static str {
        "text"
    }
}

/// Marker type for comment nodes.
///
/// Represents comment nodes (`<!-- ... -->`) in the DOM tree.
#[derive(Debug, Clone, Copy)]
pub struct CommentMarker;

impl private::Sealed for CommentMarker {}
impl NodeType for CommentMarker {
    fn type_name() -> &'static str {
        "comment"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_names() {
        assert_eq!(ElementMarker::type_name(), "element");
        assert_eq!(TextMarker::type_name(), "text");
        assert_eq!(CommentMarker::type_name(), "comment");
    }

    #[test]
    fn markers_are_zero_sized() {
        assert_eq!(std::mem::size_of::<ElementMarker>(), 0);
        assert_eq!(std::mem::size_of::<TextMarker>(), 0);
        assert_eq!(std::mem::size_of::<CommentMarker>(), 0);
    }

    #[test]
    #[allow(clippy::no_effect_underscore_binding)]
    fn markers_are_copy() {
        // Copy trait is verified by using value twice
        let e = ElementMarker;
        let _e2 = e; // First use (copy)
        let _e3 = e; // Second use (copy) - would fail without Copy

        let t = TextMarker;
        let _t2 = t;
        let _t3 = t;

        let c = CommentMarker;
        let _c2 = c;
        let _c3 = c;
    }
}
