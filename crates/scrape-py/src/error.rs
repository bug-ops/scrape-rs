//! Error conversion utilities for Python bindings.
//!
//! Maps scrape_core errors to Python exceptions.

use pyo3::{PyErr, exceptions::PyValueError};
use scrape_core::QueryError;

/// Convert query errors to Python exceptions.
pub trait IntoPyErr {
    /// Convert self into a Python exception.
    fn into_py_err(self) -> PyErr;
}

impl IntoPyErr for QueryError {
    fn into_py_err(self) -> PyErr {
        match self {
            QueryError::InvalidSelector { message, .. } => {
                PyValueError::new_err(format!("Invalid CSS selector: {message}"))
            }
        }
    }
}
