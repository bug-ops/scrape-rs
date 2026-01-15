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
    #[allow(dead_code)]
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

    /// Finds the first element matching the selector.
    #[napi]
    pub fn find(&self, _selector: String) -> Option<Tag> {
        // TODO: implement find
        None
    }

    /// Finds all elements matching the selector.
    #[napi]
    pub fn find_all(&self, _selector: String) -> Vec<Tag> {
        // TODO: implement find_all
        Vec::new()
    }

    /// Selects elements using a CSS selector.
    #[napi]
    pub fn select(&self, selector: String) -> Vec<Tag> {
        self.find_all(selector)
    }
}

/// An HTML element in the DOM tree.
#[napi]
pub struct Tag {
    inner: scrape_core::Tag,
}

#[napi]
impl Tag {
    /// Returns the tag name.
    #[napi(getter)]
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    /// Returns the text content.
    #[napi(getter)]
    pub fn text(&self) -> String {
        // TODO: implement when Tag::text is implemented
        String::new()
    }

    /// Returns the inner HTML.
    #[napi(getter, js_name = "innerHTML")]
    pub fn inner_html(&self) -> String {
        // TODO: implement when Tag::inner_html is implemented
        String::new()
    }

    /// Returns the value of an attribute.
    #[napi]
    pub fn get(&self, _attr: String) -> Option<String> {
        // TODO: implement when Tag::get is implemented
        None
    }

    /// Alias for get(), for users familiar with DOM API.
    #[napi]
    pub fn attr(&self, attr: String) -> Option<String> {
        self.get(attr)
    }
}

/// Parse multiple HTML documents in parallel.
#[napi]
pub fn parse_batch(documents: Vec<String>, _config: Option<SoupConfig>) -> Vec<Soup> {
    // TODO: implement parallel batch parsing
    documents.into_iter().map(|html| Soup::new(html, None)).collect()
}
