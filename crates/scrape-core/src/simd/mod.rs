//! SIMD-accelerated operations for HTML parsing.
//!
//! This module provides optimized byte scanning and text search
//! using SIMD instructions where available, with automatic
//! fallback to scalar implementations via the `memchr` crate.
//!
//! # Platform Support
//!
//! | Platform   | SIMD Instructions |
//! |------------|-------------------|
//! | `x86_64`   | SSE2, SSE4.2, AVX2 (auto-detected at runtime) |
//! | `aarch64`  | NEON |
//! | `wasm32`   | SIMD128 |
//! | Other      | Scalar fallback |
//!
//! # Feature Flag
//!
//! This module is gated behind the `simd` feature:
//!
//! ```toml
//! [dependencies]
//! scrape-core = { version = "0.1", features = ["simd"] }
//! ```
//!
//! # Examples
//!
//! ```rust
//! # #[cfg(feature = "simd")]
//! use scrape_core::simd::{contains_class, find_tag_start, find_text};
//!
//! # #[cfg(feature = "simd")]
//! # fn example() {
//! // Fast byte scanning
//! let html = b"<div>Hello</div>";
//! assert_eq!(find_tag_start(html), Some(0));
//!
//! // Fast class matching
//! let classes = "container flex-row justify-center";
//! assert!(contains_class(classes, "flex-row"));
//!
//! // Fast text search
//! let content = b"Hello World";
//! assert_eq!(find_text(content, b"World"), Some(6));
//! # }
//! ```

mod scan;
mod text;
mod whitespace;

pub use scan::{
    find_attr_delimiter, find_close_tag, find_self_close, find_tag_end, find_tag_start,
    scan_until_any,
};
pub use text::{TextFinder, contains_text, count_text, find_text, find_text_all};
pub use whitespace::{ClassIter, contains_class, split_classes};

/// Returns a string describing the current SIMD implementation.
///
/// This is primarily for diagnostics and logging purposes.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "simd")]
/// # fn example() {
/// use scrape_core::simd::simd_impl;
///
/// println!("Using SIMD backend: {}", simd_impl());
/// # }
/// ```
#[must_use]
pub fn simd_impl() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return "x86_64/AVX2";
        }
        if is_x86_feature_detected!("sse4.2") {
            return "x86_64/SSE4.2";
        }
        "x86_64/SSE2"
    }

    #[cfg(target_arch = "aarch64")]
    {
        "aarch64/NEON"
    }

    #[cfg(target_arch = "wasm32")]
    {
        #[cfg(target_feature = "simd128")]
        {
            "wasm32/SIMD128"
        }
        #[cfg(not(target_feature = "simd128"))]
        {
            "wasm32/scalar"
        }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "wasm32")))]
    {
        "scalar"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_impl_returns_string() {
        let impl_name = simd_impl();
        assert!(!impl_name.is_empty());
    }
}
