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
}
