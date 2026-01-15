//! Fast whitespace splitting for CSS class matching.
//!
//! This module provides optimized class attribute parsing using SIMD-accelerated
//! whitespace detection. The key function `contains_class` is designed to replace
//! `split_whitespace().any()` in hot paths with better performance.

use memchr::{memchr, memchr3};

/// ASCII whitespace characters: space, tab, newline, carriage return.
#[allow(dead_code)]
const WHITESPACE: &[u8] = b" \t\n\r";

/// Checks if a byte is ASCII whitespace.
#[inline]
fn is_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

/// Checks if a class attribute contains a specific class name.
///
/// This function splits the class attribute by whitespace and checks if any
/// of the resulting tokens exactly matches the target class. It uses SIMD-
/// accelerated whitespace detection for better performance than
/// `split_whitespace().any()`.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "simd")]
/// # fn example() {
/// use scrape_core::simd::contains_class;
///
/// let classes = "container flex-row justify-center items-center";
///
/// assert!(contains_class(classes, "flex-row"));
/// assert!(contains_class(classes, "container"));
/// assert!(!contains_class(classes, "flex"));
/// assert!(!contains_class(classes, "row"));
/// # }
/// ```
///
/// # Edge Cases
///
/// - Empty class attribute: returns false
/// - Empty target: returns false
/// - Multiple whitespace between classes: handled correctly
/// - Leading/trailing whitespace: handled correctly
#[must_use]
pub fn contains_class(class_attr: &str, target: &str) -> bool {
    if target.is_empty() || class_attr.is_empty() {
        return false;
    }

    let bytes = class_attr.as_bytes();
    let target_bytes = target.as_bytes();
    let target_len = target_bytes.len();

    // Early exit if target is longer than class_attr
    if target_len > bytes.len() {
        return false;
    }

    let mut pos = 0;

    while pos < bytes.len() {
        // Skip leading whitespace
        while pos < bytes.len() && is_whitespace(bytes[pos]) {
            pos += 1;
        }

        if pos >= bytes.len() {
            break;
        }

        // Find end of current class token
        let start = pos;
        let end = find_next_whitespace(&bytes[pos..]).map_or(bytes.len(), |offset| pos + offset);

        let token_len = end - start;

        // Check if token matches target
        if token_len == target_len && &bytes[start..end] == target_bytes {
            return true;
        }

        pos = end;
    }

    false
}

/// Finds the position of the next whitespace character.
#[inline]
fn find_next_whitespace(bytes: &[u8]) -> Option<usize> {
    // Use memchr3 for common whitespace (space, tab, newline)
    // then check for carriage return separately
    let space_pos = memchr3(b' ', b'\t', b'\n', bytes);
    let cr_pos = memchr(b'\r', bytes);

    match (space_pos, cr_pos) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// An iterator over individual classes in a class attribute string.
///
/// This iterator yields `&str` slices without allocation, splitting the
/// input by ASCII whitespace.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "simd")]
/// # fn example() {
/// use scrape_core::simd::split_classes;
///
/// let classes: Vec<_> = split_classes("foo bar baz").collect();
/// assert_eq!(classes, vec!["foo", "bar", "baz"]);
///
/// // Handles multiple spaces
/// let classes: Vec<_> = split_classes("  foo   bar  ").collect();
/// assert_eq!(classes, vec!["foo", "bar"]);
/// # }
/// ```
#[must_use]
pub fn split_classes(class_attr: &str) -> ClassIter<'_> {
    ClassIter { remaining: class_attr.as_bytes() }
}

/// Iterator over class names in a class attribute.
///
/// Created by [`split_classes`].
#[derive(Debug, Clone)]
pub struct ClassIter<'a> {
    remaining: &'a [u8],
}

impl<'a> Iterator for ClassIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip leading whitespace
        while !self.remaining.is_empty() && is_whitespace(self.remaining[0]) {
            self.remaining = &self.remaining[1..];
        }

        if self.remaining.is_empty() {
            return None;
        }

        // Find end of current token
        let end = find_next_whitespace(self.remaining).unwrap_or(self.remaining.len());

        let token = &self.remaining[..end];
        self.remaining = &self.remaining[end..];

        // The input was a valid UTF-8 string, and we're only splitting
        // on ASCII whitespace, so the result is also valid UTF-8.
        // Using expect here because the invariant is guaranteed by construction.
        Some(std::str::from_utf8(token).expect("valid UTF-8 slice from valid UTF-8 input"))
    }
}

impl std::iter::FusedIterator for ClassIter<'_> {}

#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests for contains_class
    #[test]
    fn test_contains_class_single() {
        assert!(contains_class("foo", "foo"));
    }

    #[test]
    fn test_contains_class_first() {
        assert!(contains_class("foo bar baz", "foo"));
    }

    #[test]
    fn test_contains_class_middle() {
        assert!(contains_class("foo bar baz", "bar"));
    }

    #[test]
    fn test_contains_class_last() {
        assert!(contains_class("foo bar baz", "baz"));
    }

    #[test]
    fn test_contains_class_not_found() {
        assert!(!contains_class("foo bar baz", "qux"));
    }

    #[test]
    fn test_contains_class_partial_match() {
        assert!(!contains_class("foobar", "foo"));
        assert!(!contains_class("foo bar", "oo"));
        assert!(!contains_class("foo bar", "ba"));
    }

    #[test]
    fn test_contains_class_empty_target() {
        assert!(!contains_class("foo bar", ""));
    }

    #[test]
    fn test_contains_class_empty_attr() {
        assert!(!contains_class("", "foo"));
    }

    #[test]
    fn test_contains_class_both_empty() {
        assert!(!contains_class("", ""));
    }

    #[test]
    fn test_contains_class_whitespace_only() {
        assert!(!contains_class("   ", "foo"));
    }

    #[test]
    fn test_contains_class_leading_whitespace() {
        assert!(contains_class("  foo bar", "foo"));
    }

    #[test]
    fn test_contains_class_trailing_whitespace() {
        assert!(contains_class("foo bar  ", "bar"));
    }

    #[test]
    fn test_contains_class_multiple_spaces() {
        assert!(contains_class("foo    bar", "bar"));
    }

    #[test]
    fn test_contains_class_tabs() {
        assert!(contains_class("foo\tbar\tbaz", "bar"));
    }

    #[test]
    fn test_contains_class_newlines() {
        assert!(contains_class("foo\nbar\nbaz", "bar"));
    }

    #[test]
    fn test_contains_class_mixed_whitespace() {
        assert!(contains_class("foo \t\n bar", "bar"));
    }

    #[test]
    fn test_contains_class_target_longer() {
        assert!(!contains_class("foo", "foobar"));
    }

    // Unit tests for split_classes
    #[test]
    fn test_split_classes_simple() {
        let classes: Vec<_> = split_classes("foo bar baz").collect();
        assert_eq!(classes, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn test_split_classes_single() {
        let classes: Vec<_> = split_classes("foo").collect();
        assert_eq!(classes, vec!["foo"]);
    }

    #[test]
    fn test_split_classes_empty() {
        assert!(split_classes("").next().is_none());
    }

    #[test]
    fn test_split_classes_whitespace_only() {
        assert!(split_classes("   ").next().is_none());
    }

    #[test]
    fn test_split_classes_leading_whitespace() {
        let classes: Vec<_> = split_classes("  foo bar").collect();
        assert_eq!(classes, vec!["foo", "bar"]);
    }

    #[test]
    fn test_split_classes_trailing_whitespace() {
        let classes: Vec<_> = split_classes("foo bar  ").collect();
        assert_eq!(classes, vec!["foo", "bar"]);
    }

    #[test]
    fn test_split_classes_multiple_spaces() {
        let classes: Vec<_> = split_classes("foo    bar").collect();
        assert_eq!(classes, vec!["foo", "bar"]);
    }

    #[test]
    fn test_split_classes_tabs() {
        let classes: Vec<_> = split_classes("foo\tbar").collect();
        assert_eq!(classes, vec!["foo", "bar"]);
    }

    #[test]
    fn test_split_classes_newlines() {
        let classes: Vec<_> = split_classes("foo\nbar").collect();
        assert_eq!(classes, vec!["foo", "bar"]);
    }

    #[test]
    fn test_split_classes_carriage_return() {
        let classes: Vec<_> = split_classes("foo\r\nbar").collect();
        assert_eq!(classes, vec!["foo", "bar"]);
    }

    #[test]
    fn test_class_iter_is_fused() {
        let mut iter = split_classes("foo");
        assert_eq!(iter.next(), Some("foo"));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    // Parity tests: verify SIMD produces same results as scalar
    mod parity {
        use super::*;

        fn scalar_contains_class(class_attr: &str, target: &str) -> bool {
            class_attr.split_whitespace().any(|c| c == target)
        }

        fn scalar_split_classes(class_attr: &str) -> Vec<&str> {
            class_attr.split_whitespace().collect()
        }

        const CLASS_ATTRS: &[&str] = &[
            "",
            "foo",
            "foo bar",
            "foo bar baz",
            "  leading",
            "trailing  ",
            "  both  ",
            "  multiple   spaces  ",
            "tab\tseparated",
            "newline\nseparated",
            "mixed \t\n whitespace",
            "container flex-row justify-center items-center bg-gray-100",
        ];

        const TARGETS: &[&str] =
            &["", "foo", "bar", "baz", "container", "flex-row", "notfound", "fl", "oo", "x"];

        #[test]
        fn parity_contains_class() {
            for attr in CLASS_ATTRS {
                for target in TARGETS {
                    assert_eq!(
                        contains_class(attr, target),
                        scalar_contains_class(attr, target),
                        "mismatch for target {target:?} in {attr:?}"
                    );
                }
            }
        }

        #[test]
        fn parity_split_classes() {
            for attr in CLASS_ATTRS {
                let simd_result: Vec<_> = split_classes(attr).collect();
                let scalar_result = scalar_split_classes(attr);
                assert_eq!(simd_result, scalar_result, "mismatch for {attr:?}");
            }
        }
    }
}
