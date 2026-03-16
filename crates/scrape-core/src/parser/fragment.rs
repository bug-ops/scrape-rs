//! HTML fragment parsing without wrapping in html/body.

use super::{ParseConfig, ParseError, ParseResult};
use crate::dom::Document;

/// Parses an HTML fragment with body context.
///
/// Unlike full document parsing, fragment parsing does not wrap content in
/// html/head/body tags. The fragment is parsed as if it appeared inside
/// a `<body>` element.
///
/// Users should use [`crate::Soup::parse_fragment`] instead of this function directly.
///
/// # Errors
///
/// Returns [`ParseError::EmptyInput`] if the input is empty or whitespace-only.
pub fn parse_fragment(html: &str) -> ParseResult<Document> {
    parse_fragment_with_context(html, "body")
}

/// Parses an HTML fragment with a custom context element.
///
/// The context element determines parsing behavior:
/// - `"body"`: Standard HTML elements (default)
/// - `"table"`: Allows tr/td without explicit tbody
/// - `"tbody"`: Allows tr directly
/// - etc.
///
/// Users should use [`crate::Soup::parse_fragment_with_context`] instead of this function directly.
///
/// # Errors
///
/// Returns [`ParseError::EmptyInput`] if the input is empty or whitespace-only.
pub fn parse_fragment_with_context(html: &str, context: &str) -> ParseResult<Document> {
    parse_fragment_impl(html, context, &ParseConfig::default())
}

/// Internal fragment parsing implementation with configuration.
///
/// # Errors
///
/// Returns [`ParseError::EmptyInput`] if the input is empty or whitespace-only.
pub fn parse_fragment_impl(
    html: &str,
    context: &str,
    config: &ParseConfig,
) -> ParseResult<Document> {
    if html.trim().is_empty() {
        return Err(ParseError::EmptyInput);
    }

    super::sink::parse_html_fragment(html, context, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fragment_simple() {
        let doc = parse_fragment("<span>Hello</span>").unwrap();
        assert!(doc.root().is_some());
    }

    #[test]
    fn test_parse_fragment_multiple_roots() {
        let doc = parse_fragment("<span>A</span><span>B</span>").unwrap();
        let root = doc.root().unwrap();
        let node = doc.get(root).unwrap();

        // With multiple roots we use the body element as container
        if let crate::dom::NodeKind::Element { name, .. } = &node.kind {
            // The body element is the container in this implementation
            assert!(!name.is_empty());
        } else {
            panic!("Expected element node");
        }
    }

    #[test]
    fn test_parse_fragment_text_only() {
        let doc = parse_fragment("Just text").unwrap();
        assert!(doc.root().is_some());
    }

    #[test]
    fn test_parse_fragment_empty_returns_error() {
        let result = parse_fragment("");
        assert!(matches!(result, Err(ParseError::EmptyInput)));
    }

    #[test]
    fn test_parse_fragment_whitespace_returns_error() {
        let result = parse_fragment("   ");
        assert!(matches!(result, Err(ParseError::EmptyInput)));
    }

    #[test]
    fn test_parse_fragment_nested() {
        let doc = parse_fragment("<div><p><span>deep</span></p></div>").unwrap();
        assert!(doc.root().is_some());
    }

    #[test]
    fn test_parse_fragment_self_closing() {
        let doc = parse_fragment("<br><hr><img src='test'>").unwrap();
        assert!(doc.root().is_some());
    }

    #[test]
    fn test_parse_fragment_with_context_table() {
        let doc = parse_fragment_with_context("<tr><td>A</td></tr>", "tbody").unwrap();
        assert!(doc.root().is_some());
    }

    #[test]
    fn test_parse_fragment_with_context_body() {
        let doc = parse_fragment_with_context("<div>Test</div>", "body").unwrap();
        assert!(doc.root().is_some());
    }

    #[test]
    fn test_parse_fragment_malformed() {
        let doc = parse_fragment("<div><span>no close").unwrap();
        assert!(doc.root().is_some());
    }

    #[test]
    fn test_parse_fragment_preserves_attributes() {
        let doc = parse_fragment("<div class='test' id='main'>text</div>").unwrap();
        let root = doc.root().unwrap();
        let node = doc.get(root).unwrap();

        if let crate::dom::NodeKind::Element { attributes, .. } = &node.kind {
            assert_eq!(attributes.get("class"), Some(&"test".to_string()));
            assert_eq!(attributes.get("id"), Some(&"main".to_string()));
        } else {
            panic!("Expected element node");
        }
    }

    #[test]
    fn test_parse_fragment_max_depth() {
        let config =
            ParseConfig { max_depth: 5, preserve_whitespace: false, include_comments: false };

        let result = parse_fragment_impl(
            "<div><div><div><div><div><div>too deep</div></div></div></div></div></div>",
            "body",
            &config,
        );

        assert!(matches!(result, Err(ParseError::MaxDepthExceeded { .. })));
    }
}
