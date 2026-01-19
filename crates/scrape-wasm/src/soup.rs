//! Soup document wrapper for WASM.

use std::rc::Rc;

use scrape_core::Soup as CoreSoup;
use wasm_bindgen::prelude::*;

use crate::{config::SoupConfig, selector::CompiledSelector, tag::Tag};

/// A parsed HTML document.
///
/// `Soup` is the main entry point for parsing and querying HTML documents.
/// It provides methods for finding elements by CSS selector.
///
/// @example
/// ```javascript
/// import init, { Soup } from '@scrape-rs/wasm';
///
/// await init();
///
/// const soup = new Soup("<div class='item'>Hello</div>");
/// const div = soup.find("div.item");
/// console.log(div.text); // "Hello"
/// ```
#[wasm_bindgen]
pub struct Soup {
    inner: Rc<CoreSoup>,
}

#[wasm_bindgen]
impl Soup {
    /// Parses an HTML string into a Soup document.
    ///
    /// @param html - The HTML string to parse
    /// @param config - Optional configuration options
    #[wasm_bindgen(constructor)]
    pub fn new(html: &str, config: Option<SoupConfig>) -> Self {
        let core_config = config.map(|c| c.to_core()).unwrap_or_default();
        let soup = CoreSoup::parse_with_config(html, core_config);
        Self { inner: Rc::new(soup) }
    }

    /// Finds the first element matching a CSS selector.
    ///
    /// @param selector - CSS selector string
    /// @returns The first matching Tag, or undefined if not found
    /// @throws Error if the selector syntax is invalid
    pub fn find(&self, selector: &str) -> Result<Option<Tag>, JsError> {
        self.inner
            .find(selector)
            .map_err(|e| JsError::new(&e.to_string()))
            .map(|opt| opt.map(|tag| Tag::new(Rc::clone(&self.inner), tag.node_id())))
    }

    /// Finds all elements matching a CSS selector.
    ///
    /// @param selector - CSS selector string
    /// @returns Array of matching Tag instances
    /// @throws Error if the selector syntax is invalid
    #[wasm_bindgen(js_name = "findAll")]
    pub fn find_all(&self, selector: &str) -> Result<Vec<Tag>, JsError> {
        self.inner.find_all(selector).map_err(|e| JsError::new(&e.to_string())).map(|tags| {
            tags.into_iter().map(|tag| Tag::new(Rc::clone(&self.inner), tag.node_id())).collect()
        })
    }

    /// Finds all elements matching a CSS selector (alias for findAll).
    ///
    /// @param selector - CSS selector string
    /// @returns Array of matching Tag instances
    pub fn select(&self, selector: &str) -> Result<Vec<Tag>, JsError> {
        self.find_all(selector)
    }

    /// Get the root element of the document.
    ///
    /// @returns The root Tag (usually <html>), or undefined for empty documents
    #[wasm_bindgen(getter)]
    pub fn root(&self) -> Option<Tag> {
        self.inner.root().map(|tag| Tag::new(Rc::clone(&self.inner), tag.node_id()))
    }

    /// Get the document title.
    ///
    /// @returns The title text, or undefined if no <title> element exists
    #[wasm_bindgen(getter)]
    pub fn title(&self) -> Option<String> {
        self.inner.title()
    }

    /// Get the text content of the entire document.
    ///
    /// @returns All text content with HTML tags stripped
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> String {
        self.inner.text()
    }

    /// Get the HTML representation of the document.
    ///
    /// @returns The document as an HTML string
    #[wasm_bindgen(js_name = "toHtml")]
    pub fn to_html(&self) -> String {
        self.inner.to_html()
    }

    /// Get the number of nodes in the document.
    #[wasm_bindgen(getter)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn length(&self) -> u32 {
        self.inner.document().len() as u32
    }

    // ==================== Compiled Selector Methods ====================

    /// Find the first element matching a compiled selector.
    ///
    /// @param selector - A compiled CSS selector
    /// @returns The first matching Tag, or undefined if not found
    #[wasm_bindgen(js_name = "findCompiled")]
    pub fn find_compiled(&self, selector: &CompiledSelector) -> Option<Tag> {
        self.inner
            .find_compiled(&selector.inner)
            .map(|tag| Tag::new(Rc::clone(&self.inner), tag.node_id()))
    }

    /// Find all elements matching a compiled selector.
    ///
    /// @param selector - A compiled CSS selector
    /// @returns Array of matching Tag instances
    #[wasm_bindgen(js_name = "selectCompiled")]
    pub fn select_compiled(&self, selector: &CompiledSelector) -> Vec<Tag> {
        self.inner
            .select_compiled(&selector.inner)
            .into_iter()
            .map(|tag| Tag::new(Rc::clone(&self.inner), tag.node_id()))
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
    /// // Parse without wrapper
    /// const soup = Soup.parseFragment("<div>A</div><div>B</div>");
    /// const divs = soup.findAll("div");
    /// console.log(divs.length); // 2
    ///
    /// // Parse with td context
    /// const tdSoup = Soup.parseFragment("<td>Cell</td>", "tr");
    /// ```
    #[wasm_bindgen(js_name = "parseFragment")]
    pub fn parse_fragment(html: &str, context: Option<String>, config: Option<SoupConfig>) -> Self {
        let core_config = config.map(|c| c.to_core()).unwrap_or_default();
        let ctx = context.as_deref().unwrap_or("body");

        let soup = CoreSoup::parse_fragment_with_config(html, ctx, core_config);
        Self { inner: Rc::new(soup) }
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
    #[wasm_bindgen(js_name = "selectText")]
    pub fn select_text(&self, selector: &str) -> Result<Vec<String>, JsError> {
        self.inner.select_text(selector).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Extract attribute values from all elements matching a selector.
    ///
    /// @param selector - CSS selector string
    /// @param attr - Attribute name to extract
    /// @returns Array of attribute values (undefined if attribute is missing)
    /// @throws Error if the selector syntax is invalid
    ///
    /// @example
    /// ```javascript
    /// const soup = new Soup("<a href='/a'>A</a><a href='/b'>B</a><a>C</a>");
    /// const hrefs = soup.selectAttr("a", "href");
    /// // hrefs: ["/a", "/b", undefined]
    /// ```
    #[wasm_bindgen(js_name = "selectAttr")]
    pub fn select_attr(&self, selector: &str, attr: &str) -> Result<Vec<JsValue>, JsError> {
        self.inner.select_attr(selector, attr).map_err(|e| JsError::new(&e.to_string())).map(
            |values| {
                values
                    .into_iter()
                    .map(|opt| opt.map_or(JsValue::UNDEFINED, JsValue::from))
                    .collect()
            },
        )
    }
}

impl Soup {
    /// Returns a clone of the inner Rc for use by Tag.
    #[must_use]
    pub fn inner_rc(&self) -> Rc<CoreSoup> {
        Rc::clone(&self.inner)
    }
}
