//! SIMD-accelerated byte scanning for HTML parsing.
//!
//! This module provides fast single-byte and multi-byte scanning using
//! the `memchr` crate, which automatically uses SIMD instructions when
//! available.

use memchr::{memchr, memchr2, memchr3};

/// Finds the first occurrence of `<` in the byte slice.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "simd")]
/// # fn example() {
/// use scrape_core::simd::find_tag_start;
///
/// assert_eq!(find_tag_start(b"<div>"), Some(0));
/// assert_eq!(find_tag_start(b"Hello <span>"), Some(6));
/// assert_eq!(find_tag_start(b"no tags here"), None);
/// # }
/// ```
#[inline]
#[must_use]
pub fn find_tag_start(bytes: &[u8]) -> Option<usize> {
    memchr(b'<', bytes)
}

/// Finds the first occurrence of `>` in the byte slice.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "simd")]
/// # fn example() {
/// use scrape_core::simd::find_tag_end;
///
/// assert_eq!(find_tag_end(b"<div>"), Some(4));
/// assert_eq!(find_tag_end(b"class=\"foo\">"), Some(11));
/// assert_eq!(find_tag_end(b"no closing"), None);
/// # }
/// ```
#[inline]
#[must_use]
pub fn find_tag_end(bytes: &[u8]) -> Option<usize> {
    memchr(b'>', bytes)
}

/// Finds the first occurrence of any byte in the needle set.
///
/// Optimized for 1-3 byte needles using `memchr`, `memchr2`, or `memchr3`.
/// For larger needle sets, falls back to a simple loop.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "simd")]
/// # fn example() {
/// use scrape_core::simd::scan_until_any;
///
/// // Find first whitespace
/// assert_eq!(scan_until_any(b"hello world", &[b' ', b'\t', b'\n']), Some(5));
///
/// // Find first delimiter
/// assert_eq!(scan_until_any(b"key=value", &[b'=', b':']), Some(3));
/// # }
/// ```
#[inline]
#[must_use]
pub fn scan_until_any(bytes: &[u8], needles: &[u8]) -> Option<usize> {
    match needles.len() {
        0 => None,
        1 => memchr(needles[0], bytes),
        2 => memchr2(needles[0], needles[1], bytes),
        3 => memchr3(needles[0], needles[1], needles[2], bytes),
        _ => bytes.iter().position(|b| needles.contains(b)),
    }
}

/// Finds the first attribute delimiter (`"`, `'`, or `>`).
///
/// Useful for parsing attribute values in HTML tags.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "simd")]
/// # fn example() {
/// use scrape_core::simd::find_attr_delimiter;
///
/// assert_eq!(find_attr_delimiter(b"class=\"foo\""), Some(6));
/// assert_eq!(find_attr_delimiter(b"class='bar'"), Some(6));
/// assert_eq!(find_attr_delimiter(b"disabled>"), Some(8));
/// # }
/// ```
#[inline]
#[must_use]
pub fn find_attr_delimiter(bytes: &[u8]) -> Option<usize> {
    memchr3(b'"', b'\'', b'>', bytes)
}

/// Finds the first occurrence of the `</` closing tag sequence.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "simd")]
/// # fn example() {
/// use scrape_core::simd::find_close_tag;
///
/// assert_eq!(find_close_tag(b"Hello</div>"), Some(5));
/// assert_eq!(find_close_tag(b"<div><span></span></div>"), Some(11));
/// assert_eq!(find_close_tag(b"no close tag"), None);
/// # }
/// ```
#[must_use]
pub fn find_close_tag(bytes: &[u8]) -> Option<usize> {
    let mut start = 0;
    while start < bytes.len() {
        if let Some(pos) = memchr(b'<', &bytes[start..]) {
            let absolute_pos = start + pos;
            if absolute_pos + 1 < bytes.len() && bytes[absolute_pos + 1] == b'/' {
                return Some(absolute_pos);
            }
            start = absolute_pos + 1;
        } else {
            return None;
        }
    }
    None
}

/// Finds the first occurrence of the `/>` self-closing tag sequence.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "simd")]
/// # fn example() {
/// use scrape_core::simd::find_self_close;
///
/// assert_eq!(find_self_close(b"<br/>"), Some(3));
/// assert_eq!(find_self_close(b"<img src=\"x\" />"), Some(13));
/// assert_eq!(find_self_close(b"<div>content</div>"), None);
/// # }
/// ```
#[must_use]
pub fn find_self_close(bytes: &[u8]) -> Option<usize> {
    let mut start = 0;
    while start < bytes.len() {
        if let Some(pos) = memchr(b'/', &bytes[start..]) {
            let absolute_pos = start + pos;
            if absolute_pos + 1 < bytes.len() && bytes[absolute_pos + 1] == b'>' {
                return Some(absolute_pos);
            }
            start = absolute_pos + 1;
        } else {
            return None;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests for find_tag_start
    #[test]
    fn test_find_tag_start_at_beginning() {
        assert_eq!(find_tag_start(b"<div>"), Some(0));
    }

    #[test]
    fn test_find_tag_start_in_middle() {
        assert_eq!(find_tag_start(b"hello <span>"), Some(6));
    }

    #[test]
    fn test_find_tag_start_not_found() {
        assert_eq!(find_tag_start(b"no tags"), None);
    }

    #[test]
    fn test_find_tag_start_empty() {
        assert_eq!(find_tag_start(b""), None);
    }

    #[test]
    fn test_find_tag_start_multiple() {
        assert_eq!(find_tag_start(b"a<b>c<d>"), Some(1));
    }

    // Unit tests for find_tag_end
    #[test]
    fn test_find_tag_end_simple() {
        assert_eq!(find_tag_end(b"<div>"), Some(4));
    }

    #[test]
    fn test_find_tag_end_with_attrs() {
        assert_eq!(find_tag_end(b"<div class=\"foo\">"), Some(16));
    }

    #[test]
    fn test_find_tag_end_not_found() {
        assert_eq!(find_tag_end(b"<div class"), None);
    }

    #[test]
    fn test_find_tag_end_empty() {
        assert_eq!(find_tag_end(b""), None);
    }

    // Unit tests for scan_until_any
    #[test]
    fn test_scan_until_any_single() {
        assert_eq!(scan_until_any(b"hello world", b" "), Some(5));
    }

    #[test]
    fn test_scan_until_any_two() {
        assert_eq!(scan_until_any(b"key=value", b"=:"), Some(3));
    }

    #[test]
    fn test_scan_until_any_three() {
        assert_eq!(scan_until_any(b"a b\tc", b" \t\n"), Some(1));
    }

    #[test]
    fn test_scan_until_any_many() {
        assert_eq!(scan_until_any(b"hello!", b"!?.,"), Some(5));
    }

    #[test]
    fn test_scan_until_any_not_found() {
        assert_eq!(scan_until_any(b"hello", b"xy"), None);
    }

    #[test]
    fn test_scan_until_any_empty_needles() {
        assert_eq!(scan_until_any(b"hello", &[]), None);
    }

    #[test]
    fn test_scan_until_any_empty_haystack() {
        assert_eq!(scan_until_any(b"", b"a"), None);
    }

    // Unit tests for find_attr_delimiter
    #[test]
    fn test_find_attr_delimiter_double_quote() {
        assert_eq!(find_attr_delimiter(b"class=\"foo\""), Some(6));
    }

    #[test]
    fn test_find_attr_delimiter_single_quote() {
        assert_eq!(find_attr_delimiter(b"class='foo'"), Some(6));
    }

    #[test]
    fn test_find_attr_delimiter_end_bracket() {
        assert_eq!(find_attr_delimiter(b"disabled>"), Some(8));
    }

    #[test]
    fn test_find_attr_delimiter_not_found() {
        assert_eq!(find_attr_delimiter(b"no delimiter"), None);
    }

    // Unit tests for find_close_tag
    #[test]
    fn test_find_close_tag_simple() {
        assert_eq!(find_close_tag(b"text</div>"), Some(4));
    }

    #[test]
    fn test_find_close_tag_nested() {
        assert_eq!(find_close_tag(b"<div><span></span></div>"), Some(11));
    }

    #[test]
    fn test_find_close_tag_not_found() {
        assert_eq!(find_close_tag(b"<div>no close"), None);
    }

    #[test]
    fn test_find_close_tag_open_not_close() {
        assert_eq!(find_close_tag(b"<div><span>"), None);
    }

    #[test]
    fn test_find_close_tag_empty() {
        assert_eq!(find_close_tag(b""), None);
    }

    #[test]
    fn test_find_close_tag_at_start() {
        assert_eq!(find_close_tag(b"</div>"), Some(0));
    }

    // Unit tests for find_self_close
    #[test]
    fn test_find_self_close_br() {
        assert_eq!(find_self_close(b"<br/>"), Some(3));
    }

    #[test]
    fn test_find_self_close_with_space() {
        assert_eq!(find_self_close(b"<img src=\"x\" />"), Some(13));
    }

    #[test]
    fn test_find_self_close_not_found() {
        assert_eq!(find_self_close(b"<div>text</div>"), None);
    }

    #[test]
    fn test_find_self_close_empty() {
        assert_eq!(find_self_close(b""), None);
    }

    #[test]
    fn test_find_self_close_slash_not_followed_by_gt() {
        assert_eq!(find_self_close(b"a/b>c"), None);
    }

    // Parity tests: verify SIMD produces same results as scalar
    mod parity {
        use super::*;

        fn scalar_find_tag_start(bytes: &[u8]) -> Option<usize> {
            bytes.iter().position(|&b| b == b'<')
        }

        fn scalar_find_tag_end(bytes: &[u8]) -> Option<usize> {
            bytes.iter().position(|&b| b == b'>')
        }

        fn scalar_find_close_tag(bytes: &[u8]) -> Option<usize> {
            bytes.windows(2).position(|w| w == b"</")
        }

        fn scalar_find_self_close(bytes: &[u8]) -> Option<usize> {
            bytes.windows(2).position(|w| w == b"/>")
        }

        const TEST_CASES: &[&[u8]] = &[
            b"",
            b"<",
            b">",
            b"<>",
            b"<div>",
            b"hello<world>",
            b"no tags here",
            b"</div>",
            b"<br/>",
            b"< not a tag",
            b"multi\n<line>\ntext",
            b"<<<<>>>>",
            b"</></></>",
        ];

        #[test]
        fn parity_find_tag_start() {
            for case in TEST_CASES {
                let case_str = String::from_utf8_lossy(case);
                assert_eq!(
                    find_tag_start(case),
                    scalar_find_tag_start(case),
                    "mismatch for {case_str:?}"
                );
            }
        }

        #[test]
        fn parity_find_tag_end() {
            for case in TEST_CASES {
                let case_str = String::from_utf8_lossy(case);
                assert_eq!(
                    find_tag_end(case),
                    scalar_find_tag_end(case),
                    "mismatch for {case_str:?}"
                );
            }
        }

        #[test]
        fn parity_find_close_tag() {
            for case in TEST_CASES {
                let case_str = String::from_utf8_lossy(case);
                assert_eq!(
                    find_close_tag(case),
                    scalar_find_close_tag(case),
                    "mismatch for {case_str:?}"
                );
            }
        }

        #[test]
        fn parity_find_self_close() {
            for case in TEST_CASES {
                let case_str = String::from_utf8_lossy(case);
                assert_eq!(
                    find_self_close(case),
                    scalar_find_self_close(case),
                    "mismatch for {case_str:?}"
                );
            }
        }
    }
}
