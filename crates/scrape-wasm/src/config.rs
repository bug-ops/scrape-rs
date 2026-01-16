//! Configuration options for HTML parsing.

use wasm_bindgen::prelude::*;

/// Configuration options for HTML parsing.
///
/// All options have sensible defaults. Use setters to customize behavior.
///
/// @example
/// ```javascript
/// const config = new SoupConfig();
/// config.maxDepth = 256;
/// config.strictMode = true;
/// const soup = new Soup("<div>Hello</div>", config);
/// ```
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SoupConfig {
    max_depth: u32,
    strict_mode: bool,
    preserve_whitespace: bool,
    include_comments: bool,
}

impl Default for SoupConfig {
    fn default() -> Self {
        Self {
            max_depth: 512,
            strict_mode: false,
            preserve_whitespace: false,
            include_comments: false,
        }
    }
}

#[wasm_bindgen]
impl SoupConfig {
    /// Creates a new configuration with default values.
    ///
    /// Default values:
    /// - maxDepth: 512
    /// - strictMode: false
    /// - preserveWhitespace: false
    /// - includeComments: false
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Maximum nesting depth for DOM tree.
    #[wasm_bindgen(getter, js_name = "maxDepth")]
    pub fn max_depth(&self) -> u32 {
        self.max_depth
    }

    /// Sets the maximum nesting depth.
    #[wasm_bindgen(setter, js_name = "maxDepth")]
    pub fn set_max_depth(&mut self, value: u32) {
        self.max_depth = value;
    }

    /// Enable strict parsing mode (fail on malformed HTML).
    #[wasm_bindgen(getter, js_name = "strictMode")]
    pub fn strict_mode(&self) -> bool {
        self.strict_mode
    }

    /// Sets strict parsing mode.
    #[wasm_bindgen(setter, js_name = "strictMode")]
    pub fn set_strict_mode(&mut self, value: bool) {
        self.strict_mode = value;
    }

    /// Preserve whitespace-only text nodes.
    #[wasm_bindgen(getter, js_name = "preserveWhitespace")]
    pub fn preserve_whitespace(&self) -> bool {
        self.preserve_whitespace
    }

    /// Sets whitespace preservation.
    #[wasm_bindgen(setter, js_name = "preserveWhitespace")]
    pub fn set_preserve_whitespace(&mut self, value: bool) {
        self.preserve_whitespace = value;
    }

    /// Include comment nodes in DOM.
    #[wasm_bindgen(getter, js_name = "includeComments")]
    pub fn include_comments(&self) -> bool {
        self.include_comments
    }

    /// Sets comment inclusion.
    #[wasm_bindgen(setter, js_name = "includeComments")]
    pub fn set_include_comments(&mut self, value: bool) {
        self.include_comments = value;
    }
}

impl SoupConfig {
    /// Converts to core SoupConfig.
    #[must_use]
    pub fn to_core(&self) -> scrape_core::SoupConfig {
        scrape_core::SoupConfig::builder()
            .max_depth(self.max_depth as usize)
            .strict_mode(self.strict_mode)
            .preserve_whitespace(self.preserve_whitespace)
            .include_comments(self.include_comments)
            .build()
    }
}
