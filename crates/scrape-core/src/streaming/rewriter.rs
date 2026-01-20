//! HTML rewriter for streaming modification.

use std::io::Write;

use crate::{
    Error, Result,
    streaming::{RewriterConfig, StreamingElement},
};

/// HTML rewriter for modifying HTML during streaming.
///
/// This type allows you to register handlers that modify HTML elements,
/// text, and other content as it is being processed. The modified HTML
/// is output to a buffer or writer.
///
/// # Examples
///
/// ```ignore
/// use scrape_core::HtmlRewriter;
///
/// let mut rewriter = HtmlRewriter::new();
///
/// rewriter.on_element("img", |el| {
///     el.set_attribute("loading", "lazy")?;
///     Ok(())
/// })?;
///
/// let output = rewriter.process("<img src='test.jpg'>")?;
/// assert!(output.contains("loading=\"lazy\""));
/// ```
/// Type alias for element handler functions.
type ElementHandlerFn = Box<dyn FnMut(&mut StreamingElement) -> Result<()> + Send>;

/// In-flight HTML rewriter for modifying elements during streaming.
///
/// Processes HTML chunks, allowing modification of attributes and content
/// as elements are encountered. Useful for transformations like adding `loading="lazy"`
/// to images or rewriting URLs.
///
/// # Example
///
/// ```ignore
/// let mut rewriter = HtmlRewriter::new();
/// rewriter.on_element("img", |el| {
///     el.set_attribute("loading", "lazy");
///     Ok(())
/// })?;
/// ```
pub struct HtmlRewriter {
    _config: RewriterConfig,
    _element_handlers: Vec<(String, ElementHandlerFn)>,
}

impl HtmlRewriter {
    /// Creates a new HTML rewriter with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(RewriterConfig::default())
    }

    /// Creates a new HTML rewriter with the given configuration.
    #[must_use]
    pub fn with_config(config: RewriterConfig) -> Self {
        Self { _config: config, _element_handlers: Vec::new() }
    }

    /// Registers a handler for elements matching the given selector.
    ///
    /// # Errors
    ///
    /// Returns an error if the selector is invalid.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// rewriter.on_element("a[href]", |el| {
    ///     el.set_attribute("rel", "noopener")?;
    ///     Ok(())
    /// })?;
    /// ```
    pub fn on_element<F>(&mut self, selector: &str, _handler: F) -> Result<&mut Self>
    where
        F: FnMut(&mut StreamingElement) -> Result<()> + 'static,
    {
        // Basic validation
        if selector.is_empty() {
            return Err(Error::streaming_selector_error("selector cannot be empty"));
        }

        // NOTE: Full implementation and selector validation deferred to Week 3
        Ok(self)
    }

    /// Processes HTML string and returns modified output.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing or rewriting fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let output = rewriter.process("<div>content</div>")?;
    /// ```
    pub fn process(&mut self, html: &str) -> Result<String> {
        self.process_bytes(html.as_bytes())
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }

    /// Processes HTML bytes and returns modified output.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing or rewriting fails.
    pub fn process_bytes(&mut self, html: &[u8]) -> Result<Vec<u8>> {
        // NOTE: Actual implementation deferred to Week 3
        // For now, just return input unchanged
        Ok(html.to_vec())
    }

    /// Processes HTML and writes output to the given writer.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing, rewriting, or writing fails.
    pub fn process_to<W: Write>(&mut self, html: &[u8], mut output: W) -> Result<()> {
        let result = self.process_bytes(html)?;
        output.write_all(&result)?;
        Ok(())
    }
}

impl Default for HtmlRewriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rewriter_new() {
        let _rewriter = HtmlRewriter::new();
    }

    #[test]
    fn test_rewriter_with_config() {
        let config = RewriterConfig::new().strict_mode(true);
        let _rewriter = HtmlRewriter::with_config(config);
    }

    #[test]
    fn test_register_element_handler() {
        let mut rewriter = HtmlRewriter::new();
        let result = rewriter.on_element("div", |_el| Ok(()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_selector() {
        let mut rewriter = HtmlRewriter::new();
        let result = rewriter.on_element("", |_el| Ok(()));
        assert!(result.is_err());
    }

    #[test]
    fn test_process_basic() {
        let mut rewriter = HtmlRewriter::new();
        let result = rewriter.process("<div>test</div>");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "<div>test</div>");
    }

    #[test]
    fn test_process_bytes() {
        let mut rewriter = HtmlRewriter::new();
        let input = b"<div>test</div>";
        let result = rewriter.process_bytes(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), input);
    }

    #[test]
    fn test_process_to() {
        let mut rewriter = HtmlRewriter::new();
        let input = b"<div>test</div>";
        let mut output = Vec::new();
        let result = rewriter.process_to(input, &mut output);
        assert!(result.is_ok());
        assert_eq!(output, input);
    }
}
