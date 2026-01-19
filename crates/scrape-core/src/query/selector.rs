//! CSS selector parsing and matching via the `selectors` crate.
//!
//! This module provides integration with Mozilla's `selectors` crate for CSS selector
//! parsing and element matching. The key types are:
//!
//! - [`ScrapeSelector`] - Marker type implementing [`selectors::SelectorImpl`]
//! - [`ElementWrapper`] - Adapter implementing [`selectors::Element`] for our DOM

use std::{
    borrow::Borrow,
    fmt,
    hash::{Hash, Hasher},
};

use cssparser::ToCss;
use selectors::{
    Element, OpaqueElement, SelectorList,
    attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint},
    context::{MatchingForInvalidation, NeedsSelectorFlags, QuirksMode, SelectorCaches},
    matching::{ElementSelectorFlags, MatchingContext, MatchingMode},
    parser::{ParseRelative, Parser, SelectorImpl, SelectorParseErrorKind},
};

use super::error::{QueryError, QueryResult};
use crate::dom::{Document, NodeId};

/// A CSS value string that implements the traits required by `selectors`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct CssString(String);

impl CssString {
    /// Creates a new CSS string.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Returns the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for CssString {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl AsRef<str> for CssString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToCss for CssString {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        cssparser::serialize_identifier(&self.0, dest)
    }
}

impl Borrow<str> for CssString {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl precomputed_hash::PrecomputedHash for CssString {
    #[allow(clippy::cast_possible_truncation)]
    fn precomputed_hash(&self) -> u32 {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        // Intentional truncation for hash value
        hasher.finish() as u32
    }
}

/// A local name (tag name) that implements the traits required by `selectors`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct CssLocalName(String);

impl CssLocalName {
    /// Creates a new local name.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into().to_ascii_lowercase())
    }

    /// Returns the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for CssLocalName {
    fn from(s: &str) -> Self {
        Self(s.to_ascii_lowercase())
    }
}

impl AsRef<str> for CssLocalName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToCss for CssLocalName {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str(&self.0)
    }
}

impl Borrow<str> for CssLocalName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl precomputed_hash::PrecomputedHash for CssLocalName {
    #[allow(clippy::cast_possible_truncation)]
    fn precomputed_hash(&self) -> u32 {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        // Intentional truncation for hash value
        hasher.finish() as u32
    }
}

/// Marker type for our selector implementation.
///
/// This type implements [`SelectorImpl`] to configure the selectors crate
/// for our DOM representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrapeSelector;

/// Pseudo-class variants (non-tree-structural).
///
/// We only support a minimal set of pseudo-classes that can be evaluated
/// statically without browser state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NonTSPseudoClass {
    /// The `:link` pseudo-class (matches `<a>` with href).
    Link,
    /// The `:any-link` pseudo-class.
    AnyLink,
}

impl selectors::parser::NonTSPseudoClass for NonTSPseudoClass {
    type Impl = ScrapeSelector;

    fn is_active_or_hover(&self) -> bool {
        false
    }

    fn is_user_action_state(&self) -> bool {
        false
    }
}

impl ToCss for NonTSPseudoClass {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        match self {
            Self::Link => dest.write_str(":link"),
            Self::AnyLink => dest.write_str(":any-link"),
        }
    }
}

/// Pseudo-element variants (not supported for matching).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PseudoElement {}

impl selectors::parser::PseudoElement for PseudoElement {
    type Impl = ScrapeSelector;
}

impl ToCss for PseudoElement {
    fn to_css<W>(&self, _dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        // PseudoElement is an uninhabited type (no variants), so this is unreachable
        unreachable!("PseudoElement has no variants")
    }
}

impl SelectorImpl for ScrapeSelector {
    type ExtraMatchingData<'a> = ();
    type AttrValue = CssString;
    type Identifier = CssLocalName;
    type LocalName = CssLocalName;
    type NamespaceUrl = CssString;
    type NamespacePrefix = CssLocalName;
    type BorrowedLocalName = CssLocalName;
    type BorrowedNamespaceUrl = CssString;
    type NonTSPseudoClass = NonTSPseudoClass;
    type PseudoElement = PseudoElement;
}

/// Custom selector parser for our implementation.
struct SelectorParser;

impl<'i> Parser<'i> for SelectorParser {
    type Impl = ScrapeSelector;
    type Error = SelectorParseErrorKind<'i>;

    fn parse_non_ts_pseudo_class(
        &self,
        location: cssparser::SourceLocation,
        name: cssparser::CowRcStr<'i>,
    ) -> Result<NonTSPseudoClass, cssparser::ParseError<'i, Self::Error>> {
        match name.as_ref() {
            "link" => Ok(NonTSPseudoClass::Link),
            "any-link" => Ok(NonTSPseudoClass::AnyLink),
            _ => Err(cssparser::ParseError {
                kind: cssparser::ParseErrorKind::Custom(
                    SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name),
                ),
                location,
            }),
        }
    }
}

/// Parses a CSS selector string into a compiled selector list.
///
/// # Errors
///
/// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
///
/// # Examples
///
/// ```rust
/// use scrape_core::query::parse_selector;
///
/// let selectors = parse_selector("div.container > span").unwrap();
/// ```
pub fn parse_selector(selector: &str) -> QueryResult<SelectorList<ScrapeSelector>> {
    let mut parser_input = cssparser::ParserInput::new(selector);
    let mut parser = cssparser::Parser::new(&mut parser_input);

    SelectorList::parse(&SelectorParser, &mut parser, ParseRelative::No).map_err(|e| {
        // Sanitize error messages to expose only position info, avoiding potential
        // information disclosure from internal parser state in public error messages.
        QueryError::invalid_selector(format!(
            "invalid selector at line {}, column {}",
            e.location.line, e.location.column
        ))
    })
}

/// Adapter wrapping a DOM node for selector matching.
///
/// This type implements the [`selectors::Element`] trait, allowing our
/// arena-based DOM to be matched against CSS selectors.
#[derive(Debug, Clone, Copy)]
pub struct ElementWrapper<'a> {
    doc: &'a Document,
    id: NodeId,
}

impl<'a> ElementWrapper<'a> {
    /// Creates a new element wrapper.
    #[must_use]
    pub fn new(doc: &'a Document, id: NodeId) -> Self {
        Self { doc, id }
    }

    /// Returns the node ID.
    #[must_use]
    pub fn node_id(&self) -> NodeId {
        self.id
    }

    /// Returns a reference to the document.
    #[must_use]
    pub fn document(&self) -> &'a Document {
        self.doc
    }
}

impl PartialEq for ElementWrapper<'_> {
    fn eq(&self, other: &Self) -> bool {
        // Document equality via pointer comparison ensures elements from different documents
        // are never considered equal, maintaining correctness for cross-document operations.
        // NodeId equality alone is insufficient since different documents may have nodes
        // with the same ID but different content.
        std::ptr::eq(self.doc, other.doc) && self.id == other.id
    }
}

impl Eq for ElementWrapper<'_> {}

impl Element for ElementWrapper<'_> {
    type Impl = ScrapeSelector;

    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(self)
    }

    fn parent_element(&self) -> Option<Self> {
        let parent_id = self.doc.parent(self.id)?;
        let parent_node = self.doc.get(parent_id)?;
        if parent_node.kind.is_element() { Some(Self::new(self.doc, parent_id)) } else { None }
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }

    fn is_pseudo_element(&self) -> bool {
        false
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        let mut current = self.doc.prev_sibling(self.id);
        while let Some(sibling_id) = current {
            if let Some(node) = self.doc.get(sibling_id)
                && node.kind.is_element()
            {
                return Some(Self::new(self.doc, sibling_id));
            }
            current = self.doc.prev_sibling(sibling_id);
        }
        None
    }

    fn next_sibling_element(&self) -> Option<Self> {
        let mut current = self.doc.next_sibling(self.id);
        while let Some(sibling_id) = current {
            if let Some(node) = self.doc.get(sibling_id)
                && node.kind.is_element()
            {
                return Some(Self::new(self.doc, sibling_id));
            }
            current = self.doc.next_sibling(sibling_id);
        }
        None
    }

    fn first_element_child(&self) -> Option<Self> {
        for child_id in self.doc.children(self.id) {
            if let Some(node) = self.doc.get(child_id)
                && node.kind.is_element()
            {
                return Some(Self::new(self.doc, child_id));
            }
        }
        None
    }

    fn is_html_element_in_html_document(&self) -> bool {
        true
    }

    fn has_local_name(&self, local_name: &<Self::Impl as SelectorImpl>::BorrowedLocalName) -> bool {
        self.doc
            .get(self.id)
            .and_then(|n| n.kind.tag_name())
            .is_some_and(|name| name.eq_ignore_ascii_case(local_name.as_str()))
    }

    fn has_namespace(&self, _ns: &<Self::Impl as SelectorImpl>::BorrowedNamespaceUrl) -> bool {
        // We don't track namespaces, so match everything
        true
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.doc
            .get(self.id)
            .and_then(|n| n.kind.tag_name())
            .zip(other.doc.get(other.id).and_then(|n| n.kind.tag_name()))
            .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b))
    }

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&<Self::Impl as SelectorImpl>::NamespaceUrl>,
        local_name: &<Self::Impl as SelectorImpl>::BorrowedLocalName,
        operation: &AttrSelectorOperation<&<Self::Impl as SelectorImpl>::AttrValue>,
    ) -> bool {
        // In HTML, we don't track namespaces, so we accept all namespace constraints
        // - NamespaceConstraint::Any: matches any namespace (e.g., [*|href])
        // - NamespaceConstraint::Specific: matches a specific namespace (we ignore since HTML has
        //   no namespaces)
        let _ = ns;

        let Some(node) = self.doc.get(self.id) else { return false };
        let Some(attrs) = node.kind.attributes() else { return false };

        // HTML attribute names are case-insensitive
        let attr_name = local_name.as_str();
        let value = attrs.iter().find(|(k, _)| k.eq_ignore_ascii_case(attr_name)).map(|(_, v)| v);

        let Some(value) = value else { return false };

        operation.eval_str(value)
    }

    fn match_non_ts_pseudo_class(
        &self,
        pc: &NonTSPseudoClass,
        _context: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        match pc {
            NonTSPseudoClass::Link | NonTSPseudoClass::AnyLink => {
                // Match <a>, <area>, or <link> elements with href
                let Some(node) = self.doc.get(self.id) else { return false };
                let Some(tag_name) = node.kind.tag_name() else { return false };
                let Some(attrs) = node.kind.attributes() else { return false };

                matches!(tag_name, "a" | "area" | "link") && attrs.contains_key("href")
            }
        }
    }

    fn match_pseudo_element(
        &self,
        _pe: &PseudoElement,
        _context: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        // No pseudo-elements supported
        false
    }

    fn is_link(&self) -> bool {
        let Some(node) = self.doc.get(self.id) else { return false };
        let Some(tag_name) = node.kind.tag_name() else { return false };
        let Some(attrs) = node.kind.attributes() else { return false };

        matches!(tag_name, "a" | "area" | "link") && attrs.contains_key("href")
    }

    fn is_html_slot_element(&self) -> bool {
        false
    }

    fn has_id(
        &self,
        id: &<Self::Impl as SelectorImpl>::Identifier,
        case_sensitivity: CaseSensitivity,
    ) -> bool {
        let Some(node) = self.doc.get(self.id) else { return false };
        let Some(attrs) = node.kind.attributes() else { return false };
        let Some(element_id) = attrs.get("id") else { return false };

        case_sensitivity.eq(element_id.as_bytes(), id.as_str().as_bytes())
    }

    fn has_class(
        &self,
        name: &<Self::Impl as SelectorImpl>::Identifier,
        case_sensitivity: CaseSensitivity,
    ) -> bool {
        let Some(node) = self.doc.get(self.id) else { return false };
        let Some(attrs) = node.kind.attributes() else { return false };
        let Some(class_attr) = attrs.get("class") else { return false };

        class_attr
            .split_whitespace()
            .any(|class| case_sensitivity.eq(class.as_bytes(), name.as_str().as_bytes()))
    }

    fn imported_part(
        &self,
        _name: &<Self::Impl as SelectorImpl>::Identifier,
    ) -> Option<<Self::Impl as SelectorImpl>::Identifier> {
        None
    }

    fn is_part(&self, _name: &<Self::Impl as SelectorImpl>::Identifier) -> bool {
        false
    }

    fn is_empty(&self) -> bool {
        // Element is empty if it has no element or text children
        for child_id in self.doc.children(self.id) {
            if let Some(node) = self.doc.get(child_id) {
                match &node.kind {
                    crate::dom::NodeKind::Element { .. } => return false,
                    crate::dom::NodeKind::Text { content } => {
                        if !content.trim().is_empty() {
                            return false;
                        }
                    }
                    crate::dom::NodeKind::Comment { .. } => {}
                }
            }
        }
        true
    }

    fn is_root(&self) -> bool {
        self.doc.root().is_some_and(|_root_id| {
            // Walk up to find the html element
            self.doc
                .get(self.id)
                .is_some_and(|node| node.kind.tag_name().is_some_and(|name| name == "html"))
                && self.parent_element().is_none()
        })
    }

    fn apply_selector_flags(&self, _flags: ElementSelectorFlags) {
        // No-op: we don't need to track selector flags
    }

    fn add_element_unique_hashes(&self, _filter: &mut selectors::bloom::BloomFilter) -> bool {
        false
    }

    fn has_custom_state(&self, _name: &<Self::Impl as SelectorImpl>::Identifier) -> bool {
        false
    }
}

/// Checks if an element matches a selector list.
///
/// This creates new [`SelectorCaches`] for each call. For batch operations
/// (e.g., iterating over many elements), use [`matches_selector_with_caches`]
/// to reuse caches and avoid allocation overhead.
///
/// # Examples
///
/// ```rust
/// use scrape_core::{
///     Html5everParser, Parser,
///     query::{matches_selector, parse_selector},
/// };
///
/// let parser = Html5everParser;
/// let doc = parser.parse("<div class=\"foo\"><span id=\"bar\">text</span></div>").unwrap();
/// let selectors = parse_selector("span#bar").unwrap();
///
/// // Find span element and check if it matches
/// for (id, node) in doc.nodes() {
///     if node.kind.tag_name() == Some("span") {
///         assert!(matches_selector(&doc, id, &selectors));
///     }
/// }
/// ```
#[must_use]
pub fn matches_selector(
    doc: &Document,
    id: NodeId,
    selectors: &SelectorList<ScrapeSelector>,
) -> bool {
    let mut caches = SelectorCaches::default();
    matches_selector_with_caches(doc, id, selectors, &mut caches)
}

/// Checks if an element matches a selector list, reusing provided caches.
///
/// This is more efficient than [`matches_selector`] when matching many elements
/// against the same selector, as it avoids creating new [`SelectorCaches`]
/// for each element.
///
/// # Examples
///
/// ```rust
/// use scrape_core::{
///     Html5everParser, Parser,
///     query::{matches_selector_with_caches, parse_selector},
/// };
/// use selectors::context::SelectorCaches;
///
/// let parser = Html5everParser;
/// let doc = parser.parse("<ul><li>A</li><li>B</li><li>C</li></ul>").unwrap();
/// let selectors = parse_selector("li").unwrap();
///
/// // Reuse caches for efficiency when matching many elements
/// let mut caches = SelectorCaches::default();
/// let count = doc
///     .nodes()
///     .filter(|(id, n)| {
///         n.kind.is_element() && matches_selector_with_caches(&doc, *id, &selectors, &mut caches)
///     })
///     .count();
/// assert_eq!(count, 3);
/// ```
#[must_use]
pub fn matches_selector_with_caches(
    doc: &Document,
    id: NodeId,
    selectors: &SelectorList<ScrapeSelector>,
    caches: &mut SelectorCaches,
) -> bool {
    let element = ElementWrapper::new(doc, id);
    let mut context = MatchingContext::new(
        MatchingMode::Normal,
        None,
        caches,
        QuirksMode::NoQuirks,
        NeedsSelectorFlags::No,
        MatchingForInvalidation::No,
    );

    selectors.slice().iter().any(|selector| {
        selectors::matching::matches_selector(selector, 0, None, &element, &mut context)
    })
}

/// Checks if an element matches a selector list.
///
/// This is a convenience wrapper around [`matches_selector`] for use with `Tag::closest()`.
///
/// # Examples
///
/// ```rust
/// use scrape_core::{
///     Html5everParser, Parser,
///     query::{matches_selector_list, parse_selector},
/// };
///
/// let parser = Html5everParser;
/// let doc = parser.parse("<div class='foo'><span id='bar'>text</span></div>").unwrap();
/// let selectors = parse_selector("span#bar").unwrap();
///
/// // Find span element and check if it matches
/// for (id, node) in doc.nodes() {
///     if node.kind.tag_name() == Some("span") {
///         assert!(matches_selector_list(&doc, id, &selectors));
///     }
/// }
/// ```
#[must_use]
pub fn matches_selector_list(
    doc: &Document,
    id: NodeId,
    selector_list: &SelectorList<ScrapeSelector>,
) -> bool {
    matches_selector(doc, id, selector_list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Html5everParser, Parser};

    fn parse_doc(html: &str) -> Document {
        Html5everParser.parse(html).unwrap()
    }

    fn find_element_by_tag(doc: &Document, tag: &str) -> Option<NodeId> {
        doc.nodes().find(|(_, n)| n.kind.tag_name() == Some(tag)).map(|(id, _)| id)
    }

    #[test]
    fn test_parse_simple_selector() {
        let selectors = parse_selector("div").unwrap();
        assert_eq!(selectors.slice().len(), 1);
    }

    #[test]
    fn test_parse_class_selector() {
        let selectors = parse_selector(".foo").unwrap();
        assert_eq!(selectors.slice().len(), 1);
    }

    #[test]
    fn test_parse_id_selector() {
        let selectors = parse_selector("#bar").unwrap();
        assert_eq!(selectors.slice().len(), 1);
    }

    #[test]
    fn test_parse_compound_selector() {
        let selectors = parse_selector("div.foo#bar").unwrap();
        assert_eq!(selectors.slice().len(), 1);
    }

    #[test]
    fn test_parse_descendant_combinator() {
        let selectors = parse_selector("div span").unwrap();
        assert_eq!(selectors.slice().len(), 1);
    }

    #[test]
    fn test_parse_child_combinator() {
        let selectors = parse_selector("div > span").unwrap();
        assert_eq!(selectors.slice().len(), 1);
    }

    #[test]
    fn test_parse_adjacent_sibling() {
        let selectors = parse_selector("h1 + p").unwrap();
        assert_eq!(selectors.slice().len(), 1);
    }

    #[test]
    fn test_parse_general_sibling() {
        let selectors = parse_selector("h1 ~ p").unwrap();
        assert_eq!(selectors.slice().len(), 1);
    }

    #[test]
    fn test_parse_attribute_exists() {
        let selectors = parse_selector("[href]").unwrap();
        assert_eq!(selectors.slice().len(), 1);
    }

    #[test]
    fn test_parse_attribute_equals() {
        let selectors = parse_selector("[type=\"text\"]").unwrap();
        assert_eq!(selectors.slice().len(), 1);
    }

    #[test]
    fn test_parse_multiple_selectors() {
        let selectors = parse_selector("div, span, p").unwrap();
        assert_eq!(selectors.slice().len(), 3);
    }

    #[test]
    fn test_parse_invalid_selector() {
        let result = parse_selector("[");
        assert!(result.is_err());
    }

    #[test]
    fn test_match_tag_selector() {
        let doc = parse_doc("<div><span>text</span></div>");
        let span_id = find_element_by_tag(&doc, "span").unwrap();
        let selectors = parse_selector("span").unwrap();
        assert!(matches_selector(&doc, span_id, &selectors));
    }

    #[test]
    fn test_match_class_selector() {
        let doc = parse_doc("<div class=\"foo bar\">text</div>");
        let div_id = find_element_by_tag(&doc, "div").unwrap();

        let selectors = parse_selector(".foo").unwrap();
        assert!(matches_selector(&doc, div_id, &selectors));

        let selectors = parse_selector(".bar").unwrap();
        assert!(matches_selector(&doc, div_id, &selectors));

        let selectors = parse_selector(".baz").unwrap();
        assert!(!matches_selector(&doc, div_id, &selectors));
    }

    #[test]
    fn test_match_id_selector() {
        let doc = parse_doc("<div id=\"main\">text</div>");
        let div_id = find_element_by_tag(&doc, "div").unwrap();

        let selectors = parse_selector("#main").unwrap();
        assert!(matches_selector(&doc, div_id, &selectors));

        let selectors = parse_selector("#other").unwrap();
        assert!(!matches_selector(&doc, div_id, &selectors));
    }

    #[test]
    fn test_match_compound_selector() {
        let doc = parse_doc("<div class=\"foo\" id=\"bar\">text</div>");
        let div_id = find_element_by_tag(&doc, "div").unwrap();

        let selectors = parse_selector("div.foo#bar").unwrap();
        assert!(matches_selector(&doc, div_id, &selectors));

        let selectors = parse_selector("div.foo#baz").unwrap();
        assert!(!matches_selector(&doc, div_id, &selectors));
    }

    #[test]
    fn test_match_attribute_exists() {
        let doc = parse_doc("<a href=\"/page\">link</a>");
        let a_id = find_element_by_tag(&doc, "a").unwrap();

        // Verify we have the right element
        let node = doc.get(a_id).unwrap();
        let attrs = node.kind.attributes().unwrap();
        assert!(attrs.contains_key("href"), "Element should have href attribute: {attrs:?}");

        let selectors = parse_selector("[href]").unwrap();
        assert_eq!(selectors.slice().len(), 1, "Should have one selector");
        assert!(matches_selector(&doc, a_id, &selectors), "Element with href should match [href]");

        let selectors = parse_selector("[title]").unwrap();
        assert!(!matches_selector(&doc, a_id, &selectors));
    }

    #[test]
    fn test_match_attribute_equals() {
        let doc = parse_doc("<input type=\"text\">");
        let input_id = find_element_by_tag(&doc, "input").unwrap();

        let selectors = parse_selector("[type=\"text\"]").unwrap();
        assert!(matches_selector(&doc, input_id, &selectors));

        let selectors = parse_selector("[type=\"password\"]").unwrap();
        assert!(!matches_selector(&doc, input_id, &selectors));
    }

    #[test]
    fn test_element_is_empty() {
        let doc = parse_doc("<div></div><span>text</span>");
        let div_id = find_element_by_tag(&doc, "div").unwrap();
        let span_id = find_element_by_tag(&doc, "span").unwrap();

        let selectors = parse_selector(":empty").unwrap();
        assert!(matches_selector(&doc, div_id, &selectors));
        assert!(!matches_selector(&doc, span_id, &selectors));
    }

    #[test]
    fn test_element_first_child() {
        let doc = parse_doc("<ul><li>first</li><li>second</li></ul>");

        // Find first li
        let first_li =
            doc.nodes().find(|(_, n)| n.kind.tag_name() == Some("li")).map(|(id, _)| id).unwrap();

        let selectors = parse_selector("li:first-child").unwrap();
        assert!(matches_selector(&doc, first_li, &selectors));
    }

    #[test]
    fn test_match_not_selector() {
        let doc = parse_doc("<div class=\"foo\">a</div><div class=\"bar\">b</div>");

        let divs: Vec<_> = doc
            .nodes()
            .filter(|(_, n)| n.kind.tag_name() == Some("div"))
            .map(|(id, _)| id)
            .collect();

        let selectors = parse_selector("div:not(.foo)").unwrap();

        // Only the second div (with class="bar") should match
        let match_count = divs.iter().filter(|id| matches_selector(&doc, **id, &selectors)).count();
        assert_eq!(match_count, 1);
    }

    // ==================== Attribute Substring Selectors ====================

    #[test]
    fn test_match_attribute_prefix() {
        let doc = parse_doc(
            r#"<a href="https://example.com">secure</a><a href="http://example.com">insecure</a>"#,
        );

        let links: Vec<_> =
            doc.nodes().filter(|(_, n)| n.kind.tag_name() == Some("a")).map(|(id, _)| id).collect();
        assert_eq!(links.len(), 2);

        let selectors = parse_selector("[href^=\"https\"]").unwrap();
        let match_count =
            links.iter().filter(|id| matches_selector(&doc, **id, &selectors)).count();
        assert_eq!(match_count, 1, "[attr^=prefix] should match elements starting with prefix");
    }

    #[test]
    fn test_match_attribute_suffix() {
        let doc = parse_doc(r#"<a href="/page.html">html</a><a href="/page.pdf">pdf</a>"#);

        let links: Vec<_> =
            doc.nodes().filter(|(_, n)| n.kind.tag_name() == Some("a")).map(|(id, _)| id).collect();
        assert_eq!(links.len(), 2);

        let selectors = parse_selector("[href$=\".html\"]").unwrap();
        let match_count =
            links.iter().filter(|id| matches_selector(&doc, **id, &selectors)).count();
        assert_eq!(match_count, 1, "[attr$=suffix] should match elements ending with suffix");
    }

    #[test]
    fn test_match_attribute_contains() {
        let doc = parse_doc(r#"<a href="/foo/bar/baz">yes</a><a href="/qux">no</a>"#);

        let links: Vec<_> =
            doc.nodes().filter(|(_, n)| n.kind.tag_name() == Some("a")).map(|(id, _)| id).collect();
        assert_eq!(links.len(), 2);

        let selectors = parse_selector("[href*=\"bar\"]").unwrap();
        let match_count =
            links.iter().filter(|id| matches_selector(&doc, **id, &selectors)).count();
        assert_eq!(match_count, 1, "[attr*=substring] should match elements containing substring");
    }

    #[test]
    fn test_match_attribute_word() {
        let doc = parse_doc(r#"<div class="foo bar baz">yes</div><div class="foobar">no</div>"#);

        let divs: Vec<_> = doc
            .nodes()
            .filter(|(_, n)| n.kind.tag_name() == Some("div"))
            .map(|(id, _)| id)
            .collect();
        assert_eq!(divs.len(), 2);

        let selectors = parse_selector("[class~=\"bar\"]").unwrap();
        let match_count = divs.iter().filter(|id| matches_selector(&doc, **id, &selectors)).count();
        assert_eq!(
            match_count, 1,
            "[attr~=word] should match elements with word in space-separated list"
        );
    }

    #[test]
    fn test_match_attribute_lang() {
        let doc = parse_doc(
            r#"<div lang="en-US">US</div><div lang="en-GB">GB</div><div lang="fr">FR</div>"#,
        );

        let divs: Vec<_> = doc
            .nodes()
            .filter(|(_, n)| n.kind.tag_name() == Some("div"))
            .map(|(id, _)| id)
            .collect();
        assert_eq!(divs.len(), 3);

        let selectors = parse_selector("[lang|=\"en\"]").unwrap();
        let match_count = divs.iter().filter(|id| matches_selector(&doc, **id, &selectors)).count();
        assert_eq!(match_count, 2, "[attr|=lang] should match 'en' and 'en-*' values");
    }

    // ==================== Pseudo-class Selectors ====================

    #[test]
    fn test_match_nth_child_even() {
        let doc = parse_doc("<ul><li>1</li><li>2</li><li>3</li><li>4</li></ul>");

        let lis: Vec<_> = doc
            .nodes()
            .filter(|(_, n)| n.kind.tag_name() == Some("li"))
            .map(|(id, _)| id)
            .collect();
        assert_eq!(lis.len(), 4);

        let selectors = parse_selector("li:nth-child(even)").unwrap();
        let match_count = lis.iter().filter(|id| matches_selector(&doc, **id, &selectors)).count();
        assert_eq!(match_count, 2, ":nth-child(even) should match 2nd and 4th elements");
    }

    #[test]
    fn test_match_nth_child_2n_plus_1() {
        let doc = parse_doc("<ul><li>1</li><li>2</li><li>3</li><li>4</li></ul>");

        let lis: Vec<_> = doc
            .nodes()
            .filter(|(_, n)| n.kind.tag_name() == Some("li"))
            .map(|(id, _)| id)
            .collect();
        assert_eq!(lis.len(), 4);

        let selectors = parse_selector("li:nth-child(2n+1)").unwrap();
        let match_count = lis.iter().filter(|id| matches_selector(&doc, **id, &selectors)).count();
        assert_eq!(match_count, 2, ":nth-child(2n+1) should match odd elements (1st and 3rd)");
    }

    #[test]
    fn test_match_last_child() {
        let doc = parse_doc("<ul><li id=\"first\">1</li><li id=\"last\">2</li></ul>");

        let lis: Vec<_> = doc
            .nodes()
            .filter(|(_, n)| n.kind.tag_name() == Some("li"))
            .map(|(id, _)| id)
            .collect();
        assert_eq!(lis.len(), 2);

        let selectors = parse_selector("li:last-child").unwrap();
        let matches: Vec<_> =
            lis.iter().filter(|id| matches_selector(&doc, **id, &selectors)).collect();
        assert_eq!(matches.len(), 1, ":last-child should match exactly one element");

        // Verify it's the last one
        let last_id = matches[0];
        let node = doc.get(*last_id).unwrap();
        let attrs = node.kind.attributes().unwrap();
        assert_eq!(attrs.get("id"), Some(&"last".to_string()));
    }

    // ==================== Sibling Combinator Selectors ====================

    #[test]
    fn test_match_adjacent_sibling() {
        let doc = parse_doc("<h1>Title</h1><p>First paragraph</p><p>Second paragraph</p>");

        let ps: Vec<_> =
            doc.nodes().filter(|(_, n)| n.kind.tag_name() == Some("p")).map(|(id, _)| id).collect();
        assert_eq!(ps.len(), 2);

        let selectors = parse_selector("h1 + p").unwrap();
        let match_count = ps.iter().filter(|id| matches_selector(&doc, **id, &selectors)).count();
        assert_eq!(match_count, 1, "h1 + p should match only the immediately adjacent paragraph");
    }

    #[test]
    fn test_match_general_sibling() {
        let doc = parse_doc("<h1>Title</h1><p>First</p><p>Second</p>");

        let ps: Vec<_> =
            doc.nodes().filter(|(_, n)| n.kind.tag_name() == Some("p")).map(|(id, _)| id).collect();
        assert_eq!(ps.len(), 2);

        let selectors = parse_selector("h1 ~ p").unwrap();
        let match_count = ps.iter().filter(|id| matches_selector(&doc, **id, &selectors)).count();
        assert_eq!(match_count, 2, "h1 ~ p should match all following sibling paragraphs");
    }

    #[test]
    fn test_match_general_sibling_not_preceding() {
        let doc = parse_doc("<p>Before</p><h1>Title</h1><p>After</p>");

        let ps: Vec<_> =
            doc.nodes().filter(|(_, n)| n.kind.tag_name() == Some("p")).map(|(id, _)| id).collect();
        assert_eq!(ps.len(), 2);

        let selectors = parse_selector("h1 ~ p").unwrap();
        let match_count = ps.iter().filter(|id| matches_selector(&doc, **id, &selectors)).count();
        assert_eq!(match_count, 1, "h1 ~ p should not match paragraphs preceding h1");
    }

    #[test]
    fn test_match_adjacent_sibling_requires_immediate() {
        let doc = parse_doc("<h1>Title</h1><div>Separator</div><p>Paragraph</p>");

        let p_id = find_element_by_tag(&doc, "p").unwrap();

        let selectors = parse_selector("h1 + p").unwrap();
        assert!(
            !matches_selector(&doc, p_id, &selectors),
            "h1 + p should not match when div is between them"
        );
    }

    // ==================== matches_selector_with_caches ====================

    #[test]
    fn test_matches_selector_with_caches() {
        let doc = parse_doc("<ul><li>A</li><li>B</li><li>C</li></ul>");
        let selectors = parse_selector("li").unwrap();

        let mut caches = SelectorCaches::default();
        let count = doc
            .nodes()
            .filter(|(id, n)| {
                n.kind.is_element()
                    && matches_selector_with_caches(&doc, *id, &selectors, &mut caches)
            })
            .count();
        assert_eq!(count, 3);
    }
}
