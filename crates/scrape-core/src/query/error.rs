//! Error types for query operations.

use thiserror::Error;

/// Result type alias for query operations.
pub type QueryResult<T> = std::result::Result<T, QueryError>;

/// Error type for query operations.
///
/// This error type distinguishes between invalid selectors and other query failures,
/// enabling `Result<Option<Tag>, QueryError>` to differentiate "not found" from
/// "invalid query".
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum QueryError {
    /// Invalid CSS selector syntax.
    #[error("invalid selector: {0}")]
    InvalidSelector(String),
}

impl QueryError {
    /// Creates a new invalid selector error.
    #[must_use]
    pub fn invalid_selector(message: impl Into<String>) -> Self {
        Self::InvalidSelector(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_error_display() {
        let err = QueryError::invalid_selector("unexpected token at position 5");
        assert_eq!(err.to_string(), "invalid selector: unexpected token at position 5");
    }

    #[test]
    fn test_query_error_equality() {
        let err1 = QueryError::invalid_selector("foo");
        let err2 = QueryError::invalid_selector("foo");
        let err3 = QueryError::invalid_selector("bar");
        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_query_result_type() {
        let ok: QueryResult<i32> = Ok(42);
        let err: QueryResult<i32> = Err(QueryError::invalid_selector("test"));

        assert!(ok.is_ok());
        assert!(err.is_err());
    }
}
