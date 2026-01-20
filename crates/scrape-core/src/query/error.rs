//! Error types for query operations.

use thiserror::Error;

use crate::error::{SourcePosition, SourceSpan};

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
    #[error("invalid selector{}: {message}", format_position(span.as_ref()))]
    InvalidSelector {
        /// Error message from selector parser.
        message: String,
        /// Source location, if available.
        span: Option<SourceSpan>,
    },
}

fn format_position(span: Option<&SourceSpan>) -> String {
    span.map_or_else(String::new, |s| {
        format!(" at line {}, column {}", s.start.line, s.start.column)
    })
}

impl QueryError {
    /// Creates a new invalid selector error.
    #[must_use]
    pub fn invalid_selector(message: impl Into<String>) -> Self {
        Self::InvalidSelector { message: message.into(), span: None }
    }

    /// Creates a new invalid selector error with position.
    #[must_use]
    pub fn invalid_selector_at(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self::InvalidSelector {
            message: message.into(),
            span: Some(SourceSpan::new(
                SourcePosition::new(line, column, 0),
                SourcePosition::new(line, column + 1, 0),
            )),
        }
    }

    /// Returns the source span if available.
    #[must_use]
    pub fn span(&self) -> Option<&SourceSpan> {
        match self {
            Self::InvalidSelector { span, .. } => span.as_ref(),
        }
    }

    /// Returns the line number (1-indexed) if available.
    #[must_use]
    pub fn line(&self) -> Option<usize> {
        self.span().map(|s| s.start.line)
    }

    /// Returns the column number (1-indexed) if available.
    #[must_use]
    pub fn column(&self) -> Option<usize> {
        self.span().map(|s| s.start.column)
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
    fn test_query_error_with_position() {
        let err = QueryError::invalid_selector_at("unexpected token", 1, 5);
        assert_eq!(err.to_string(), "invalid selector at line 1, column 5: unexpected token");
        assert_eq!(err.line(), Some(1));
        assert_eq!(err.column(), Some(5));
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
    fn test_query_error_span() {
        let err_with_span = QueryError::invalid_selector_at("test", 2, 7);
        assert!(err_with_span.span().is_some());

        let err_without_span = QueryError::invalid_selector("test");
        assert!(err_without_span.span().is_none());
        assert_eq!(err_without_span.line(), None);
        assert_eq!(err_without_span.column(), None);
    }

    #[test]
    fn test_query_result_type() {
        let ok: QueryResult<i32> = Ok(42);
        let err: QueryResult<i32> = Err(QueryError::invalid_selector("test"));

        assert!(ok.is_ok());
        assert!(err.is_err());
    }
}
