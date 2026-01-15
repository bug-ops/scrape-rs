//! HTML parsing implementations.
//!
//! This module provides different HTML parsing strategies:
//!
//! - **html5ever**: Spec-compliant HTML5 parser for correct parsing of all HTML
//!
//! # Architecture
//!
//! The parser module is responsible for converting raw HTML bytes into a DOM
//! tree structure. It uses the `html5ever` crate for spec-compliant parsing.
//!
//! # Example
//!
//! ```rust,ignore
//! use scrape_core::{Html5everParser, Parser, ParseConfig};
//!
//! let parser = Html5everParser;
//! let config = ParseConfig::default();
//! let document = parser.parse_with_config("<html><body>Hello</body></html>", &config)?;
//! ```

mod error;
mod html5;
#[cfg(test)]
mod tests;

pub use error::{ParseError, ParseResult};
pub use html5::Html5everParser;

use crate::dom::Document;

/// Sealed trait module to prevent external implementations.
mod private {
    /// Marker trait for sealing [`Parser`](super::Parser).
    pub trait Sealed {}
}

/// A sealed trait for HTML parsers.
///
/// This trait is sealed and cannot be implemented outside of this crate.
/// Use [`Html5everParser`] for spec-compliant HTML5 parsing.
///
/// # Example
///
/// ```rust,ignore
/// use scrape_core::{Html5everParser, Parser, ParseConfig};
///
/// let parser = Html5everParser;
/// let document = parser.parse("<html><body>Hello</body></html>")?;
/// ```
pub trait Parser: private::Sealed {
    /// Parses HTML with default configuration.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError::EmptyInput`] if the input is empty or whitespace-only.
    fn parse(&self, html: &str) -> ParseResult<Document> {
        self.parse_with_config(html, &ParseConfig::default())
    }

    /// Parses HTML with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] if parsing fails:
    /// - [`ParseError::EmptyInput`] if the input is empty or whitespace-only
    /// - [`ParseError::MaxDepthExceeded`] if nesting exceeds `config.max_depth`
    fn parse_with_config(&self, html: &str, config: &ParseConfig) -> ParseResult<Document>;
}

/// Configuration for HTML parsing behavior.
///
/// # Example
///
/// ```rust
/// use scrape_core::ParseConfig;
///
/// let config = ParseConfig { max_depth: 256, preserve_whitespace: true, include_comments: false };
/// ```
#[derive(Debug, Clone)]
pub struct ParseConfig {
    /// Maximum nesting depth for the DOM tree.
    ///
    /// Parsing will return [`ParseError::MaxDepthExceeded`] if this limit is exceeded.
    /// Default: 512.
    pub max_depth: usize,

    /// Whether to preserve whitespace-only text nodes.
    ///
    /// When `false` (default), text nodes containing only whitespace are filtered out.
    pub preserve_whitespace: bool,

    /// Whether to include comment nodes in the parsed document.
    ///
    /// Default: `false`.
    pub include_comments: bool,
}

impl Default for ParseConfig {
    fn default() -> Self {
        Self { max_depth: 512, preserve_whitespace: false, include_comments: false }
    }
}
