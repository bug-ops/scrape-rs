//! Configuration options for HTML parsing.

use napi_derive::napi;

/// Configuration options for HTML parsing.
///
/// @example
/// ```javascript
/// const config = {
///   maxDepth: 256,
///   strictMode: false,
///   preserveWhitespace: false,
///   includeComments: false
/// };
/// const soup = new Soup("<div>Hello</div>", config);
/// ```
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct SoupConfig {
    /// Maximum nesting depth for DOM tree. Default: 512
    pub max_depth: Option<u32>,

    /// Enable strict parsing mode. Default: false
    pub strict_mode: Option<bool>,

    /// Preserve whitespace-only text nodes. Default: false
    pub preserve_whitespace: Option<bool>,

    /// Include comment nodes in DOM. Default: false
    pub include_comments: Option<bool>,
}

impl SoupConfig {
    /// Convert to core SoupConfig.
    #[must_use]
    pub fn to_core(&self) -> scrape_core::SoupConfig {
        scrape_core::SoupConfig::builder()
            .max_depth(self.max_depth.unwrap_or(512) as usize)
            .strict_mode(self.strict_mode.unwrap_or(false))
            .preserve_whitespace(self.preserve_whitespace.unwrap_or(false))
            .include_comments(self.include_comments.unwrap_or(false))
            .build()
    }
}
