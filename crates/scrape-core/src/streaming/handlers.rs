//! Handler traits and registry for streaming parser.

use crate::{Result, streaming::element::StreamingElement};

/// Handler for element events during streaming.
///
/// Implement this trait to process elements as they are encountered
/// during streaming parsing.
///
/// Note: Send bound is required for cross-platform support. Python and Node.js
/// bindings (Week 4) will invoke handlers from thread pools, requiring thread-safe handlers.
pub trait ElementHandler: Send {
    /// Called when an element matching the selector is found.
    ///
    /// # Errors
    ///
    /// Returns an error if the handler fails to process the element.
    fn handle(&mut self, element: &mut StreamingElement) -> Result<()>;
}

/// Handler for text node events during streaming.
///
/// Implement this trait to process text nodes as they are encountered
/// during streaming parsing.
///
/// Note: Send bound is required for cross-platform support. Python and Node.js
/// bindings (Week 4) will invoke handlers from thread pools, requiring thread-safe handlers.
pub trait TextHandler: Send {
    /// Called when text content is found.
    ///
    /// # Errors
    ///
    /// Returns an error if the handler fails to process the text.
    fn handle(&mut self, text: &str) -> Result<()>;
}

/// Handler for end tag events during streaming.
///
/// Implement this trait to process end tags as they are encountered
/// during streaming parsing.
///
/// Note: Send bound is required for cross-platform support. Python and Node.js
/// bindings (Week 4) will invoke handlers from thread pools, requiring thread-safe handlers.
pub trait EndTagHandler: Send {
    /// Called when an end tag is encountered.
    ///
    /// # Errors
    ///
    /// Returns an error if the handler fails to process the end tag.
    fn handle(&mut self, tag_name: &str) -> Result<()>;
}

/// Wrapper for boxed element handler functions.
struct BoxedElementHandler<F>
where
    F: FnMut(&mut StreamingElement) -> Result<()> + Send,
{
    handler: F,
}

impl<F> ElementHandler for BoxedElementHandler<F>
where
    F: FnMut(&mut StreamingElement) -> Result<()> + Send,
{
    fn handle(&mut self, element: &mut StreamingElement) -> Result<()> {
        (self.handler)(element)
    }
}

/// Wrapper for boxed text handler functions.
struct BoxedTextHandler<F>
where
    F: FnMut(&str) -> Result<()> + Send,
{
    handler: F,
}

impl<F> TextHandler for BoxedTextHandler<F>
where
    F: FnMut(&str) -> Result<()> + Send,
{
    fn handle(&mut self, text: &str) -> Result<()> {
        (self.handler)(text)
    }
}

/// Wrapper for boxed end tag handler functions.
struct BoxedEndTagHandler<F>
where
    F: FnMut(&str) -> Result<()> + Send,
{
    handler: F,
}

impl<F> EndTagHandler for BoxedEndTagHandler<F>
where
    F: FnMut(&str) -> Result<()> + Send,
{
    fn handle(&mut self, tag_name: &str) -> Result<()> {
        (self.handler)(tag_name)
    }
}

/// Registry for streaming handlers.
///
/// Manages registered handlers and their associated selectors.
#[derive(Default)]
#[allow(clippy::struct_field_names, clippy::redundant_pub_crate)]
pub(crate) struct HandlerRegistry {
    element_handlers: Vec<(String, Box<dyn ElementHandler>)>,
    text_handlers: Vec<(String, Box<dyn TextHandler>)>,
    end_tag_handlers: Vec<(String, Box<dyn EndTagHandler>)>,
}

impl HandlerRegistry {
    /// Creates a new empty handler registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an element handler for the given selector.
    pub fn register_element<F>(&mut self, selector: String, handler: F)
    where
        F: FnMut(&mut StreamingElement) -> Result<()> + Send + 'static,
    {
        let boxed = Box::new(BoxedElementHandler { handler });
        self.element_handlers.push((selector, boxed));
    }

    /// Registers a text handler for the given selector.
    pub fn register_text<F>(&mut self, selector: String, handler: F)
    where
        F: FnMut(&str) -> Result<()> + Send + 'static,
    {
        let boxed = Box::new(BoxedTextHandler { handler });
        self.text_handlers.push((selector, boxed));
    }

    /// Registers an end tag handler for the given selector.
    pub fn register_end_tag<F>(&mut self, selector: String, handler: F)
    where
        F: FnMut(&str) -> Result<()> + Send + 'static,
    {
        let boxed = Box::new(BoxedEndTagHandler { handler });
        self.end_tag_handlers.push((selector, boxed));
    }

    /// Returns the number of registered element handlers.
    #[must_use]
    pub fn element_count(&self) -> usize {
        self.element_handlers.len()
    }

    /// Returns the number of registered text handlers.
    #[must_use]
    pub fn text_count(&self) -> usize {
        self.text_handlers.len()
    }

    /// Returns the number of registered end tag handlers.
    #[must_use]
    pub fn end_tag_count(&self) -> usize {
        self.end_tag_handlers.len()
    }

    /// Returns an iterator over element handler selectors.
    pub fn element_selectors(&self) -> impl Iterator<Item = &str> {
        self.element_handlers.iter().map(|(sel, _)| sel.as_str())
    }

    /// Returns an iterator over text handler selectors.
    pub fn text_selectors(&self) -> impl Iterator<Item = &str> {
        self.text_handlers.iter().map(|(sel, _)| sel.as_str())
    }

    /// Returns an iterator over end tag handler selectors.
    pub fn end_tag_selectors(&self) -> impl Iterator<Item = &str> {
        self.end_tag_handlers.iter().map(|(sel, _)| sel.as_str())
    }

    /// Returns a mutable reference to element handlers.
    pub fn element_handlers_mut(&mut self) -> &mut Vec<(String, Box<dyn ElementHandler>)> {
        &mut self.element_handlers
    }

    /// Returns a mutable reference to text handlers.
    pub fn text_handlers_mut(&mut self) -> &mut Vec<(String, Box<dyn TextHandler>)> {
        &mut self.text_handlers
    }

    /// Returns a mutable reference to end tag handlers.
    pub fn end_tag_handlers_mut(&mut self) -> &mut Vec<(String, Box<dyn EndTagHandler>)> {
        &mut self.end_tag_handlers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_registry_new() {
        let registry = HandlerRegistry::new();
        assert_eq!(registry.element_count(), 0);
        assert_eq!(registry.text_count(), 0);
        assert_eq!(registry.end_tag_count(), 0);
    }

    #[test]
    fn test_register_element_handler() {
        let mut registry = HandlerRegistry::new();
        registry.register_element("div".to_string(), |_el| Ok(()));
        assert_eq!(registry.element_count(), 1);

        let selectors: Vec<_> = registry.element_selectors().collect();
        assert_eq!(selectors, vec!["div"]);
    }

    #[test]
    fn test_register_text_handler() {
        let mut registry = HandlerRegistry::new();
        registry.register_text("p".to_string(), |_text| Ok(()));
        assert_eq!(registry.text_count(), 1);

        let selectors: Vec<_> = registry.text_selectors().collect();
        assert_eq!(selectors, vec!["p"]);
    }

    #[test]
    fn test_register_end_tag_handler() {
        let mut registry = HandlerRegistry::new();
        registry.register_end_tag("div".to_string(), |_tag| Ok(()));
        assert_eq!(registry.end_tag_count(), 1);

        let selectors: Vec<_> = registry.end_tag_selectors().collect();
        assert_eq!(selectors, vec!["div"]);
    }

    #[test]
    fn test_multiple_handlers() {
        let mut registry = HandlerRegistry::new();
        registry.register_element("div".to_string(), |_el| Ok(()));
        registry.register_element("span".to_string(), |_el| Ok(()));
        registry.register_text("p".to_string(), |_text| Ok(()));

        assert_eq!(registry.element_count(), 2);
        assert_eq!(registry.text_count(), 1);
        assert_eq!(registry.end_tag_count(), 0);
    }
}
