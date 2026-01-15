//! WebAssembly bindings for scrape-rs.
//!
//! This module provides WASM bindings for the scrape-core library using wasm-bindgen.

use wasm_bindgen::prelude::*;

/// Initialize the WASM module.
///
/// This should be called once before using any other functions.
#[wasm_bindgen(start)]
pub fn init() {
    // TODO: add console_error_panic_hook feature for better error messages in browser console
}

/// Configuration options for HTML parsing.
#[wasm_bindgen]
#[derive(Debug, Clone, Default)]
pub struct SoupConfig {
    max_depth: usize,
    strict_mode: bool,
}

#[wasm_bindgen]
impl SoupConfig {
    /// Creates a new configuration with default values.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { max_depth: 256, strict_mode: false }
    }

    /// Sets the maximum nesting depth.
    #[wasm_bindgen(js_name = "setMaxDepth")]
    pub fn set_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Sets strict parsing mode.
    #[wasm_bindgen(js_name = "setStrictMode")]
    pub fn set_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Returns the maximum depth setting.
    #[wasm_bindgen(getter, js_name = "maxDepth")]
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Returns the strict mode setting.
    #[wasm_bindgen(getter, js_name = "strictMode")]
    pub fn strict_mode(&self) -> bool {
        self.strict_mode
    }
}

impl From<SoupConfig> for scrape_core::SoupConfig {
    fn from(config: SoupConfig) -> Self {
        scrape_core::SoupConfig::builder()
            .max_depth(config.max_depth)
            .strict_mode(config.strict_mode)
            .build()
    }
}

/// A parsed HTML document.
#[wasm_bindgen]
pub struct Soup {
    inner: scrape_core::Soup,
}

#[wasm_bindgen]
impl Soup {
    /// Parses an HTML string into a Soup document.
    #[wasm_bindgen(constructor)]
    pub fn new(html: &str) -> Self {
        Self { inner: scrape_core::Soup::parse(html) }
    }

    /// Parses an HTML string with custom configuration.
    #[wasm_bindgen(js_name = "parseWithConfig")]
    pub fn parse_with_config(html: &str, config: SoupConfig) -> Self {
        Self { inner: scrape_core::Soup::parse_with_config(html, config.into()) }
    }

    /// Returns the document title if present.
    #[wasm_bindgen(getter)]
    pub fn title(&self) -> Option<String> {
        self.inner.title()
    }

    /// Returns the text content of the document.
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> String {
        self.inner.text()
    }

    /// Returns the HTML representation of the document.
    #[wasm_bindgen(js_name = "toHtml")]
    pub fn to_html(&self) -> String {
        self.inner.to_html()
    }
}

/// Parse multiple HTML documents.
///
/// Note: WASM does not support threads, so this processes documents sequentially.
#[wasm_bindgen(js_name = "parseBatch")]
pub fn parse_batch(documents: Vec<String>) -> Vec<Soup> {
    documents.into_iter().map(|html| Soup::new(&html)).collect()
}

/// Check if WASM SIMD is supported.
#[wasm_bindgen(js_name = "hasSimdSupport")]
pub fn has_simd_support() -> bool {
    // In modern browsers, SIMD128 is widely supported
    cfg!(target_feature = "simd128")
}
