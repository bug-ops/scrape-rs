//! Python bindings for scrape-rs.
//!
//! This module provides Python bindings for the scrape-core library using PyO3.

use pyo3::prelude::*;

mod config;
mod error;
mod selector;
mod soup;
mod tag;

use config::PySoupConfig;
use selector::PyCompiledSelector;
use soup::PySoup;
use tag::{PyTag, PyTagIterator};

/// Parse multiple HTML documents in parallel.
///
/// Uses Rayon for parallel processing. The GIL is released during parsing
/// for better concurrency with other Python threads.
///
/// Args:
///     documents: List of HTML strings to parse.
///     n_threads: Optional number of threads (defaults to CPU count).
///         Note: This parameter only takes effect on the first call per process.
///         Rayon's global thread pool is initialized once and cannot be reconfigured.
///         Subsequent calls with different n_threads values will be silently ignored.
///
/// Returns:
///     List of Soup instances in the same order as input.
///
/// Example:
///     >>> htmls = ["<div>A</div>", "<div>B</div>", "<div>C</div>"]
///     >>> soups = parse_batch(htmls)
///     >>> texts = [s.find("div").text for s in soups]
///     # Returns: A, B, C
#[pyfunction]
#[pyo3(signature = (documents, n_threads=None))]
fn parse_batch(py: Python<'_>, documents: Vec<String>, n_threads: Option<usize>) -> Vec<PySoup> {
    use std::sync::Arc;

    // Configure thread pool if specified
    if let Some(threads) = n_threads {
        let _ = rayon::ThreadPoolBuilder::new().num_threads(threads).build_global();
    }

    // Release GIL during parsing using Python::detach
    py.detach(|| {
        use rayon::prelude::*;

        documents
            .par_iter()
            .map(|html| {
                let soup = scrape_core::Soup::parse(html);
                PySoup { inner: Arc::new(soup) }
            })
            .collect()
    })
}

/// Compile a CSS selector string for efficient repeated use.
///
/// Args:
///     selector: CSS selector string to compile.
///
/// Returns:
///     A CompiledSelector instance.
///
/// Raises:
///     ValueError: If the selector syntax is invalid.
///
/// Example:
///     >>> from scrape_rs import compile_selector
///     >>> selector = compile_selector("div.item > span")
///     >>> print(selector.source)
///     div.item > span
#[pyfunction]
fn compile_selector(selector: &str) -> PyResult<PyCompiledSelector> {
    PyCompiledSelector::compile(selector)
}

/// Python module definition.
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySoupConfig>()?;
    m.add_class::<PySoup>()?;
    m.add_class::<PyTag>()?;
    m.add_class::<PyTagIterator>()?;
    m.add_class::<PyCompiledSelector>()?;
    m.add_function(wrap_pyfunction!(parse_batch, m)?)?;
    m.add_function(wrap_pyfunction!(compile_selector, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
