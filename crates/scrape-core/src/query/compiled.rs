//! Pre-compiled CSS selectors for efficient repeated matching.

use selectors::SelectorList;

use super::{QueryResult, ScrapeSelector, selector::parse_selector};

/// A pre-compiled CSS selector for efficient repeated matching.
///
/// Compiled selectors avoid the overhead of parsing the selector string on each query.
/// They can be reused across multiple documents and queries.
///
/// # Examples
///
/// ```rust
/// use scrape_core::{Soup, query::CompiledSelector};
///
/// let selector = CompiledSelector::compile("div.item > span").unwrap();
/// let soup = Soup::parse("<div class='item'><span>A</span></div>");
///
/// let results = soup.select_compiled(&selector);
/// assert_eq!(results.len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct CompiledSelector {
    selector_list: SelectorList<ScrapeSelector>,
    source: String,
}

impl CompiledSelector {
    /// Compiles a CSS selector string.
    ///
    /// # Errors
    ///
    /// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::query::CompiledSelector;
    ///
    /// let selector = CompiledSelector::compile("div.container > span").unwrap();
    /// assert_eq!(selector.source(), "div.container > span");
    /// ```
    pub fn compile(selector: &str) -> QueryResult<Self> {
        let selector_list = parse_selector(selector)?;
        Ok(Self { selector_list, source: selector.to_string() })
    }

    /// Returns the underlying selector list for matching.
    #[must_use]
    pub fn selector_list(&self) -> &SelectorList<ScrapeSelector> {
        &self.selector_list
    }

    /// Returns the original selector string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::query::CompiledSelector;
    ///
    /// let selector = CompiledSelector::compile("div").unwrap();
    /// assert_eq!(selector.source(), "div");
    /// ```
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }
}

/// Compiles a CSS selector string (convenience function).
///
/// This is equivalent to [`CompiledSelector::compile`].
///
/// # Errors
///
/// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
///
/// # Examples
///
/// ```rust
/// use scrape_core::query::compile_selector;
///
/// let selector = compile_selector("div > span").unwrap();
/// ```
pub fn compile_selector(selector: &str) -> QueryResult<CompiledSelector> {
    CompiledSelector::compile(selector)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_valid_selector() {
        let selector = CompiledSelector::compile("div.class > span").unwrap();
        assert_eq!(selector.source(), "div.class > span");
    }

    #[test]
    fn test_compile_invalid_selector_returns_error() {
        let result = CompiledSelector::compile("[");
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_selector_function() {
        let selector = compile_selector("div").unwrap();
        assert_eq!(selector.source(), "div");
    }

    #[test]
    fn test_selector_list_accessor() {
        let selector = CompiledSelector::compile("span").unwrap();
        assert_eq!(selector.selector_list().slice().len(), 1);
    }

    #[test]
    fn test_clone() {
        let selector = CompiledSelector::compile("div").unwrap();
        let cloned = selector.clone();
        assert_eq!(selector.source(), cloned.source());
    }
}
