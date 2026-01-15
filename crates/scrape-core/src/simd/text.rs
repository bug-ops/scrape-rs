//! SIMD-accelerated substring search for HTML content.
//!
//! This module provides fast substring search using the `memchr::memmem`
//! module, which automatically uses SIMD instructions when available.

use memchr::memmem;

/// Finds the first occurrence of a needle in the haystack.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "simd")]
/// # fn example() {
/// use scrape_core::simd::find_text;
///
/// assert_eq!(find_text(b"Hello World", b"World"), Some(6));
/// assert_eq!(find_text(b"abcabc", b"bc"), Some(1));
/// assert_eq!(find_text(b"no match", b"xyz"), None);
/// # }
/// ```
#[inline]
#[must_use]
pub fn find_text(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    memmem::find(haystack, needle)
}

/// Returns an iterator over all occurrences of a needle in the haystack.
///
/// The iterator yields the starting position of each match.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "simd")]
/// # fn example() {
/// use scrape_core::simd::find_text_all;
///
/// let positions: Vec<_> = find_text_all(b"abcabc", b"bc").collect();
/// assert_eq!(positions, vec![1, 4]);
/// # }
/// ```
#[inline]
#[must_use]
pub fn find_text_all<'h, 'n>(haystack: &'h [u8], needle: &'n [u8]) -> memmem::FindIter<'h, 'n> {
    memmem::find_iter(haystack, needle)
}

/// Counts the number of occurrences of a needle in the haystack.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "simd")]
/// # fn example() {
/// use scrape_core::simd::count_text;
///
/// assert_eq!(count_text(b"aaa", b"a"), 3);
/// assert_eq!(count_text(b"abcabc", b"bc"), 2);
/// assert_eq!(count_text(b"no match", b"xyz"), 0);
/// # }
/// ```
#[inline]
#[must_use]
pub fn count_text(haystack: &[u8], needle: &[u8]) -> usize {
    memmem::find_iter(haystack, needle).count()
}

/// Checks if a needle exists in the haystack.
///
/// This is more efficient than `find_text(...).is_some()` when you only
/// need to check existence, as it can short-circuit on the first match.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "simd")]
/// # fn example() {
/// use scrape_core::simd::contains_text;
///
/// assert!(contains_text(b"Hello World", b"World"));
/// assert!(!contains_text(b"Hello World", b"Rust"));
/// # }
/// ```
#[inline]
#[must_use]
pub fn contains_text(haystack: &[u8], needle: &[u8]) -> bool {
    memmem::find(haystack, needle).is_some()
}

/// A precompiled pattern for repeated substring searches.
///
/// When searching for the same needle multiple times in different haystacks,
/// using `TextFinder` is more efficient than calling `find_text` repeatedly,
/// as the pattern preprocessing is done once during construction.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "simd")]
/// # fn example() {
/// use scrape_core::simd::TextFinder;
///
/// let finder = TextFinder::new(b"search");
///
/// assert_eq!(finder.find(b"search for text"), Some(0));
/// assert_eq!(finder.find(b"do a search"), Some(5));
/// assert!(!finder.contains(b"no match here"));
/// # }
/// ```
pub struct TextFinder<'n> {
    finder: memmem::Finder<'n>,
}

impl<'n> TextFinder<'n> {
    /// Creates a new `TextFinder` with the given needle.
    ///
    /// The needle is precompiled for efficient repeated searches.
    #[must_use]
    pub fn new(needle: &'n [u8]) -> Self {
        Self { finder: memmem::Finder::new(needle) }
    }

    /// Finds the first occurrence of the needle in the haystack.
    #[inline]
    #[must_use]
    pub fn find(&self, haystack: &[u8]) -> Option<usize> {
        self.finder.find(haystack)
    }

    /// Checks if the needle exists in the haystack.
    #[inline]
    #[must_use]
    pub fn contains(&self, haystack: &[u8]) -> bool {
        self.finder.find(haystack).is_some()
    }

    /// Returns an iterator over all occurrences in the haystack.
    #[inline]
    #[must_use]
    pub fn find_iter<'h>(&'n self, haystack: &'h [u8]) -> memmem::FindIter<'h, 'n> {
        self.finder.find_iter(haystack)
    }

    /// Counts the number of occurrences in the haystack.
    #[inline]
    #[must_use]
    pub fn count(&self, haystack: &[u8]) -> usize {
        self.finder.find_iter(haystack).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests for find_text
    #[test]
    fn test_find_text_at_start() {
        assert_eq!(find_text(b"hello world", b"hello"), Some(0));
    }

    #[test]
    fn test_find_text_in_middle() {
        assert_eq!(find_text(b"hello world", b"world"), Some(6));
    }

    #[test]
    fn test_find_text_not_found() {
        assert_eq!(find_text(b"hello world", b"rust"), None);
    }

    #[test]
    fn test_find_text_empty_needle() {
        assert_eq!(find_text(b"hello", b""), Some(0));
    }

    #[test]
    fn test_find_text_empty_haystack() {
        assert_eq!(find_text(b"", b"x"), None);
    }

    #[test]
    fn test_find_text_both_empty() {
        assert_eq!(find_text(b"", b""), Some(0));
    }

    #[test]
    fn test_find_text_needle_longer() {
        assert_eq!(find_text(b"hi", b"hello"), None);
    }

    #[test]
    fn test_find_text_overlapping() {
        assert_eq!(find_text(b"aaa", b"aa"), Some(0));
    }

    // Unit tests for find_text_all
    #[test]
    fn test_find_text_all_multiple() {
        let positions: Vec<_> = find_text_all(b"abcabcabc", b"abc").collect();
        assert_eq!(positions, vec![0, 3, 6]);
    }

    #[test]
    fn test_find_text_all_none() {
        assert!(find_text_all(b"hello", b"xyz").next().is_none());
    }

    #[test]
    fn test_find_text_all_overlapping() {
        let positions: Vec<_> = find_text_all(b"aaaa", b"aa").collect();
        assert_eq!(positions, vec![0, 2]);
    }

    // Unit tests for count_text
    #[test]
    fn test_count_text_multiple() {
        assert_eq!(count_text(b"abcabcabc", b"abc"), 3);
    }

    #[test]
    fn test_count_text_none() {
        assert_eq!(count_text(b"hello", b"xyz"), 0);
    }

    #[test]
    fn test_count_text_single() {
        assert_eq!(count_text(b"hello", b"ell"), 1);
    }

    // Unit tests for contains_text
    #[test]
    fn test_contains_text_found() {
        assert!(contains_text(b"hello world", b"world"));
    }

    #[test]
    fn test_contains_text_not_found() {
        assert!(!contains_text(b"hello world", b"rust"));
    }

    #[test]
    fn test_contains_text_empty_needle() {
        assert!(contains_text(b"hello", b""));
    }

    // Unit tests for TextFinder
    #[test]
    fn test_text_finder_find() {
        let finder = TextFinder::new(b"world");
        assert_eq!(finder.find(b"hello world"), Some(6));
        assert_eq!(finder.find(b"world first"), Some(0));
        assert_eq!(finder.find(b"no match"), None);
    }

    #[test]
    fn test_text_finder_contains() {
        let finder = TextFinder::new(b"test");
        assert!(finder.contains(b"this is a test"));
        assert!(!finder.contains(b"no match here"));
    }

    #[test]
    fn test_text_finder_find_iter() {
        let finder = TextFinder::new(b"ab");
        let positions: Vec<_> = finder.find_iter(b"abcab").collect();
        assert_eq!(positions, vec![0, 3]);
    }

    #[test]
    fn test_text_finder_count() {
        let finder = TextFinder::new(b"x");
        assert_eq!(finder.count(b"x x x"), 3);
    }

    // Parity tests: verify SIMD produces same results as scalar
    mod parity {
        use super::*;

        fn scalar_find_text(haystack: &[u8], needle: &[u8]) -> Option<usize> {
            if needle.is_empty() {
                return Some(0);
            }
            if needle.len() > haystack.len() {
                return None;
            }
            haystack.windows(needle.len()).position(|w| w == needle)
        }

        fn scalar_count_text(haystack: &[u8], needle: &[u8]) -> usize {
            if needle.is_empty() {
                return haystack.len() + 1;
            }
            if needle.len() > haystack.len() {
                return 0;
            }
            let mut count = 0;
            let mut pos = 0;
            while pos + needle.len() <= haystack.len() {
                if &haystack[pos..pos + needle.len()] == needle {
                    count += 1;
                    pos += needle.len();
                } else {
                    pos += 1;
                }
            }
            count
        }

        const TEST_CASES: &[(&[u8], &[u8])] = &[
            (b"", b""),
            (b"", b"x"),
            (b"x", b""),
            (b"hello", b"ell"),
            (b"hello", b"xyz"),
            (b"aaa", b"aa"),
            (b"abcabcabc", b"abc"),
            (b"needle in haystack", b"needle"),
            (b"at the end", b"end"),
            (b"multiline\ntext\nsearch", b"\n"),
        ];

        #[test]
        fn parity_find_text() {
            for (haystack, needle) in TEST_CASES {
                let needle_str = String::from_utf8_lossy(needle);
                let haystack_str = String::from_utf8_lossy(haystack);
                assert_eq!(
                    find_text(haystack, needle),
                    scalar_find_text(haystack, needle),
                    "mismatch for needle {needle_str:?} in {haystack_str:?}"
                );
            }
        }

        #[test]
        fn parity_contains_text() {
            for (haystack, needle) in TEST_CASES {
                let needle_str = String::from_utf8_lossy(needle);
                let haystack_str = String::from_utf8_lossy(haystack);
                assert_eq!(
                    contains_text(haystack, needle),
                    scalar_find_text(haystack, needle).is_some(),
                    "mismatch for needle {needle_str:?} in {haystack_str:?}"
                );
            }
        }
    }
}
