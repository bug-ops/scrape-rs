//! Python bindings for scrape-rs.
//!
//! This module provides Python bindings for the scrape-core library using PyO3.

use pyo3::prelude::*;

/// Python wrapper for [`scrape_core::SoupConfig`].
#[pyclass(name = "SoupConfig")]
#[derive(Debug, Clone)]
pub struct PySoupConfig {
    inner: scrape_core::SoupConfig,
}

#[pymethods]
impl PySoupConfig {
    /// Creates a new configuration with default values.
    #[new]
    #[pyo3(signature = (max_depth=256, strict_mode=false))]
    fn new(max_depth: usize, strict_mode: bool) -> Self {
        Self {
            inner: scrape_core::SoupConfig::builder()
                .max_depth(max_depth)
                .strict_mode(strict_mode)
                .build(),
        }
    }

    /// Maximum nesting depth for DOM tree.
    #[getter]
    fn max_depth(&self) -> usize {
        self.inner.max_depth
    }

    /// Whether strict parsing mode is enabled.
    #[getter]
    fn strict_mode(&self) -> bool {
        self.inner.strict_mode
    }
}

/// Python wrapper for [`scrape_core::Soup`].
#[pyclass(name = "Soup")]
pub struct PySoup {
    inner: scrape_core::Soup,
}

#[pymethods]
impl PySoup {
    /// Parses an HTML string into a Soup document.
    #[new]
    #[pyo3(signature = (html, config=None))]
    fn new(html: &str, config: Option<PySoupConfig>) -> Self {
        let config = config.map_or_else(scrape_core::SoupConfig::default, |c| c.inner);
        Self { inner: scrape_core::Soup::parse_with_config(html, config) }
    }

    /// Returns the document title if present.
    #[getter]
    fn title(&self) -> Option<String> {
        self.inner.title()
    }

    /// Returns the text content of the document.
    #[getter]
    fn text(&self) -> String {
        self.inner.text()
    }

    /// Returns the HTML representation of the document.
    fn to_html(&self) -> String {
        self.inner.to_html()
    }
}

/// Parse multiple HTML documents in parallel.
///
/// This function uses Rayon for parallel processing on native platforms.
#[pyfunction]
#[pyo3(signature = (documents, n_threads=None))]
#[allow(unused_variables)]
fn parse_batch(documents: Vec<String>, n_threads: Option<usize>) -> Vec<PySoup> {
    // TODO: implement parallel batch parsing with rayon
    documents
        .into_iter()
        .map(|html| PySoup { inner: scrape_core::Soup::parse(&html) })
        .collect()
}

/// Python module definition.
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySoupConfig>()?;
    m.add_class::<PySoup>()?;
    m.add_function(wrap_pyfunction!(parse_batch, m)?)?;
    Ok(())
}
