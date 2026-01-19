//! Node.js wrapper for CompiledSelector.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use scrape_core::query::CompiledSelector as CoreCompiledSelector;

use crate::error::IntoNapiError;

/// A pre-compiled CSS selector for efficient repeated matching.
///
/// Compiled selectors avoid the overhead of parsing the selector string on each query.
///
/// @example
/// ```javascript
/// import { CompiledSelector, Soup } from 'scrape-rs';
///
/// const selector = CompiledSelector.compile("div.item");
/// const soup = new Soup("<div class='item'>A</div><div class='item'>B</div>");
/// const items = soup.selectCompiled(selector);
/// console.log(items.length); // 2
/// ```
#[napi]
pub struct CompiledSelector {
    pub(crate) inner: CoreCompiledSelector,
}

#[napi]
impl CompiledSelector {
    /// Compile a CSS selector string.
    ///
    /// @param selector - The CSS selector to compile
    /// @returns A compiled selector
    /// @throws Error if the selector syntax is invalid
    #[napi(factory)]
    pub fn compile(selector: String) -> Result<Self> {
        CoreCompiledSelector::compile(&selector)
            .map(|inner| Self { inner })
            .map_err(IntoNapiError::into_napi_error)
    }

    /// Get the original selector string.
    ///
    /// @returns The selector string that was compiled
    #[napi(getter)]
    pub fn source(&self) -> String {
        self.inner.source().to_string()
    }
}
