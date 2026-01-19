//! Python wrapper for CompiledSelector.

use pyo3::prelude::*;
use scrape_core::query::CompiledSelector;

use crate::error::IntoPyErr;

/// A pre-compiled CSS selector for efficient repeated matching.
///
/// Compiled selectors avoid the overhead of parsing the selector string on each query.
///
/// Example:
///     >>> from scrape_rs import Soup, compile_selector
///     >>> selector = compile_selector("div.item")
///     >>> soup = Soup("<div class='item'>A</div><div class='item'>B</div>")
///     >>> items = soup.select_compiled(selector)
///     >>> len(items)
///     2
#[pyclass(name = "CompiledSelector")]
pub struct PyCompiledSelector {
    pub(crate) inner: CompiledSelector,
}

#[pymethods]
impl PyCompiledSelector {
    /// Get the original selector string.
    ///
    /// Returns:
    ///     The selector string that was compiled.
    #[getter]
    fn source(&self) -> &str {
        self.inner.source()
    }

    fn __repr__(&self) -> String {
        format!("CompiledSelector('{}')", self.inner.source())
    }
}

impl PyCompiledSelector {
    /// Compile a CSS selector string.
    ///
    /// # Errors
    ///
    /// Returns a Python ValueError if the selector syntax is invalid.
    pub fn compile(selector: &str) -> PyResult<Self> {
        CompiledSelector::compile(selector)
            .map(|inner| Self { inner })
            .map_err(IntoPyErr::into_py_err)
    }
}
