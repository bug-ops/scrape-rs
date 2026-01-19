//! Python wrapper for Soup document.

use std::sync::Arc;

use pyo3::prelude::*;
use scrape_core::Soup;

use crate::{config::PySoupConfig, error::IntoPyErr, selector::PyCompiledSelector, tag::PyTag};

/// A parsed HTML document.
///
/// Example:
///     >>> from scrape_rs import Soup
///     >>> soup = Soup("<div>Hello</div>")
///     >>> print(soup.find("div").text)
///     Hello
#[pyclass(name = "Soup")]
pub struct PySoup {
    pub(crate) inner: Arc<Soup>,
}

#[pymethods]
impl PySoup {
    /// Parse an HTML string into a Soup document.
    ///
    /// Args:
    ///     html: HTML string to parse.
    ///     config: Optional parsing configuration.
    ///
    /// Returns:
    ///     A new Soup instance.
    #[new]
    #[pyo3(signature = (html, config=None))]
    fn new(html: &str, config: Option<&PySoupConfig>) -> Self {
        let core_config = config.map(PySoupConfig::to_core).unwrap_or_default();

        let soup = Soup::parse_with_config(html, core_config);
        Self { inner: Arc::new(soup) }
    }

    /// Parse HTML from a file.
    ///
    /// Args:
    ///     path: Path to the HTML file.
    ///     config: Optional parsing configuration.
    ///
    /// Returns:
    ///     A new Soup instance.
    ///
    /// Raises:
    ///     ValueError: If the file cannot be read.
    #[staticmethod]
    #[pyo3(signature = (path, config=None))]
    fn from_file(path: &str, config: Option<&PySoupConfig>) -> PyResult<Self> {
        let html = std::fs::read_to_string(path).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Failed to read file: {e}"))
        })?;
        Ok(Self::new(&html, config))
    }

    /// Find the first element matching a CSS selector.
    ///
    /// Args:
    ///     selector: CSS selector string.
    ///
    /// Returns:
    ///     The first matching Tag, or None if not found.
    ///
    /// Raises:
    ///     ValueError: If the selector syntax is invalid.
    fn find(&self, selector: &str) -> PyResult<Option<PyTag>> {
        self.inner
            .find(selector)
            .map_err(IntoPyErr::into_py_err)
            .map(|opt| opt.map(|tag| PyTag::new(Arc::clone(&self.inner), tag.node_id())))
    }

    /// Find all elements matching a CSS selector.
    ///
    /// Args:
    ///     selector: CSS selector string.
    ///
    /// Returns:
    ///     List of matching Tag instances.
    ///
    /// Raises:
    ///     ValueError: If the selector syntax is invalid.
    fn find_all(&self, selector: &str) -> PyResult<Vec<PyTag>> {
        self.inner.find_all(selector).map_err(IntoPyErr::into_py_err).map(|tags| {
            tags.into_iter().map(|tag| PyTag::new(Arc::clone(&self.inner), tag.node_id())).collect()
        })
    }

    /// Find all elements matching a CSS selector (alias for find_all).
    ///
    /// Args:
    ///     selector: CSS selector string.
    ///
    /// Returns:
    ///     List of matching Tag instances.
    fn select(&self, selector: &str) -> PyResult<Vec<PyTag>> {
        self.find_all(selector)
    }

    /// Find the first element using a pre-compiled selector.
    ///
    /// Args:
    ///     selector: A CompiledSelector instance.
    ///
    /// Returns:
    ///     The first matching Tag, or None if not found.
    fn find_compiled(&self, selector: &PyCompiledSelector) -> Option<PyTag> {
        self.inner
            .find_compiled(&selector.inner)
            .map(|tag| PyTag::new(Arc::clone(&self.inner), tag.node_id()))
    }

    /// Find all elements using a pre-compiled selector.
    ///
    /// Args:
    ///     selector: A CompiledSelector instance.
    ///
    /// Returns:
    ///     List of matching Tag instances.
    fn select_compiled(&self, selector: &PyCompiledSelector) -> Vec<PyTag> {
        self.inner
            .select_compiled(&selector.inner)
            .into_iter()
            .map(|tag| PyTag::new(Arc::clone(&self.inner), tag.node_id()))
            .collect()
    }

    /// Parse an HTML fragment without wrapping in html/body tags.
    ///
    /// Args:
    ///     html: HTML fragment to parse.
    ///     context: Optional context element name (default: "body").
    ///     config: Optional parsing configuration.
    ///
    /// Returns:
    ///     A new Soup instance.
    #[staticmethod]
    #[pyo3(signature = (html, context=None, config=None))]
    fn parse_fragment(html: &str, context: Option<&str>, config: Option<&PySoupConfig>) -> Self {
        let core_config = config.map(PySoupConfig::to_core).unwrap_or_default();

        let soup = if let Some(ctx) = context {
            Soup::parse_fragment_with_config(html, ctx, core_config)
        } else {
            Soup::parse_fragment_with_config(html, "body", core_config)
        };

        Self { inner: Arc::new(soup) }
    }

    /// Extract text content from all elements matching a CSS selector.
    ///
    /// Args:
    ///     selector: CSS selector string.
    ///
    /// Returns:
    ///     List of text strings, one per matching element.
    ///
    /// Raises:
    ///     ValueError: If the selector syntax is invalid.
    fn select_text(&self, selector: &str) -> PyResult<Vec<String>> {
        self.inner.select_text(selector).map_err(IntoPyErr::into_py_err)
    }

    /// Extract attribute values from all elements matching a CSS selector.
    ///
    /// Args:
    ///     selector: CSS selector string.
    ///     attr: Attribute name to extract.
    ///
    /// Returns:
    ///     List of attribute values (None for missing attributes).
    ///
    /// Raises:
    ///     ValueError: If the selector syntax is invalid.
    fn select_attr(&self, selector: &str, attr: &str) -> PyResult<Vec<Option<String>>> {
        self.inner.select_attr(selector, attr).map_err(IntoPyErr::into_py_err)
    }

    /// Get the root element of the document.
    ///
    /// Returns:
    ///     The root Tag (usually <html>), or None for empty documents.
    #[getter]
    fn root(&self) -> Option<PyTag> {
        self.inner.root().map(|tag| PyTag::new(Arc::clone(&self.inner), tag.node_id()))
    }

    /// Get the document title.
    ///
    /// Returns:
    ///     The title text, or None if no <title> element exists.
    #[getter]
    fn title(&self) -> Option<String> {
        self.inner.title()
    }

    /// Get the text content of the entire document.
    ///
    /// Returns:
    ///     All text content with HTML tags stripped.
    #[getter]
    fn text(&self) -> String {
        self.inner.text()
    }

    /// Get the HTML representation of the document.
    ///
    /// Returns:
    ///     The document as an HTML string.
    fn to_html(&self) -> String {
        self.inner.to_html()
    }

    fn __repr__(&self) -> String {
        let node_count = self.inner.document().len();
        format!("Soup(nodes={node_count})")
    }

    fn __str__(&self) -> String {
        self.to_html()
    }

    fn __len__(&self) -> usize {
        self.inner.document().len()
    }
}
