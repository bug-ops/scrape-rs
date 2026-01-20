//! Streaming HTML parser with typestate pattern.

#[cfg(feature = "streaming")]
use std::marker::PhantomData;

#[cfg(feature = "streaming")]
use crate::{
    Error, Result,
    streaming::{StreamingConfig, StreamingElement, handlers::HandlerRegistry},
};

/// State markers for streaming parser typestate pattern.
pub mod state {
    /// Parser is idle and accepting handler registrations.
    #[derive(Debug)]
    pub struct Idle;
    /// Parser is actively processing chunks.
    #[derive(Debug)]
    pub struct Processing;
    /// Parser has finished processing.
    #[derive(Debug)]
    pub struct Finished;
}

/// Sealed trait for streaming states.
mod private {
    pub trait Sealed {}
    impl Sealed for super::state::Idle {}
    impl Sealed for super::state::Processing {}
    impl Sealed for super::state::Finished {}
}

/// Marker trait for valid streaming parser states.
pub trait StreamingState: private::Sealed {}
impl StreamingState for state::Idle {}
impl StreamingState for state::Processing {}
impl StreamingState for state::Finished {}

/// Streaming HTML parser with typestate-enforced lifecycle.
///
/// This parser uses the typestate pattern to enforce correct usage at compile time:
/// - In `Idle` state: can register handlers
/// - In `Processing` state: can write chunks
/// - In `Finished` state: can access results
///
/// # State Transitions
///
/// ```text
/// StreamingSoup<Idle> --[start()]--> StreamingSoup<Processing> --[end()]--> StreamingSoup<Finished>
/// ```
///
/// # Examples
///
/// ```ignore
/// use scrape_core::StreamingSoup;
///
/// let mut streaming = StreamingSoup::new();
/// streaming.on_element("a[href]", |el| {
///     println!("Link: {}", el.get_attribute("href").unwrap_or_default());
///     Ok(())
/// })?;
///
/// let mut processor = streaming.start();
/// processor.write(b"<a href='test'>Link</a>")?;
/// let finished = processor.end()?;
/// println!("Processed {} bytes", finished.stats().bytes_processed);
/// ```
#[cfg(feature = "streaming")]
pub struct StreamingSoup<S: StreamingState = state::Idle> {
    inner: StreamingSoupInner,
    _state: PhantomData<S>,
}

#[cfg(feature = "streaming")]
struct StreamingSoupInner {
    config: StreamingConfig,
    handlers: HandlerRegistry,
    stats: StreamingStats,
}

/// Statistics collected during streaming parse.
#[cfg(feature = "streaming")]
#[derive(Debug, Clone, Default)]
pub struct StreamingStats {
    /// Total bytes processed.
    pub bytes_processed: usize,
    /// Number of elements encountered.
    pub elements_count: usize,
    /// Number of text nodes encountered.
    pub text_nodes_count: usize,
}

#[cfg(feature = "streaming")]
impl StreamingSoup<state::Idle> {
    /// Creates a new streaming parser with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(StreamingConfig::default())
    }

    /// Creates a new streaming parser with the given configuration.
    #[must_use]
    pub fn with_config(config: StreamingConfig) -> Self {
        Self {
            inner: StreamingSoupInner {
                config,
                handlers: HandlerRegistry::new(),
                stats: StreamingStats::default(),
            },
            _state: PhantomData,
        }
    }

    /// Registers a handler for elements matching the given selector.
    ///
    /// The handler will be called for each element that matches the selector
    /// during streaming parse.
    ///
    /// # Errors
    ///
    /// Returns an error if the selector is invalid.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// streaming.on_element("a[href]", |el| {
    ///     println!("Link: {}", el.get_attribute("href")?);
    ///     Ok(())
    /// })?;
    /// ```
    pub fn on_element<F>(&mut self, selector: &str, handler: F) -> Result<&mut Self>
    where
        F: FnMut(&mut StreamingElement) -> Result<()> + Send + 'static,
    {
        // Basic validation - ensure non-empty selector
        if selector.is_empty() {
            return Err(Error::streaming_selector_error("selector cannot be empty"));
        }

        // NOTE: Full selector validation deferred later when we integrate lol_html rewriter
        // lol_html will validate selectors when building the rewriter

        self.inner.handlers.register_element(selector.to_string(), handler);
        Ok(self)
    }

    /// Registers a handler for text nodes within elements matching the selector.
    ///
    /// # Errors
    ///
    /// Returns an error if the selector is invalid.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// streaming.on_text("p", |text| {
    ///     println!("Paragraph text: {}", text);
    ///     Ok(())
    /// })?;
    /// ```
    pub fn on_text<F>(&mut self, selector: &str, handler: F) -> Result<&mut Self>
    where
        F: FnMut(&str) -> Result<()> + Send + 'static,
    {
        // Basic validation
        if selector.is_empty() {
            return Err(Error::streaming_selector_error("selector cannot be empty"));
        }

        self.inner.handlers.register_text(selector.to_string(), handler);
        Ok(self)
    }

    /// Registers a handler for end tags matching the given selector.
    ///
    /// # Errors
    ///
    /// Returns an error if the selector is invalid.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// streaming.on_end_tag("div", |tag_name| {
    ///     println!("End tag: {}", tag_name);
    ///     Ok(())
    /// })?;
    /// ```
    pub fn on_end_tag<F>(&mut self, selector: &str, handler: F) -> Result<&mut Self>
    where
        F: FnMut(&str) -> Result<()> + Send + 'static,
    {
        // Basic validation
        if selector.is_empty() {
            return Err(Error::streaming_selector_error("selector cannot be empty"));
        }

        self.inner.handlers.register_end_tag(selector.to_string(), handler);
        Ok(self)
    }

    /// Starts the streaming parser, transitioning to Processing state.
    ///
    /// After calling this method, you can write chunks using `write()`.
    #[must_use]
    pub fn start(self) -> StreamingSoup<state::Processing> {
        StreamingSoup { inner: self.inner, _state: PhantomData }
    }
}

#[cfg(feature = "streaming")]
impl StreamingSoup<state::Processing> {
    /// Writes a chunk of HTML to the streaming parser.
    ///
    /// The chunk will be processed and any registered handlers will be called
    /// for matching elements/text.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails or a handler returns an error.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// processor.write(b"<div>content</div>")?;
    /// ```
    pub fn write(&mut self, chunk: &[u8]) -> Result<()> {
        // Update stats
        self.inner.stats.bytes_processed += chunk.len();

        // NOTE: Actual lol_html integration deferred to Week 2
        // This is a stub implementation for Week 1 foundation
        Ok(())
    }

    /// Writes multiple chunks to the streaming parser.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails or a handler returns an error.
    pub fn write_all<'a>(&mut self, chunks: impl Iterator<Item = &'a [u8]>) -> Result<()> {
        for chunk in chunks {
            self.write(chunk)?;
        }
        Ok(())
    }

    /// Finishes processing and transitions to Finished state.
    ///
    /// After calling this method, you can access statistics via `stats()`.
    ///
    /// # Errors
    ///
    /// Returns an error if finalizing the parse fails.
    pub fn end(self) -> Result<StreamingSoup<state::Finished>> {
        Ok(StreamingSoup { inner: self.inner, _state: PhantomData })
    }
}

#[cfg(feature = "streaming")]
impl StreamingSoup<state::Finished> {
    /// Returns statistics about the streaming parse.
    #[must_use]
    pub fn stats(&self) -> &StreamingStats {
        &self.inner.stats
    }
}

#[cfg(feature = "streaming")]
impl Default for StreamingSoup<state::Idle> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "streaming"))]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_soup_new() {
        let streaming = StreamingSoup::new();
        assert_eq!(streaming.inner.config.buffer_size, 8192);
    }

    #[test]
    fn test_streaming_soup_with_config() {
        let config = StreamingConfig::new().buffer_size(16384);
        let streaming = StreamingSoup::with_config(config);
        assert_eq!(streaming.inner.config.buffer_size, 16384);
    }

    #[test]
    fn test_state_transitions() {
        let mut streaming = StreamingSoup::new();

        // Register handler in Idle state
        let result = streaming.on_element("div", |_el| Ok(()));
        assert!(result.is_ok());

        // Transition to Processing
        let mut processor = streaming.start();

        // Write chunk in Processing state
        let result = processor.write(b"<div>test</div>");
        assert!(result.is_ok());

        // Transition to Finished
        let finished = processor.end();
        assert!(finished.is_ok());

        // Access stats in Finished state
        let finished = finished.unwrap();
        assert_eq!(finished.stats().bytes_processed, 15); // "<div>test</div>" is 15 bytes
    }

    #[test]
    fn test_register_multiple_handlers() {
        let mut streaming = StreamingSoup::new();

        streaming.on_element("div", |_el| Ok(())).unwrap();
        streaming.on_element("span", |_el| Ok(())).unwrap();
        streaming.on_text("p", |_text| Ok(())).unwrap();

        assert_eq!(streaming.inner.handlers.element_count(), 2);
        assert_eq!(streaming.inner.handlers.text_count(), 1);
    }

    #[test]
    fn test_invalid_selector() {
        let mut streaming = StreamingSoup::new();
        let result = streaming.on_element("", |_el| Ok(()));
        assert!(result.is_err());
    }

    #[test]
    fn test_write_all() {
        let streaming = StreamingSoup::new();
        let mut processor = streaming.start();

        let chunks = vec![b"<div>".as_slice(), b"test".as_slice(), b"</div>".as_slice()];
        let result = processor.write_all(chunks.into_iter());
        assert!(result.is_ok());

        let finished = processor.end().unwrap();
        assert_eq!(finished.stats().bytes_processed, 15); // "<div>" + "test" + "</div>" = 5 + 4 + 6 = 15 bytes
    }

    #[test]
    fn test_streaming_stats_default() {
        let stats = StreamingStats::default();
        assert_eq!(stats.bytes_processed, 0);
        assert_eq!(stats.elements_count, 0);
        assert_eq!(stats.text_nodes_count, 0);
    }
}
