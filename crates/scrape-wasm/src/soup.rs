//! Soup document wrapper for WASM.

use std::rc::Rc;

use scrape_core::Soup as CoreSoup;
use wasm_bindgen::prelude::*;

use crate::{config::SoupConfig, tag::Tag};

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
}

impl Soup {
    /// Returns a clone of the inner Rc for use by Tag.
    #[must_use]
    pub fn inner_rc(&self) -> Rc<CoreSoup> {
        Rc::clone(&self.inner)
    }
}
