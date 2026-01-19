//! WASM wrapper for CompiledSelector.

use scrape_core::query::CompiledSelector as CoreCompiledSelector;
use wasm_bindgen::prelude::*;

/// A pre-compiled CSS selector for efficient repeated matching.
///
/// Compiled selectors avoid the overhead of parsing the selector string on each query.
///
/// @example
/// ```javascript
/// import init, { CompiledSelector, Soup } from '@scrape-rs/wasm';
///
/// await init();
///
/// const selector = CompiledSelector.compile("div.item");
/// const soup = new Soup("<div class='item'>A</div><div class='item'>B</div>");
/// const items = soup.selectCompiled(selector);
/// console.log(items.length); // 2
/// ```
#[wasm_bindgen]
pub struct CompiledSelector {
    pub(crate) inner: CoreCompiledSelector,
}

#[wasm_bindgen]
impl CompiledSelector {
    /// Compile a CSS selector string.
    ///
    /// @param selector - The CSS selector to compile
    /// @returns A compiled selector
    /// @throws Error if the selector syntax is invalid
    pub fn compile(selector: &str) -> Result<CompiledSelector, JsError> {
        CoreCompiledSelector::compile(selector)
            .map(|inner| Self { inner })
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Get the original selector string.
    ///
    /// @returns The selector string that was compiled
    #[wasm_bindgen(getter)]
    pub fn source(&self) -> String {
        self.inner.source().to_string()
    }
}
