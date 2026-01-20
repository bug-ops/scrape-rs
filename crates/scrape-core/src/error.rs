//! Error types for scrape-core.

use thiserror::Error;

/// Result type alias using [`enum@Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during HTML parsing and querying.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to parse HTML document.
    #[error("failed to parse HTML: {message}")]
    ParseError {
        /// Description of what went wrong.
        message: String,
    },

    /// Invalid CSS selector syntax.
    #[error("invalid CSS selector: {selector}")]
    InvalidSelector {
        /// The selector string that failed to parse.
        selector: String,
    },

    /// Element not found.
    #[error("element not found: {query}")]
    NotFound {
        /// The query that returned no results.
        query: String,
    },

    /// Attribute not found on element.
    #[error("attribute '{name}' not found on element")]
    AttributeNotFound {
        /// The attribute name that was not found.
        name: String,
    },

    /// I/O error when reading from file or network.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Streaming parser in invalid state for this operation.
    #[cfg(feature = "streaming")]
    #[error("invalid streaming state: {message}")]
    InvalidStreamingState {
        /// Description of the invalid state.
        message: String,
    },

    /// Handler callback failed during streaming.
    #[cfg(feature = "streaming")]
    #[error("handler error: {message}")]
    HandlerError {
        /// Description of the handler error.
        message: String,
    },

    /// Streaming selector compilation failed.
    #[cfg(feature = "streaming")]
    #[error("streaming selector error: {message}")]
    StreamingSelectorError {
        /// Description of the selector error.
        message: String,
    },
}

impl Error {
    /// Creates a new parse error with the given message.
    #[must_use]
    pub fn parse(message: impl Into<String>) -> Self {
        Self::ParseError { message: message.into() }
    }

    /// Creates a new invalid selector error.
    #[must_use]
    pub fn invalid_selector(selector: impl Into<String>) -> Self {
        Self::InvalidSelector { selector: selector.into() }
    }

    /// Creates a new not found error.
    #[must_use]
    pub fn not_found(query: impl Into<String>) -> Self {
        Self::NotFound { query: query.into() }
    }

    /// Creates a new attribute not found error.
    #[must_use]
    pub fn attribute_not_found(name: impl Into<String>) -> Self {
        Self::AttributeNotFound { name: name.into() }
    }

    /// Creates a new invalid streaming state error.
    #[cfg(feature = "streaming")]
    #[must_use]
    pub fn invalid_streaming_state(message: impl Into<String>) -> Self {
        Self::InvalidStreamingState { message: message.into() }
    }

    /// Creates a new handler error.
    #[cfg(feature = "streaming")]
    #[must_use]
    pub fn handler_error(message: impl Into<String>) -> Self {
        Self::HandlerError { message: message.into() }
    }

    /// Creates a new streaming selector error.
    #[cfg(feature = "streaming")]
    #[must_use]
    pub fn streaming_selector_error(message: impl Into<String>) -> Self {
        Self::StreamingSelectorError { message: message.into() }
    }
}

// Source position tracking for error reporting

/// A position in source text (1-indexed line and column).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition {
    /// Line number (1-indexed).
    pub line: usize,
    /// Column number (1-indexed, in characters not bytes).
    pub column: usize,
    /// Byte offset from start of input.
    pub offset: usize,
}

impl SourcePosition {
    /// Creates a new source position.
    #[must_use]
    pub const fn new(line: usize, column: usize, offset: usize) -> Self {
        Self { line, column, offset }
    }
}

/// A span in source text with start and end positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    /// Start position (inclusive).
    pub start: SourcePosition,
    /// End position (exclusive).
    pub end: SourcePosition,
}

impl SourceSpan {
    /// Creates a new source span.
    #[must_use]
    pub const fn new(start: SourcePosition, end: SourcePosition) -> Self {
        Self { start, end }
    }

    /// Returns the length of the span in bytes.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end.offset.saturating_sub(self.start.offset)
    }

    /// Returns true if the span is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Context around an error for display purposes.
#[derive(Debug, Clone)]
pub struct SpanContext {
    /// The line of source text containing the error.
    pub line_text: String,
    /// The line number (1-indexed).
    pub line_number: usize,
    /// Column where error starts (1-indexed).
    pub column_start: usize,
    /// Column where error ends (1-indexed).
    pub column_end: usize,
}

impl SpanContext {
    /// Creates a context from source text and span.
    #[must_use]
    pub fn from_source(source: &str, span: &SourceSpan) -> Self {
        let lines: Vec<&str> = source.lines().collect();
        let line_idx = span.start.line.saturating_sub(1);
        let line_text = lines.get(line_idx).unwrap_or(&"").to_string();

        Self {
            line_text: line_text.clone(),
            line_number: span.start.line,
            column_start: span.start.column,
            column_end: if span.start.line == span.end.line {
                span.end.column
            } else {
                line_text.chars().count() + 1
            },
        }
    }

    /// Formats the context with error highlighting.
    ///
    /// Returns a multi-line string showing the error line with carets (^) indicating
    /// the error location.
    #[must_use]
    pub fn format_highlight(&self) -> String {
        use std::fmt::Write;
        let mut result = String::new();
        let _ = writeln!(result, "{:>4} | {}", self.line_number, self.line_text);
        let _ = write!(
            result,
            "     | {}{}",
            " ".repeat(self.column_start.saturating_sub(1)),
            "^".repeat(self.column_end.saturating_sub(self.column_start).max(1))
        );
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::parse("unexpected end of input");
        assert_eq!(err.to_string(), "failed to parse HTML: unexpected end of input");

        let err = Error::invalid_selector("div[");
        assert_eq!(err.to_string(), "invalid CSS selector: div[");

        let err = Error::not_found("div.missing");
        assert_eq!(err.to_string(), "element not found: div.missing");

        let err = Error::attribute_not_found("href");
        assert_eq!(err.to_string(), "attribute 'href' not found on element");
    }

    #[test]
    fn test_source_position_creation() {
        let pos = SourcePosition::new(1, 5, 4);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 5);
        assert_eq!(pos.offset, 4);
    }

    #[test]
    fn test_source_span_length() {
        let span = SourceSpan::new(SourcePosition::new(1, 1, 0), SourcePosition::new(1, 6, 5));
        assert_eq!(span.len(), 5);
        assert!(!span.is_empty());
    }

    #[test]
    fn test_source_span_empty() {
        let pos = SourcePosition::new(1, 1, 0);
        let span = SourceSpan::new(pos, pos);
        assert_eq!(span.len(), 0);
        assert!(span.is_empty());
    }

    #[test]
    fn test_span_context_formatting() {
        let source = "line1\nline2 error here\nline3";
        let span = SourceSpan::new(SourcePosition::new(2, 7, 12), SourcePosition::new(2, 12, 17));
        let ctx = SpanContext::from_source(source, &span);

        assert_eq!(ctx.line_number, 2);
        assert_eq!(ctx.line_text, "line2 error here");
        assert_eq!(ctx.column_start, 7);
        assert_eq!(ctx.column_end, 12);

        let formatted = ctx.format_highlight();
        assert!(formatted.contains("line2 error here"));
        assert!(formatted.contains("^^^^^"));
    }

    #[test]
    fn test_span_context_multiline_span() {
        let source = "line1\nline2 starts here\nline3 continues";
        let span = SourceSpan::new(SourcePosition::new(2, 13, 18), SourcePosition::new(3, 5, 29));
        let ctx = SpanContext::from_source(source, &span);

        assert_eq!(ctx.line_number, 2);
        assert_eq!(ctx.column_start, 13);
        assert!(ctx.column_end > ctx.column_start);
    }

    #[test]
    fn test_span_context_single_char() {
        let source = "hello world";
        let span = SourceSpan::new(SourcePosition::new(1, 7, 6), SourcePosition::new(1, 8, 7));
        let ctx = SpanContext::from_source(source, &span);
        let formatted = ctx.format_highlight();
        assert!(formatted.contains('^'));
        assert!(!formatted.contains("^^"));
    }

    #[test]
    fn test_span_context_invalid_line() {
        let source = "line1\nline2";
        let span =
            SourceSpan::new(SourcePosition::new(10, 1, 100), SourcePosition::new(10, 5, 104));
        let ctx = SpanContext::from_source(source, &span);

        assert_eq!(ctx.line_text, "");
        assert_eq!(ctx.line_number, 10);
    }
}
