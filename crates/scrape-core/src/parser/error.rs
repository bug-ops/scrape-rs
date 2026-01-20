//! Parser error types.

use thiserror::Error;

use crate::error::{SourceSpan, SpanContext};

/// Result type for parser operations.
pub type ParseResult<T> = Result<T, ParseError>;

/// Errors that can occur during HTML parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    /// Document exceeds maximum nesting depth.
    #[error("maximum nesting depth of {max_depth} exceeded{}", format_position(span.as_ref()))]
    MaxDepthExceeded {
        /// Configured maximum depth.
        max_depth: usize,
        /// Source location, if available.
        span: Option<SourceSpan>,
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

    /// Malformed HTML construct.
    #[error("malformed HTML: {message}{}", format_position(span.as_ref()))]
    MalformedHtml {
        /// Description of the malformation.
        message: String,
        /// Source location, if available.
        span: Option<SourceSpan>,
    },

    /// Internal parser error.
    #[error("internal parser error: {0}")]
    InternalError(String),
}

fn format_position(span: Option<&SourceSpan>) -> String {
    span.map_or_else(String::new, |s| {
        format!(" at line {}, column {}", s.start.line, s.start.column)
    })
}

impl ParseError {
    /// Returns the source span associated with this error, if any.
    #[must_use]
    pub fn span(&self) -> Option<&SourceSpan> {
        match self {
            Self::MaxDepthExceeded { span, .. } | Self::MalformedHtml { span, .. } => span.as_ref(),
            _ => None,
        }
    }

    /// Returns the line number of the error (1-indexed), if available.
    #[must_use]
    pub fn line(&self) -> Option<usize> {
        self.span().map(|s| s.start.line)
    }

    /// Returns the column number of the error (1-indexed), if available.
    #[must_use]
    pub fn column(&self) -> Option<usize> {
        self.span().map(|s| s.start.column)
    }

    /// Returns the span context for display, given the source text.
    #[must_use]
    pub fn span_context(&self, source: &str) -> Option<SpanContext> {
        self.span().map(|s| SpanContext::from_source(source, s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_depth_exceeded_display() {
        let err = ParseError::MaxDepthExceeded { max_depth: 512, span: None };
        assert_eq!(err.to_string(), "maximum nesting depth of 512 exceeded");
    }

    #[test]
    fn test_max_depth_exceeded_with_position() {
        use crate::error::SourcePosition;
        let span =
            SourceSpan::new(SourcePosition::new(10, 5, 100), SourcePosition::new(10, 10, 105));
        let err = ParseError::MaxDepthExceeded { max_depth: 512, span: Some(span) };
        assert_eq!(err.to_string(), "maximum nesting depth of 512 exceeded at line 10, column 5");
        assert_eq!(err.line(), Some(10));
        assert_eq!(err.column(), Some(5));
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

    #[test]
    fn test_malformed_html_with_span() {
        use crate::error::SourcePosition;
        let span = SourceSpan::new(SourcePosition::new(2, 7, 12), SourcePosition::new(2, 12, 17));
        let err = ParseError::MalformedHtml { message: "unclosed tag".into(), span: Some(span) };
        assert_eq!(err.to_string(), "malformed HTML: unclosed tag at line 2, column 7");
        assert_eq!(err.line(), Some(2));
        assert_eq!(err.column(), Some(7));
    }

    #[test]
    fn test_span_context() {
        use crate::error::SourcePosition;
        let source = "line1\nline2 error here\nline3";
        let span = SourceSpan::new(SourcePosition::new(2, 7, 12), SourcePosition::new(2, 12, 17));
        let err = ParseError::MalformedHtml { message: "test".into(), span: Some(span) };

        let ctx = err.span_context(source);
        assert!(ctx.is_some());
        let ctx = ctx.unwrap();
        assert_eq!(ctx.line_number, 2);
        assert_eq!(ctx.line_text, "line2 error here");
    }

    #[test]
    fn test_error_without_span() {
        let err = ParseError::EmptyInput;
        assert_eq!(err.line(), None);
        assert_eq!(err.column(), None);
        assert_eq!(err.span(), None);
    }
}
