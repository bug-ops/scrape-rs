//! WebAssembly bindings for scrape-rs.
//!
//! This module provides WASM bindings for the scrape-core library using wasm-bindgen.
//! It exposes a high-level API for parsing and querying HTML documents in browsers,
//! Deno, and edge environments (Cloudflare Workers, Vercel Edge).
//!
//! # Example
//!
//! ```javascript
//! import init, { Soup, parseBatch, hasSimdSupport, version } from '@scrape-rs/wasm';
//!
//! await init();
//!
//! // Parse single document
//! const soup = new Soup("<div class='item'>Hello</div>");
//! const div = soup.find("div.item");
//! console.log(div.text); // "Hello"
//!
//! // Batch processing (sequential in WASM)
//! const soups = parseBatch(["<div>A</div>", "<div>B</div>"]);
//!
//! // Check SIMD support
//! console.log("SIMD:", hasSimdSupport());
//! console.log("Version:", version());
//! ```

use wasm_bindgen::prelude::*;

mod config;
mod selector;
mod soup;
mod tag;

pub use config::SoupConfig;
pub use selector::CompiledSelector;
pub use soup::Soup;
pub use tag::Tag;

/// Initialize the WASM module.
///
/// Sets up panic hook for better error messages in browser console.
/// This is called automatically when the module is loaded.
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Parse multiple HTML documents.
///
/// Note: WASM does not support threads, so this processes documents sequentially.
/// For parallel processing in browsers, use Web Workers with separate WASM instances.
///
/// @param documents - Array of HTML strings to parse
/// @returns Array of Soup documents
///
/// @example
/// ```javascript
/// const soups = parseBatch(['<div>A</div>', '<div>B</div>']);
/// console.log(soups.length); // 2
/// ```
#[wasm_bindgen(js_name = "parseBatch")]
pub fn parse_batch(documents: Vec<String>) -> Vec<Soup> {
    documents.into_iter().map(|html| Soup::new(&html, None)).collect()
}

/// Check if WASM SIMD is supported in the current environment.
///
/// Returns true if the module was compiled with SIMD support and
/// is running on a platform that supports SIMD128 instructions.
///
/// SIMD support requires:
/// - Chrome 91+ / Firefox 89+ / Safari 16.4+
/// - Module built with RUSTFLAGS='-C target-feature=+simd128'
#[wasm_bindgen(js_name = "hasSimdSupport")]
pub fn has_simd_support() -> bool {
    cfg!(target_feature = "simd128")
}

/// Get the library version.
///
/// @returns Version string (e.g., "0.1.0")
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
