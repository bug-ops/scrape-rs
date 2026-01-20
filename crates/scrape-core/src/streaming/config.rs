//! Configuration types for streaming parser.

/// Configuration for streaming HTML parser.
///
/// Controls various aspects of the streaming parser behavior.
#[cfg(feature = "streaming")]
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Buffer size for chunk processing (in bytes).
    pub buffer_size: usize,
    /// Whether to use strict HTML parsing rules.
    pub strict_mode: bool,
    /// Whether to preserve comments in the output.
    pub preserve_comments: bool,
}

#[cfg(feature = "streaming")]
impl Default for StreamingConfig {
    fn default() -> Self {
        Self { buffer_size: 8192, strict_mode: false, preserve_comments: false }
    }
}

#[cfg(feature = "streaming")]
impl StreamingConfig {
    /// Creates a new streaming config with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the buffer size for chunk processing.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let config = StreamingConfig::new().buffer_size(16384);
    /// ```
    #[must_use]
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Enables or disables strict mode.
    ///
    /// In strict mode, parsing errors will fail immediately rather than
    /// attempting error recovery.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let config = StreamingConfig::new().strict_mode(true);
    /// ```
    #[must_use]
    pub fn strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Enables or disables comment preservation.
    ///
    /// When enabled, HTML comments are preserved in the output.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let config = StreamingConfig::new().preserve_comments(true);
    /// ```
    #[must_use]
    pub fn preserve_comments(mut self, preserve: bool) -> Self {
        self.preserve_comments = preserve;
        self
    }
}

/// Configuration for HTML rewriter.
///
/// Controls behavior of the HTML rewriting functionality.
#[cfg(feature = "streaming")]
#[derive(Debug, Clone)]
pub struct RewriterConfig {
    /// Whether to use strict HTML parsing rules.
    pub strict_mode: bool,
    /// Whether to preserve comments in the output.
    pub preserve_comments: bool,
    /// Maximum nesting depth before failing.
    pub max_nesting_depth: usize,
}

#[cfg(feature = "streaming")]
impl Default for RewriterConfig {
    fn default() -> Self {
        Self { strict_mode: false, preserve_comments: false, max_nesting_depth: 512 }
    }
}

#[cfg(feature = "streaming")]
impl RewriterConfig {
    /// Creates a new rewriter config with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables or disables strict mode.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let config = RewriterConfig::new().strict_mode(true);
    /// ```
    #[must_use]
    pub fn strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Enables or disables comment preservation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let config = RewriterConfig::new().preserve_comments(true);
    /// ```
    #[must_use]
    pub fn preserve_comments(mut self, preserve: bool) -> Self {
        self.preserve_comments = preserve;
        self
    }

    /// Sets the maximum nesting depth.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let config = RewriterConfig::new().max_nesting_depth(1024);
    /// ```
    #[must_use]
    pub fn max_nesting_depth(mut self, depth: usize) -> Self {
        self.max_nesting_depth = depth;
        self
    }
}

#[cfg(all(test, feature = "streaming"))]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_config_default() {
        let config = StreamingConfig::default();
        assert_eq!(config.buffer_size, 8192);
        assert!(!config.strict_mode);
        assert!(!config.preserve_comments);
    }

    #[test]
    fn test_streaming_config_builder() {
        let config =
            StreamingConfig::new().buffer_size(16384).strict_mode(true).preserve_comments(true);

        assert_eq!(config.buffer_size, 16384);
        assert!(config.strict_mode);
        assert!(config.preserve_comments);
    }

    #[test]
    fn test_rewriter_config_default() {
        let config = RewriterConfig::default();
        assert!(!config.strict_mode);
        assert!(!config.preserve_comments);
        assert_eq!(config.max_nesting_depth, 512);
    }

    #[test]
    fn test_rewriter_config_builder() {
        let config =
            RewriterConfig::new().strict_mode(true).preserve_comments(true).max_nesting_depth(1024);

        assert!(config.strict_mode);
        assert!(config.preserve_comments);
        assert_eq!(config.max_nesting_depth, 1024);
    }
}
