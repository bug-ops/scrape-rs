//! Shared utility functions for HTML processing.
//!
//! This module provides common utilities used across the library and bindings
//! for HTML text escaping, attribute escaping, and void element detection.

use std::borrow::Cow;

/// Escapes special characters for HTML text content.
///
/// Returns borrowed input when no escaping is needed (common case),
/// avoiding allocation overhead. Only `&`, `<`, and `>` are escaped.
///
/// # Performance
///
/// This function uses a fast-path check to avoid allocation when the input
/// contains no special characters. In typical HTML content, 80-90% of text
/// nodes require no escaping.
///
/// # Examples
///
/// ```rust
/// use std::borrow::Cow;
///
/// use scrape_core::utils::escape_text;
///
/// // No escaping needed - returns borrowed reference
/// let result = escape_text("Hello World");
/// assert!(matches!(result, Cow::Borrowed(_)));
/// assert_eq!(result, "Hello World");
///
/// // Escaping needed - returns owned string
/// let result = escape_text("<script>alert('xss')</script>");
/// assert!(matches!(result, Cow::Owned(_)));
/// assert_eq!(result, "&lt;script&gt;alert('xss')&lt;/script&gt;");
/// ```
#[must_use]
pub fn escape_text(s: &str) -> Cow<'_, str> {
    if !s.contains(['&', '<', '>']) {
        return Cow::Borrowed(s);
    }

    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(c),
        }
    }
    Cow::Owned(result)
}

/// Escapes special characters for HTML attribute values.
///
/// Returns borrowed input when no escaping is needed (common case),
/// avoiding allocation overhead. Escapes `&`, `"`, `<`, and `>`.
///
/// # Performance
///
/// Similar to [`escape_text`], uses a fast-path check to avoid allocation
/// for attribute values without special characters.
///
/// # Examples
///
/// ```rust
/// use std::borrow::Cow;
///
/// use scrape_core::utils::escape_attr;
///
/// // No escaping needed
/// let result = escape_attr("simple-value");
/// assert!(matches!(result, Cow::Borrowed(_)));
///
/// // Escaping needed for quotes
/// let result = escape_attr("value with \"quotes\"");
/// assert_eq!(result, "value with &quot;quotes&quot;");
/// ```
#[must_use]
pub fn escape_attr(s: &str) -> Cow<'_, str> {
    if !s.contains(['&', '"', '<', '>']) {
        return Cow::Borrowed(s);
    }

    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(c),
        }
    }
    Cow::Owned(result)
}

/// Returns true if the element is a void element (no closing tag).
///
/// Void elements are HTML elements that cannot have content and must not
/// have a closing tag. Per the HTML5 specification, these are:
///
/// - `area`, `base`, `br`, `col`, `embed`, `hr`, `img`, `input`
/// - `link`, `meta`, `param`, `source`, `track`, `wbr`
///
/// # Examples
///
/// ```rust
/// use scrape_core::utils::is_void_element;
///
/// assert!(is_void_element("br"));
/// assert!(is_void_element("img"));
/// assert!(is_void_element("input"));
///
/// assert!(!is_void_element("div"));
/// assert!(!is_void_element("span"));
/// assert!(!is_void_element("p"));
/// ```
#[must_use]
pub fn is_void_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_text_no_special_chars() {
        let input = "Hello World";
        let result = escape_text(input);
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_escape_text_with_ampersand() {
        let result = escape_text("Tom & Jerry");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result, "Tom &amp; Jerry");
    }

    #[test]
    fn test_escape_text_with_angle_brackets() {
        let result = escape_text("<tag>");
        assert_eq!(result, "&lt;tag&gt;");
    }

    #[test]
    fn test_escape_text_mixed() {
        let result = escape_text("1 < 2 & 2 > 1");
        assert_eq!(result, "1 &lt; 2 &amp; 2 &gt; 1");
    }

    #[test]
    fn test_escape_text_empty() {
        let result = escape_text("");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "");
    }

    #[test]
    fn test_escape_attr_no_special_chars() {
        let input = "simple-value";
        let result = escape_attr(input);
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "simple-value");
    }

    #[test]
    fn test_escape_attr_with_quotes() {
        let result = escape_attr("say \"hello\"");
        assert_eq!(result, "say &quot;hello&quot;");
    }

    #[test]
    fn test_escape_attr_mixed() {
        let result = escape_attr("<a href=\"&\">link</a>");
        assert_eq!(result, "&lt;a href=&quot;&amp;&quot;&gt;link&lt;/a&gt;");
    }

    #[test]
    fn test_is_void_element_true() {
        for tag in [
            "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
            "source", "track", "wbr",
        ] {
            assert!(is_void_element(tag), "{tag} should be a void element");
        }
    }

    #[test]
    fn test_is_void_element_false() {
        for tag in ["div", "span", "p", "a", "ul", "li", "table", "form", "script", "style"] {
            assert!(!is_void_element(tag), "{tag} should not be a void element");
        }
    }
}
