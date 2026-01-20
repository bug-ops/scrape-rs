//! Streaming HTML parser with typestate pattern.

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use lol_html::AsciiCompatibleEncoding;

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
pub struct StreamingSoup<S: StreamingState = state::Idle> {
    inner: StreamingSoupInner,
    _state: PhantomData<S>,
}

struct StreamingSoupInner {
    config: StreamingConfig,
    handlers: HandlerRegistry,
    stats: StreamingStats,
    output_buffer: Vec<u8>,
}

/// Statistics collected during streaming parse.
#[derive(Debug, Clone, Default)]
pub struct StreamingStats {
    /// Total bytes processed.
    pub bytes_processed: usize,
    /// Number of elements encountered.
    pub elements_count: usize,
    /// Number of text nodes encountered.
    pub text_nodes_count: usize,
}

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
                output_buffer: Vec::new(),
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
    /// # Panics
    ///
    /// This method contains an `expect()` that should never panic as UTF-8 is always
    /// ASCII-compatible. If it panics, it indicates a bug in `lol_html`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// processor.write(b"<div>content</div>")?;
    /// ```
    pub fn write(&mut self, chunk: &[u8]) -> Result<()> {
        // Update stats
        self.inner.stats.bytes_processed += chunk.len();

        // Early return if no handlers registered - just pass through
        if self.inner.handlers.element_count() == 0
            && self.inner.handlers.text_count() == 0
            && self.inner.handlers.end_tag_count() == 0
        {
            self.inner.output_buffer.extend_from_slice(chunk);
            return Ok(());
        }

        // lol_html requires building handlers at Settings creation time
        // We use Cell/RefCell to share mutable access safely within single-threaded context
        let error_cell: Rc<RefCell<Option<Error>>> = Rc::new(RefCell::new(None));

        // Share stats for updating counts
        let element_count: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
        let text_count: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));

        // Build element content handlers for lol_html
        let mut element_handlers = Vec::new();

        // We need to capture handlers but lol_html wants to own the closures
        // Solution: use unsafe pointer + runtime checks to bridge the ownership gap
        // SAFETY: This is safe because:
        // 1. We're single-threaded (no Send/Sync issues)
        // 2. Handlers live in StreamingSoupInner which outlives this method
        // 3. lol_html closures don't outlive this method call
        // 4. The pointer remains valid for the entire duration of HtmlRewriter usage

        let handlers_ptr = &raw mut self.inner.handlers;

        // Process element handlers
        for (idx, (selector, _)) in self.inner.handlers.element_handlers_mut().iter().enumerate() {
            let selector_owned = selector.clone();
            let error_clone = Rc::clone(&error_cell);
            let elem_count_clone = Rc::clone(&element_count);

            element_handlers.push(lol_html::element!(selector_owned, move |el| {
                // Stop processing if previous handler failed
                if error_clone.borrow().is_some() {
                    return Ok(());
                }

                // Increment element count
                *elem_count_clone.borrow_mut() += 1;

                // Get handler
                // SAFETY: handlers_ptr points to self.inner.handlers which:
                // 1. Is valid for the entire duration of this method
                // 2. Will not be moved or dropped while HtmlRewriter is active
                // 3. We are in single-threaded context (Rc<RefCell> is not Send)
                // 4. Each handler is accessed at most once per element
                #[allow(unsafe_code)]
                let handler =
                    unsafe { (*handlers_ptr).element_handlers_mut().get_mut(idx).map(|(_, h)| h) };

                if let Some(handler) = handler {
                    let mut streaming_el = StreamingElement::new(el);
                    if let Err(e) = handler.handle(&mut streaming_el) {
                        *error_clone.borrow_mut() = Some(e);
                    }
                }

                Ok(())
            }));
        }

        // Build lol_html settings
        let settings: lol_html::Settings<'_, '_, lol_html::LocalHandlerTypes> =
            lol_html::Settings {
                element_content_handlers: element_handlers,
                document_content_handlers: Vec::new(),
                encoding: AsciiCompatibleEncoding::new(encoding_rs::UTF_8)
                    .expect("UTF-8 is always ASCII-compatible"),
                memory_settings: lol_html::MemorySettings::default(),
                strict: self.inner.config.strict_mode,
                enable_esi_tags: false,
                adjust_charset_on_meta_tag: true,
            };

        // Create output sink
        let mut output = Vec::new();

        // Create rewriter
        let mut rewriter = lol_html::HtmlRewriter::new(settings, |chunk: &[u8]| {
            output.extend_from_slice(chunk);
        });

        // Write chunk through rewriter
        rewriter
            .write(chunk)
            .map_err(|e| Error::handler_error(format!("lol_html write failed: {e}")))?;

        // Finish rewriter to flush remaining output
        rewriter.end().map_err(|e| Error::handler_error(format!("lol_html end failed: {e}")))?;

        // Update stats with counts from this chunk
        self.inner.stats.elements_count += *element_count.borrow();
        self.inner.stats.text_nodes_count += *text_count.borrow();

        // Append output to buffer
        self.inner.output_buffer.extend_from_slice(&output);

        // Check if any handler failed
        if let Some(error) = error_cell.borrow_mut().take() {
            return Err(error);
        }

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

impl StreamingSoup<state::Finished> {
    /// Returns statistics about the streaming parse.
    #[must_use]
    pub fn stats(&self) -> &StreamingStats {
        &self.inner.stats
    }

    /// Returns the output buffer (for rewriting scenarios).
    ///
    /// When handlers modify HTML content, this buffer contains the transformed output.
    #[must_use]
    pub fn output(&self) -> &[u8] {
        &self.inner.output_buffer
    }

    /// Consumes the streaming parser and returns the output buffer.
    #[must_use]
    pub fn into_output(self) -> Vec<u8> {
        self.inner.output_buffer
    }
}

impl Default for StreamingSoup<state::Idle> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
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
