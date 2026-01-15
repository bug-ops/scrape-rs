//! Attribute-based filtering for BeautifulSoup-style queries.
//!
//! This module provides [`Filter`], a builder for constructing element filters
//! based on tag name, classes, ID, and arbitrary attributes.
//!
//! # Examples
//!
//! ```rust
//! use scrape_core::query::Filter;
//!
//! let filter = Filter::new().tag("div").class("container").id("main").has_attr("data-section");
//! ```

use std::collections::HashMap;

use crate::dom::{Document, NodeId};

/// Filter criteria for element queries (BeautifulSoup-style).
///
/// `Filter` provides a builder API for constructing element matchers based on:
/// - Tag name
/// - CSS classes (all must match)
/// - Element ID
/// - Arbitrary attribute presence/values
///
/// # Examples
///
/// ```rust
/// use scrape_core::query::Filter;
///
/// // Match any div with class "item"
/// let filter = Filter::new().tag("div").class("item");
///
/// // Match elements with a specific data attribute
/// let filter = Filter::new().attr("data-id", "123");
///
/// // Match elements that have a certain attribute (any value)
/// let filter = Filter::new().has_attr("disabled");
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Filter {
    /// Tag name to match (case-insensitive).
    tag: Option<String>,
    /// CSS classes that must all be present.
    classes: Vec<String>,
    /// Element ID to match.
    id: Option<String>,
    /// Attributes that must exist with optional value matching.
    /// If value is None, only presence is checked.
    attrs: HashMap<String, Option<String>>,
}

impl Filter {
    /// Creates a new empty filter.
    ///
    /// An empty filter matches all elements.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters by tag name (case-insensitive).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::query::Filter;
    ///
    /// let filter = Filter::new().tag("div");
    /// ```
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into().to_lowercase());
        self
    }

    /// Adds a class that must be present.
    ///
    /// Multiple calls add multiple required classes (all must match).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::query::Filter;
    ///
    /// // Element must have both "container" AND "active" classes
    /// let filter = Filter::new().class("container").class("active");
    /// ```
    #[must_use]
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.classes.push(class.into());
        self
    }

    /// Filters by element ID.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::query::Filter;
    ///
    /// let filter = Filter::new().id("main-content");
    /// ```
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Requires an attribute to exist (any value).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::query::Filter;
    ///
    /// // Match any element with a "disabled" attribute
    /// let filter = Filter::new().has_attr("disabled");
    /// ```
    #[must_use]
    pub fn has_attr(mut self, name: impl Into<String>) -> Self {
        self.attrs.insert(name.into(), None);
        self
    }

    /// Requires an attribute with a specific value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::query::Filter;
    ///
    /// // Match input elements with type="text"
    /// let filter = Filter::new().tag("input").attr("type", "text");
    /// ```
    #[must_use]
    pub fn attr(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attrs.insert(name.into(), Some(value.into()));
        self
    }

    /// Checks if an element matches this filter.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::{Html5everParser, Parser, query::Filter};
    ///
    /// let parser = Html5everParser;
    /// let doc = parser.parse("<div class=\"foo bar\" id=\"main\">text</div>").unwrap();
    ///
    /// let filter = Filter::new().tag("div").class("foo").id("main");
    ///
    /// for (id, _) in doc.nodes() {
    ///     if filter.matches(&doc, id) {
    ///         println!("Found matching element!");
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn matches(&self, doc: &Document, id: NodeId) -> bool {
        let Some(node) = doc.get(id) else { return false };

        // Must be an element
        if !node.kind.is_element() {
            return false;
        }

        let tag_name = node.kind.tag_name();
        let attributes = node.kind.attributes();

        // Check tag name
        if let Some(ref required_tag) = self.tag
            && !tag_name.is_some_and(|name| name.eq_ignore_ascii_case(required_tag))
        {
            return false;
        }

        // Check ID
        if let Some(ref required_id) = self.id {
            let has_matching_id =
                attributes.and_then(|attrs| attrs.get("id")).is_some_and(|id| id == required_id);
            if !has_matching_id {
                return false;
            }
        }

        // Check classes (all must match)
        if !self.classes.is_empty() {
            let class_attr = attributes.and_then(|attrs| attrs.get("class"));

            for required_class in &self.classes {
                #[cfg(feature = "simd")]
                let has_class = class_attr
                    .is_some_and(|classes| crate::simd::contains_class(classes, required_class));

                #[cfg(not(feature = "simd"))]
                let has_class = class_attr
                    .is_some_and(|classes| classes.split_whitespace().any(|c| c == required_class));

                if !has_class {
                    return false;
                }
            }
        }

        // Check attributes
        for (attr_name, expected_value) in &self.attrs {
            let actual_value = attributes.and_then(|attrs| attrs.get(attr_name));

            match (actual_value, expected_value) {
                (None, _) => return false, // Attribute must exist
                (Some(actual), Some(expected)) => {
                    if actual != expected {
                        return false;
                    }
                }
                (Some(_), None) => {} // Just checking existence, which passed
            }
        }

        true
    }

    /// Returns true if this filter has no criteria (matches everything).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tag.is_none() && self.classes.is_empty() && self.id.is_none() && self.attrs.is_empty()
    }
}

/// Finds all elements matching a filter.
///
/// # Examples
///
/// ```rust
/// use scrape_core::{
///     Html5everParser, Parser,
///     query::{Filter, find_by_filter},
/// };
///
/// let parser = Html5everParser;
/// let doc = parser.parse("<div class=\"item\">A</div><div class=\"item\">B</div>").unwrap();
///
/// let filter = Filter::new().class("item");
/// let results = find_by_filter(&doc, &filter);
/// assert_eq!(results.len(), 2);
/// ```
#[must_use]
pub fn find_by_filter(doc: &Document, filter: &Filter) -> Vec<NodeId> {
    let mut results = Vec::new();

    let Some(root) = doc.root() else {
        return results;
    };

    // Check root
    if filter.matches(doc, root) {
        results.push(root);
    }

    // Check descendants
    for id in doc.descendants(root) {
        if filter.matches(doc, id) {
            results.push(id);
        }
    }

    results
}

/// Finds the first element matching a filter.
///
/// # Examples
///
/// ```rust
/// use scrape_core::{
///     Html5everParser, Parser,
///     query::{Filter, find_first_by_filter},
/// };
///
/// let parser = Html5everParser;
/// let doc = parser.parse("<div id=\"first\">A</div><div id=\"second\">B</div>").unwrap();
///
/// let filter = Filter::new().tag("div");
/// let result = find_first_by_filter(&doc, &filter);
/// assert!(result.is_some());
/// ```
#[must_use]
pub fn find_first_by_filter(doc: &Document, filter: &Filter) -> Option<NodeId> {
    let root = doc.root()?;

    // Check root
    if filter.matches(doc, root) {
        return Some(root);
    }

    // Check descendants
    doc.descendants(root).find(|&id| filter.matches(doc, id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Html5everParser, Parser};

    fn parse_doc(html: &str) -> Document {
        Html5everParser.parse(html).unwrap()
    }

    #[test]
    fn test_filter_new() {
        let filter = Filter::new();
        assert!(filter.is_empty());
    }

    #[test]
    fn test_filter_tag() {
        let filter = Filter::new().tag("div");
        assert!(!filter.is_empty());
        assert_eq!(filter.tag, Some("div".to_string()));
    }

    #[test]
    fn test_filter_class() {
        let filter = Filter::new().class("foo").class("bar");
        assert_eq!(filter.classes, vec!["foo", "bar"]);
    }

    #[test]
    fn test_filter_id() {
        let filter = Filter::new().id("main");
        assert_eq!(filter.id, Some("main".to_string()));
    }

    #[test]
    fn test_filter_has_attr() {
        let filter = Filter::new().has_attr("disabled");
        assert!(filter.attrs.contains_key("disabled"));
        assert_eq!(filter.attrs.get("disabled"), Some(&None));
    }

    #[test]
    fn test_filter_attr() {
        let filter = Filter::new().attr("type", "text");
        assert_eq!(filter.attrs.get("type"), Some(&Some("text".to_string())));
    }

    #[test]
    fn test_filter_matches_tag() {
        let doc = parse_doc("<div>text</div>");
        let filter = Filter::new().tag("div");

        let div_id =
            doc.nodes().find(|(_, n)| n.kind.tag_name() == Some("div")).map(|(id, _)| id).unwrap();

        assert!(filter.matches(&doc, div_id));
    }

    #[test]
    fn test_filter_matches_tag_case_insensitive() {
        let doc = parse_doc("<DIV>text</DIV>");
        let filter = Filter::new().tag("div");

        let div_id = doc
            .nodes()
            .find(|(_, n)| n.kind.tag_name().is_some_and(|t| t.eq_ignore_ascii_case("div")))
            .map(|(id, _)| id)
            .unwrap();

        assert!(filter.matches(&doc, div_id));
    }

    #[test]
    fn test_filter_matches_class() {
        let doc = parse_doc("<div class=\"foo bar\">text</div>");
        let filter = Filter::new().class("foo");

        let div_id =
            doc.nodes().find(|(_, n)| n.kind.tag_name() == Some("div")).map(|(id, _)| id).unwrap();

        assert!(filter.matches(&doc, div_id));
    }

    #[test]
    fn test_filter_matches_multiple_classes() {
        let doc = parse_doc("<div class=\"foo bar baz\">text</div>");

        let div_id =
            doc.nodes().find(|(_, n)| n.kind.tag_name() == Some("div")).map(|(id, _)| id).unwrap();

        let filter_all = Filter::new().class("foo").class("bar");
        assert!(filter_all.matches(&doc, div_id));

        let filter_missing = Filter::new().class("foo").class("missing");
        assert!(!filter_missing.matches(&doc, div_id));
    }

    #[test]
    fn test_filter_matches_id() {
        let doc = parse_doc("<div id=\"main\">text</div>");
        let filter = Filter::new().id("main");

        let div_id =
            doc.nodes().find(|(_, n)| n.kind.tag_name() == Some("div")).map(|(id, _)| id).unwrap();

        assert!(filter.matches(&doc, div_id));
    }

    #[test]
    fn test_filter_matches_has_attr() {
        let doc = parse_doc("<button disabled>Click</button>");
        let filter = Filter::new().has_attr("disabled");

        let button_id = doc
            .nodes()
            .find(|(_, n)| n.kind.tag_name() == Some("button"))
            .map(|(id, _)| id)
            .unwrap();

        assert!(filter.matches(&doc, button_id));
    }

    #[test]
    fn test_filter_matches_attr_value() {
        let doc = parse_doc("<input type=\"text\">");
        let filter = Filter::new().attr("type", "text");

        let input_id = doc
            .nodes()
            .find(|(_, n)| n.kind.tag_name() == Some("input"))
            .map(|(id, _)| id)
            .unwrap();

        assert!(filter.matches(&doc, input_id));

        let wrong_filter = Filter::new().attr("type", "password");
        assert!(!wrong_filter.matches(&doc, input_id));
    }

    #[test]
    fn test_filter_matches_combined() {
        let doc =
            parse_doc("<div class=\"container\" id=\"main\" data-section=\"hero\">text</div>");

        let div_id =
            doc.nodes().find(|(_, n)| n.kind.tag_name() == Some("div")).map(|(id, _)| id).unwrap();

        let filter =
            Filter::new().tag("div").class("container").id("main").attr("data-section", "hero");

        assert!(filter.matches(&doc, div_id));
    }

    #[test]
    fn test_filter_not_matches_wrong_tag() {
        let doc = parse_doc("<span>text</span>");
        let filter = Filter::new().tag("div");

        let span_id =
            doc.nodes().find(|(_, n)| n.kind.tag_name() == Some("span")).map(|(id, _)| id).unwrap();

        assert!(!filter.matches(&doc, span_id));
    }

    #[test]
    fn test_filter_not_matches_missing_class() {
        let doc = parse_doc("<div class=\"foo\">text</div>");
        let filter = Filter::new().class("bar");

        let div_id =
            doc.nodes().find(|(_, n)| n.kind.tag_name() == Some("div")).map(|(id, _)| id).unwrap();

        assert!(!filter.matches(&doc, div_id));
    }

    #[test]
    fn test_empty_filter_matches_all_elements() {
        let doc = parse_doc("<div><span>text</span></div>");
        let filter = Filter::new();

        let element_count = doc.nodes().filter(|(id, _)| filter.matches(&doc, *id)).count();

        // Should match all element nodes
        assert!(element_count >= 2);
    }

    #[test]
    fn test_find_by_filter() {
        let doc = parse_doc(
            "<div class=\"item\">A</div><span class=\"item\">B</span><div class=\"item\">C</div>",
        );
        let filter = Filter::new().tag("div").class("item");

        let results = find_by_filter(&doc, &filter);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_find_first_by_filter() {
        let doc = parse_doc("<div id=\"first\">A</div><div id=\"second\">B</div>");
        let filter = Filter::new().tag("div");

        let result = find_first_by_filter(&doc, &filter);
        assert!(result.is_some());

        let id = result.unwrap();
        let attrs = doc.get(id).unwrap().kind.attributes().unwrap();
        assert_eq!(attrs.get("id"), Some(&"first".to_string()));
    }

    #[test]
    fn test_find_by_filter_empty_document() {
        let doc = Document::new();
        let filter = Filter::new().tag("div");

        let results = find_by_filter(&doc, &filter);
        assert!(results.is_empty());
    }

    #[test]
    fn test_find_first_by_filter_no_match() {
        let doc = parse_doc("<span>text</span>");
        let filter = Filter::new().tag("div");

        let result = find_first_by_filter(&doc, &filter);
        assert!(result.is_none());
    }

    #[test]
    fn test_filter_equality() {
        let f1 = Filter::new().tag("div").class("foo");
        let f2 = Filter::new().tag("div").class("foo");
        let f3 = Filter::new().tag("div").class("bar");

        assert_eq!(f1, f2);
        assert_ne!(f1, f3);
    }

    #[test]
    fn test_filter_clone() {
        let f1 = Filter::new().tag("div").class("foo").id("main");
        let f2 = f1.clone();

        assert_eq!(f1, f2);
    }
}
