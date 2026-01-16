//! Node.js wrapper for Soup document.

use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use scrape_core::Soup as CoreSoup;

use crate::{config::SoupConfig, error::IntoNapiError, tag::Tag};

/// A parsed HTML document.
///
/// @example
/// ```javascript
/// import { Soup } from 'scrape-rs';
///
/// const soup = new Soup("<div>Hello</div>");
/// const div = soup.find("div");
/// console.log(div.text); // "Hello"
/// ```
#[napi]
pub struct Soup {
    pub(crate) inner: Arc<CoreSoup>,
}

#[napi]
impl Soup {
    /// Parse an HTML string into a Soup document.
    ///
    /// @param html - HTML string to parse
    /// @param config - Optional parsing configuration
    #[napi(constructor)]
    pub fn new(html: String, config: Option<SoupConfig>) -> Self {
        let core_config = config.map(|c| c.to_core()).unwrap_or_default();

        let soup = CoreSoup::parse_with_config(&html, core_config);
        Self { inner: Arc::new(soup) }
    }

    /// Parse HTML from a file.
    ///
    /// @param path - Path to the HTML file
    /// @param config - Optional parsing configuration
    /// @returns A new Soup instance
    /// @throws Error if the file cannot be read
    #[napi(factory)]
    pub fn from_file(path: String, config: Option<SoupConfig>) -> Result<Self> {
        let html = std::fs::read_to_string(&path).map_err(IntoNapiError::into_napi_error)?;
        Ok(Self::new(html, config))
    }

    /// Find the first element matching a CSS selector.
    ///
    /// @param selector - CSS selector string
    /// @returns The first matching Tag, or null if not found
    /// @throws Error if the selector syntax is invalid
    #[napi]
    pub fn find(&self, selector: String) -> Result<Option<Tag>> {
        self.inner
            .find(&selector)
            .map_err(IntoNapiError::into_napi_error)
            .map(|opt| opt.map(|tag| Tag::new(Arc::clone(&self.inner), tag.node_id())))
    }

    /// Find all elements matching a CSS selector.
    ///
    /// @param selector - CSS selector string
    /// @returns Array of matching Tag instances
    /// @throws Error if the selector syntax is invalid
    #[napi(js_name = "findAll")]
    pub fn find_all(&self, selector: String) -> Result<Vec<Tag>> {
        self.inner.find_all(&selector).map_err(IntoNapiError::into_napi_error).map(|tags| {
            tags.into_iter().map(|tag| Tag::new(Arc::clone(&self.inner), tag.node_id())).collect()
        })
    }

    /// Find all elements matching a CSS selector (alias for findAll).
    ///
    /// @param selector - CSS selector string
    /// @returns Array of matching Tag instances
    #[napi]
    pub fn select(&self, selector: String) -> Result<Vec<Tag>> {
        self.find_all(selector)
    }

    /// Get the root element of the document.
    ///
    /// @returns The root Tag (usually <html>), or null for empty documents
    #[napi(getter)]
    pub fn root(&self) -> Option<Tag> {
        self.inner.root().map(|tag| Tag::new(Arc::clone(&self.inner), tag.node_id()))
    }

    /// Get the document title.
    ///
    /// @returns The title text, or null if no <title> element exists
    #[napi(getter)]
    pub fn title(&self) -> Option<String> {
        self.inner.title()
    }

    /// Get the text content of the entire document.
    ///
    /// @returns All text content with HTML tags stripped
    #[napi(getter)]
    pub fn text(&self) -> String {
        self.inner.text()
    }

    /// Get the HTML representation of the document.
    ///
    /// @returns The document as an HTML string
    #[napi(js_name = "toHtml")]
    pub fn to_html(&self) -> String {
        self.inner.to_html()
    }

    /// Get the number of nodes in the document.
    #[napi(getter)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn length(&self) -> u32 {
        self.inner.document().len() as u32
    }
}
