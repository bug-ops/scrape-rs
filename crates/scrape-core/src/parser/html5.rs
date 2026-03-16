//! html5ever-based HTML parser implementation.

use super::{ParseConfig, ParseError, ParseResult, Parser, private::Sealed};
use crate::dom::Document;

/// HTML5 spec-compliant parser using html5ever.
///
/// This parser uses the [html5ever](https://github.com/servo/html5ever) crate
/// for spec-compliant HTML5 parsing. It handles malformed HTML gracefully
/// using the HTML5 error recovery algorithm.
///
/// # Example
///
/// ```rust
/// use scrape_core::{Html5everParser, Parser};
///
/// let parser = Html5everParser;
/// let document = parser.parse("<html><body><h1>Hello</h1></body></html>").unwrap();
/// assert!(document.root().is_some());
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct Html5everParser;

impl Sealed for Html5everParser {}

impl Parser for Html5everParser {
    fn parse_with_config(&self, html: &str, config: &ParseConfig) -> ParseResult<Document> {
        self.parse_with_config_and_capacity(html, config, 256)
    }
}

impl Html5everParser {
    /// Parses HTML with the given configuration and pre-allocated capacity.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] if parsing fails.
    pub fn parse_with_config_and_capacity(
        &self,
        html: &str,
        config: &ParseConfig,
        capacity: usize,
    ) -> ParseResult<Document> {
        if html.trim().is_empty() {
            return Err(ParseError::EmptyInput);
        }

        super::sink::parse_html_document(html, config, capacity)
    }
}
