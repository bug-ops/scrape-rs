//! Configuration options for HTML parsing.

use pyo3::prelude::*;

/// Configuration options for HTML parsing.
///
/// Example:
///     >>> config = SoupConfig(max_depth=256, strict_mode=True)
///     >>> soup = Soup("<div>Hello</div>", config=config)
#[pyclass(name = "SoupConfig", skip_from_py_object)]
#[derive(Debug, Clone)]
pub struct PySoupConfig {
    /// Maximum nesting depth for DOM tree (default: 512).
    #[pyo3(get, set)]
    pub max_depth: usize,

    /// Enable strict parsing mode (default: false).
    #[pyo3(get, set)]
    pub strict_mode: bool,

    /// Preserve whitespace-only text nodes (default: false).
    #[pyo3(get, set)]
    pub preserve_whitespace: bool,

    /// Include comment nodes in DOM (default: false).
    #[pyo3(get, set)]
    pub include_comments: bool,
}

#[pymethods]
impl PySoupConfig {
    /// Create a new configuration with optional parameters.
    ///
    /// Args:
    ///     max_depth: Maximum nesting depth for DOM tree.
    ///     strict_mode: Enable strict parsing mode.
    ///     preserve_whitespace: Preserve whitespace-only text nodes.
    ///     include_comments: Include comment nodes in DOM.
    #[new]
    #[pyo3(signature = (
        max_depth = 512,
        strict_mode = false,
        preserve_whitespace = false,
        include_comments = false
    ))]
    fn new(
        max_depth: usize,
        strict_mode: bool,
        preserve_whitespace: bool,
        include_comments: bool,
    ) -> Self {
        Self { max_depth, strict_mode, preserve_whitespace, include_comments }
    }

    fn __repr__(&self) -> String {
        format!(
            "SoupConfig(max_depth={}, strict_mode={}, preserve_whitespace={}, include_comments={})",
            self.max_depth, self.strict_mode, self.preserve_whitespace, self.include_comments
        )
    }
}

impl PySoupConfig {
    /// Convert to core SoupConfig.
    pub fn to_core(&self) -> scrape_core::SoupConfig {
        scrape_core::SoupConfig::builder()
            .max_depth(self.max_depth)
            .strict_mode(self.strict_mode)
            .preserve_whitespace(self.preserve_whitespace)
            .include_comments(self.include_comments)
            .build()
    }
}

impl Default for PySoupConfig {
    fn default() -> Self {
        Self {
            max_depth: 512,
            strict_mode: false,
            preserve_whitespace: false,
            include_comments: false,
        }
    }
}
