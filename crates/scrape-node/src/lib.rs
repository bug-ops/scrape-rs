//! Node.js bindings for scrape-rs.
//!
//! This module provides Node.js bindings for the scrape-core library using napi-rs.

#![deny(clippy::all)]

use napi_derive::napi;

/// Configuration options for HTML parsing.
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct SoupConfig {
    /// Maximum nesting depth for DOM tree.
    pub max_depth: Option<u32>,
    /// Enable strict parsing mode.
    pub strict_mode: Option<bool>,
}

impl From<SoupConfig> for scrape_core::SoupConfig {
    fn from(config: SoupConfig) -> Self {
        scrape_core::SoupConfig::builder()
            .max_depth(config.max_depth.unwrap_or(256) as usize)
            .strict_mode(config.strict_mode.unwrap_or(false))
            .build()
    }
}

/// A parsed HTML document.
#[napi]
pub struct Soup {
    inner: scrape_core::Soup,
}

#[napi]
impl Soup {
    /// Parses an HTML string into a Soup document.
    #[napi(constructor)]
    pub fn new(html: String, config: Option<SoupConfig>) -> Self {
        let config = config.map_or_else(scrape_core::SoupConfig::default, Into::into);
        Self { inner: scrape_core::Soup::parse_with_config(&html, config) }
    }

    /// Returns the document title if present.
    #[napi(getter)]
    pub fn title(&self) -> Option<String> {
        self.inner.title()
    }

    /// Returns the text content of the document.
    #[napi(getter)]
    pub fn text(&self) -> String {
        self.inner.text()
    }

    /// Returns the HTML representation of the document.
    #[napi(js_name = "toHtml")]
    pub fn to_html(&self) -> String {
        self.inner.to_html()
    }
}

/// Parse multiple HTML documents in parallel.
#[napi]
pub fn parse_batch(documents: Vec<String>, _config: Option<SoupConfig>) -> Vec<Soup> {
    // TODO: implement parallel batch parsing with rayon
    documents.into_iter().map(|html| Soup::new(html, None)).collect()
}
