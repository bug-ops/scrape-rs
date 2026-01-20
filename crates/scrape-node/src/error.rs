//! Error conversion utilities for Node.js bindings.
//!
//! Maps scrape_core errors to napi errors.

use napi::{Error, Status};
use scrape_core::QueryError;

/// Convert query errors to napi errors.
pub trait IntoNapiError {
    /// Convert self into a napi error.
    fn into_napi_error(self) -> Error;
}

impl IntoNapiError for QueryError {
    fn into_napi_error(self) -> Error {
        match self {
            QueryError::InvalidSelector { message, .. } => {
                Error::new(Status::InvalidArg, format!("Invalid CSS selector: {message}"))
            }
        }
    }
}

/// Convert I/O errors to napi errors.
impl IntoNapiError for std::io::Error {
    fn into_napi_error(self) -> Error {
        Error::new(Status::GenericFailure, format!("I/O error: {self}"))
    }
}
