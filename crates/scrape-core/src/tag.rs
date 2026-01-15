//! HTML element type.
//!
//! The [`Tag`] struct represents a single HTML element in the DOM tree.

use crate::Result;

/// An HTML element in the DOM tree.
///
/// `Tag` provides methods for accessing element properties, attributes,
/// and navigating the DOM tree.
///
/// # Examples
///
/// ## Accessing Attributes
///
/// ```rust,ignore
/// use scrape_core::Soup;
///
/// let soup = Soup::parse("<a href=\"https://example.com\">Link</a>");
/// let link = soup.find("a").unwrap();
///
/// assert_eq!(link.get("href"), Some("https://example.com".to_string()));
/// assert_eq!(link.text(), "Link");
/// ```
///
/// ## Tree Navigation
///
/// ```rust,ignore
/// use scrape_core::Soup;
///
/// let soup = Soup::parse("<div><span>Child</span></div>");
/// let span = soup.find("span").unwrap();
///
/// if let Some(parent) = span.parent() {
///     assert_eq!(parent.name(), "div");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Tag {
    tag_name: String,
}

impl Tag {
    /// Creates a new Tag (internal use only).
    #[must_use]
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self { tag_name: name.into() }
    }

    /// Returns the tag name (e.g., "div", "span", "a").
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div></div>");
    /// let div = soup.find("div").unwrap();
    /// assert_eq!(div.name(), "div");
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        &self.tag_name
    }

    /// Returns the value of an attribute, if present.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<a href=\"/page\">Link</a>");
    /// let link = soup.find("a").unwrap();
    /// assert_eq!(link.get("href"), Some("/page".to_string()));
    /// assert_eq!(link.get("class"), None);
    /// ```
    #[must_use]
    pub fn get(&self, _attr: &str) -> Option<String> {
        // TODO: implement attribute access
        todo!("Tag::get")
    }

    /// Returns the value of an attribute, or an error if not present.
    ///
    /// # Errors
    ///
    /// Returns [`Error::AttributeNotFound`] if the attribute does not exist.
    ///
    /// [`Error::AttributeNotFound`]: crate::Error::AttributeNotFound
    pub fn get_or_err(&self, attr: &str) -> Result<String> {
        self.get(attr).ok_or_else(|| crate::Error::attribute_not_found(attr))
    }

    /// Returns the text content of this element and its descendants.
    ///
    /// HTML tags are stripped and only text nodes are included.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div>Hello <b>World</b>!</div>");
    /// let div = soup.find("div").unwrap();
    /// assert_eq!(div.text(), "Hello World!");
    /// ```
    #[must_use]
    pub fn text(&self) -> String {
        // TODO: implement text extraction
        todo!("Tag::text")
    }

    /// Returns the inner HTML of this element.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div><span>Hello</span></div>");
    /// let div = soup.find("div").unwrap();
    /// assert_eq!(div.inner_html(), "<span>Hello</span>");
    /// ```
    #[must_use]
    pub fn inner_html(&self) -> String {
        // TODO: implement inner HTML
        todo!("Tag::inner_html")
    }

    /// Returns the outer HTML of this element (including the tag itself).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div><span>Hello</span></div>");
    /// let span = soup.find("span").unwrap();
    /// assert_eq!(span.outer_html(), "<span>Hello</span>");
    /// ```
    #[must_use]
    pub fn outer_html(&self) -> String {
        // TODO: implement outer HTML
        todo!("Tag::outer_html")
    }

    /// Returns all attribute names on this element.
    #[must_use]
    pub fn attrs(&self) -> Vec<String> {
        // TODO: implement attribute listing
        todo!("Tag::attrs")
    }

    /// Checks if this element has the specified class.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div class=\"foo bar\"></div>");
    /// let div = soup.find("div").unwrap();
    /// assert!(div.has_class("foo"));
    /// assert!(div.has_class("bar"));
    /// assert!(!div.has_class("baz"));
    /// ```
    #[must_use]
    pub fn has_class(&self, _class: &str) -> bool {
        // TODO: implement class checking
        todo!("Tag::has_class")
    }

    /// Returns the parent element, if any.
    ///
    /// Returns `None` for the root element.
    #[must_use]
    pub fn parent(&self) -> Option<Tag> {
        // TODO: implement parent navigation
        todo!("Tag::parent")
    }

    /// Returns an iterator over direct child elements.
    pub fn children(&self) -> impl Iterator<Item = Tag> {
        // TODO: implement children iteration
        std::iter::empty()
    }

    /// Returns the next sibling element.
    #[must_use]
    pub fn next_sibling(&self) -> Option<Tag> {
        // TODO: implement sibling navigation
        todo!("Tag::next_sibling")
    }

    /// Returns the previous sibling element.
    #[must_use]
    pub fn prev_sibling(&self) -> Option<Tag> {
        // TODO: implement sibling navigation
        todo!("Tag::prev_sibling")
    }

    /// Finds the first descendant matching the selector.
    #[must_use]
    pub fn find(&self, _selector: &str) -> Option<Tag> {
        // TODO: implement find within element
        todo!("Tag::find")
    }

    /// Finds all descendants matching the selector.
    pub fn find_all(&self, _selector: &str) -> impl Iterator<Item = Tag> {
        // TODO: implement find_all within element
        std::iter::empty()
    }

    /// Selects descendants using a CSS selector.
    ///
    /// Alias for [`Tag::find_all`].
    pub fn select(&self, selector: &str) -> impl Iterator<Item = Tag> {
        self.find_all(selector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_name() {
        let tag = Tag::new("div");
        assert_eq!(tag.name(), "div");
    }

    #[test]
    fn test_tag_new() {
        let tag = Tag::new(String::from("span"));
        assert_eq!(tag.name(), "span");
    }
}
