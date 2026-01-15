//! Parser error types.

use thiserror::Error;

/// Result type for parser operations.
pub type ParseResult<T> = Result<T, ParseError>;

/// Errors that can occur during HTML parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    /// Document exceeds maximum nesting depth.
    #[error("maximum nesting depth of {max_depth} exceeded")]
    MaxDepthExceeded {
        /// Configured maximum depth.
        max_depth: usize,
    },

    /// Input is empty or contains only whitespace.
    #[error("empty or whitespace-only input")]
    EmptyInput,

    /// Encoding error in input.
    #[error("encoding error: {message}")]
    EncodingError {
        /// Description of the encoding problem.
        message: String,
    },

    /// Internal parser error.
    #[error("internal parser error: {0}")]
    InternalError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_depth_exceeded_display() {
        let err = ParseError::MaxDepthExceeded { max_depth: 512 };
        assert_eq!(err.to_string(), "maximum nesting depth of 512 exceeded");
    }

    #[test]
    fn test_empty_input_display() {
        let err = ParseError::EmptyInput;
        assert_eq!(err.to_string(), "empty or whitespace-only input");
    }

    #[test]
    fn test_encoding_error_display() {
        let err = ParseError::EncodingError { message: "invalid UTF-8 sequence".into() };
        assert_eq!(err.to_string(), "encoding error: invalid UTF-8 sequence");
    }

    #[test]
    fn test_internal_error_display() {
        let err = ParseError::InternalError("unexpected state".into());
        assert_eq!(err.to_string(), "internal parser error: unexpected state");
    }
}
