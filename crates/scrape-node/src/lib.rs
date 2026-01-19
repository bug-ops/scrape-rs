//! Node.js bindings for scrape-rs.
//!
//! This module provides Node.js bindings for the scrape-core library using napi-rs.
//!
//! ## Quick Start
//!
//! ```javascript
//! import { Soup, parseBatch } from 'scrape-rs';
//!
//! // Parse single document
//! const soup = new Soup("<div class='item'>Hello</div>");
//! const div = soup.find("div.item");
//! console.log(div.text); // "Hello"
//!
//! // Parse multiple documents in parallel
//! const htmls = ['<div>A</div>', '<div>B</div>'];
//! const soups = parseBatch(htmls);
//! ```

#![deny(clippy::all)]

mod config;
mod error;
mod selector;
mod soup;
mod tag;

use std::sync::Arc;

pub use config::SoupConfig;
use napi_derive::napi;
use rayon::prelude::*;
pub use selector::CompiledSelector;
pub use soup::Soup;
pub use tag::Tag;

/// Parse multiple HTML documents in parallel.
///
/// Uses Rayon for parallel processing. Provides significant speedup
/// when parsing many documents.
///
/// @param documents - Array of HTML strings to parse
/// @param config - Optional parsing configuration
/// @returns Array of Soup instances in the same order as input
///
/// @example
/// ```javascript
/// import { parseBatch } from 'scrape-rs';
///
/// const htmls = ['<div>A</div>', '<div>B</div>', '<div>C</div>'];
/// const soups = parseBatch(htmls);
/// const texts = soups.map(s => s.find('div').text);
/// // texts: ['A', 'B', 'C']
/// ```
#[napi(js_name = "parseBatch")]
pub fn parse_batch(documents: Vec<String>, config: Option<SoupConfig>) -> Vec<Soup> {
    let core_config = config.map(|c| c.to_core()).unwrap_or_default();

    documents
        .par_iter()
        .map(|html| {
            let soup = scrape_core::Soup::parse_with_config(html, core_config.clone());
            Soup { inner: Arc::new(soup) }
        })
        .collect()
}

/// Get the library version.
#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
