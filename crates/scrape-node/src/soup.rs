//! Node.js wrapper for Soup document.

use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use scrape_core::Soup as CoreSoup;

use crate::{config::SoupConfig, error::IntoNapiError, selector::CompiledSelector, tag::Tag};

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

    // ==================== Compiled Selector Methods ====================

    /// Find the first element matching a compiled selector.
    ///
    /// @param selector - A compiled CSS selector
    /// @returns The first matching Tag, or null if not found
    #[napi(js_name = "findCompiled")]
    pub fn find_compiled(&self, selector: &CompiledSelector) -> Option<Tag> {
        self.inner
            .find_compiled(&selector.inner)
            .map(|tag| Tag::new(Arc::clone(&self.inner), tag.node_id()))
    }

    /// Find all elements matching a compiled selector.
    ///
    /// @param selector - A compiled CSS selector
    /// @returns Array of matching Tag instances
    #[napi(js_name = "selectCompiled")]
    pub fn select_compiled(&self, selector: &CompiledSelector) -> Vec<Tag> {
        self.inner
            .select_compiled(&selector.inner)
            .into_iter()
            .map(|tag| Tag::new(Arc::clone(&self.inner), tag.node_id()))
            .collect()
    }

    // ==================== Fragment Parsing ====================

    /// Parse an HTML fragment without html/body wrapper.
    ///
    /// @param html - HTML fragment string to parse
    /// @param context - Optional context element name (default: "body")
    /// @param config - Optional parsing configuration
    /// @returns A new Soup instance containing the fragment
    ///
    /// @example
    /// ```javascript
    /// import { Soup } from 'scrape-rs';
    ///
    /// // Parse without wrapper
    /// const soup = Soup.parseFragment("<div>A</div><div>B</div>");
    /// const divs = soup.findAll("div");
    /// console.log(divs.length); // 2
    ///
    /// // Parse with td context
    /// const tdSoup = Soup.parseFragment("<td>Cell</td>", "tr");
    /// ```
    #[napi(factory, js_name = "parseFragment")]
    pub fn parse_fragment(
        html: String,
        context: Option<String>,
        config: Option<SoupConfig>,
    ) -> Self {
        let core_config = config.map(|c| c.to_core()).unwrap_or_default();
        let ctx = context.as_deref().unwrap_or("body");

        let soup = CoreSoup::parse_fragment_with_config(&html, ctx, core_config);
        Self { inner: Arc::new(soup) }
    }

    // ==================== Extraction Methods ====================

    /// Extract text content from all elements matching a selector.
    ///
    /// @param selector - CSS selector string
    /// @returns Array of text content strings
    /// @throws Error if the selector syntax is invalid
    ///
    /// @example
    /// ```javascript
    /// const soup = new Soup("<div>A</div><div>B</div>");
    /// const texts = soup.selectText("div");
    /// // texts: ["A", "B"]
    /// ```
    #[napi(js_name = "selectText")]
    pub fn select_text(&self, selector: String) -> Result<Vec<String>> {
        self.inner.select_text(&selector).map_err(IntoNapiError::into_napi_error)
    }

    /// Extract attribute values from all elements matching a selector.
    ///
    /// @param selector - CSS selector string
    /// @param attr - Attribute name to extract
    /// @returns Array of attribute values (null if attribute is missing)
    /// @throws Error if the selector syntax is invalid
    ///
    /// @example
    /// ```javascript
    /// const soup = new Soup("<a href='/a'>A</a><a href='/b'>B</a><a>C</a>");
    /// const hrefs = soup.selectAttr("a", "href");
    /// // hrefs: ["/a", "/b", null]
    /// ```
    #[napi(js_name = "selectAttr")]
    pub fn select_attr(&self, selector: String, attr: String) -> Result<Vec<Option<String>>> {
        self.inner.select_attr(&selector, &attr).map_err(IntoNapiError::into_napi_error)
    }
}
