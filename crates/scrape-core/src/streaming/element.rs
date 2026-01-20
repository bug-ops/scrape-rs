//! Streaming element wrapper for `lol_html` Element.

#[cfg(feature = "streaming")]
use crate::{Error, Result};

/// Content type for element manipulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    /// Plain text content (will be escaped).
    Text,
    /// HTML content (will not be escaped).
    Html,
}

/// Wrapper around `lol_html`'s Element type providing safe, ergonomic API.
///
/// This type wraps `lol_html`'s `Element` to provide a stable API that is
/// independent of upstream changes in `lol_html`.
#[cfg(feature = "streaming")]
pub struct StreamingElement<'r, 's> {
    inner: lol_html::html_content::Element<'r, 's>,
}

#[cfg(feature = "streaming")]
impl<'r, 's> StreamingElement<'r, 's> {
    /// Creates a new `StreamingElement` from `lol_html`'s Element.
    #[must_use]
    pub(crate) fn new(element: lol_html::html_content::Element<'r, 's>) -> Self {
        Self { inner: element }
    }

    /// Returns the tag name of this element.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let name = element.tag_name();
    /// assert_eq!(name, "div");
    /// ```
    #[must_use]
    pub fn tag_name(&self) -> String {
        self.inner.tag_name()
    }

    /// Checks if this element has the specified attribute.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if element.has_attribute("href") {
    ///     println!("Element has href");
    /// }
    /// ```
    #[must_use]
    pub fn has_attribute(&self, name: &str) -> bool {
        self.inner.has_attribute(name)
    }

    /// Gets the value of an attribute by name.
    ///
    /// Returns `None` if the attribute does not exist.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(href) = element.get_attribute("href") {
    ///     println!("Link target: {}", href);
    /// }
    /// ```
    #[must_use]
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        self.inner.get_attribute(name)
    }

    /// Returns an iterator over all attributes of this element.
    ///
    /// Each item is a tuple of (name, value).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// for (name, value) in element.attributes() {
    ///     println!("{}={}", name, value);
    /// }
    /// ```
    pub fn attributes(&self) -> impl Iterator<Item = (String, String)> + '_ {
        self.inner.attributes().iter().map(|attr| (attr.name(), attr.value()))
    }

    /// Sets an attribute on this element.
    ///
    /// If the attribute already exists, its value is updated.
    ///
    /// # Errors
    ///
    /// Returns an error if the attribute name is invalid.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// element.set_attribute("class", "active")?;
    /// ```
    pub fn set_attribute(&mut self, name: &str, value: &str) -> Result<()> {
        self.inner
            .set_attribute(name, value)
            .map_err(|e| Error::handler_error(format!("failed to set attribute: {e}")))?;
        Ok(())
    }

    /// Removes an attribute from this element.
    ///
    /// Does nothing if the attribute does not exist.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// element.remove_attribute("disabled");
    /// ```
    pub fn remove_attribute(&mut self, name: &str) {
        self.inner.remove_attribute(name);
    }

    /// Inserts content before this element.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// element.before("<p>Before</p>", ContentType::Html);
    /// ```
    pub fn before(&mut self, content: &str, content_type: ContentType) {
        match content_type {
            ContentType::Text => {
                self.inner.before(content, lol_html::html_content::ContentType::Text);
            }
            ContentType::Html => {
                self.inner.before(content, lol_html::html_content::ContentType::Html);
            }
        }
    }

    /// Inserts content after this element.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// element.after("<p>After</p>", ContentType::Html);
    /// ```
    pub fn after(&mut self, content: &str, content_type: ContentType) {
        match content_type {
            ContentType::Text => {
                self.inner.after(content, lol_html::html_content::ContentType::Text);
            }
            ContentType::Html => {
                self.inner.after(content, lol_html::html_content::ContentType::Html);
            }
        }
    }

    /// Prepends content as the first child of this element.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// element.prepend("<span>First</span>", ContentType::Html);
    /// ```
    pub fn prepend(&mut self, content: &str, content_type: ContentType) {
        match content_type {
            ContentType::Text => {
                self.inner.prepend(content, lol_html::html_content::ContentType::Text);
            }
            ContentType::Html => {
                self.inner.prepend(content, lol_html::html_content::ContentType::Html);
            }
        }
    }

    /// Appends content as the last child of this element.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// element.append("<span>Last</span>", ContentType::Html);
    /// ```
    pub fn append(&mut self, content: &str, content_type: ContentType) {
        match content_type {
            ContentType::Text => {
                self.inner.append(content, lol_html::html_content::ContentType::Text);
            }
            ContentType::Html => {
                self.inner.append(content, lol_html::html_content::ContentType::Html);
            }
        }
    }

    /// Sets the inner content of this element, replacing all children.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// element.set_inner_content("<p>New content</p>", ContentType::Html);
    /// ```
    pub fn set_inner_content(&mut self, content: &str, content_type: ContentType) {
        match content_type {
            ContentType::Text => {
                self.inner.set_inner_content(content, lol_html::html_content::ContentType::Text);
            }
            ContentType::Html => {
                self.inner.set_inner_content(content, lol_html::html_content::ContentType::Html);
            }
        }
    }

    /// Replaces this element with the given content.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// element.replace("<div>Replacement</div>", ContentType::Html);
    /// ```
    pub fn replace(&mut self, content: &str, content_type: ContentType) {
        match content_type {
            ContentType::Text => {
                self.inner.replace(content, lol_html::html_content::ContentType::Text);
            }
            ContentType::Html => {
                self.inner.replace(content, lol_html::html_content::ContentType::Html);
            }
        }
    }

    /// Removes this element from the document.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// element.remove();
    /// ```
    pub fn remove(&mut self) {
        self.inner.remove();
    }

    /// Removes this element but keeps its content.
    ///
    /// This unwraps the element, leaving its children in place.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // <div><span>text</span></div> becomes <span>text</span>
    /// element.remove_and_keep_content();
    /// ```
    pub fn remove_and_keep_content(&mut self) {
        self.inner.remove_and_keep_content();
    }
}

#[cfg(all(test, feature = "streaming"))]
mod tests {
    use super::*;

    #[test]
    fn test_content_type() {
        assert_eq!(ContentType::Text, ContentType::Text);
        assert_eq!(ContentType::Html, ContentType::Html);
        assert_ne!(ContentType::Text, ContentType::Html);
    }
}
