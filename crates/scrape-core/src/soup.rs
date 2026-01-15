//! Main document container type.
//!
//! The [`Soup`] struct is the primary entry point for parsing and querying HTML documents.

use crate::{
    Result, Tag,
    dom::Document,
    parser::{Html5everParser, ParseConfig, Parser},
};

/// Configuration options for HTML parsing.
///
/// # Examples
///
/// ```rust
/// use scrape_core::SoupConfig;
///
/// let config = SoupConfig::builder().max_depth(256).strict_mode(false).build();
/// ```
#[derive(Debug, Clone)]
pub struct SoupConfig {
    /// Maximum nesting depth for DOM tree.
    pub max_depth: usize,
    /// Enable strict parsing mode (fail on malformed HTML).
    pub strict_mode: bool,
    /// Whether to preserve whitespace-only text nodes.
    pub preserve_whitespace: bool,
    /// Whether to include comment nodes.
    pub include_comments: bool,
}

impl Default for SoupConfig {
    fn default() -> Self {
        Self {
            max_depth: 512,
            strict_mode: false,
            preserve_whitespace: false,
            include_comments: false,
        }
    }
}

impl SoupConfig {
    /// Creates a new configuration builder.
    #[must_use]
    pub fn builder() -> SoupConfigBuilder {
        SoupConfigBuilder::default()
    }
}

/// Builder for [`SoupConfig`].
#[derive(Debug, Default)]
pub struct SoupConfigBuilder {
    max_depth: Option<usize>,
    strict_mode: Option<bool>,
    preserve_whitespace: Option<bool>,
    include_comments: Option<bool>,
}

impl SoupConfigBuilder {
    /// Sets the maximum nesting depth.
    #[must_use]
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Enables or disables strict parsing mode.
    #[must_use]
    pub fn strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = Some(strict);
        self
    }

    /// Enables or disables whitespace preservation.
    #[must_use]
    pub fn preserve_whitespace(mut self, preserve: bool) -> Self {
        self.preserve_whitespace = Some(preserve);
        self
    }

    /// Enables or disables comment inclusion.
    #[must_use]
    pub fn include_comments(mut self, include: bool) -> Self {
        self.include_comments = Some(include);
        self
    }

    /// Builds the configuration.
    #[must_use]
    pub fn build(self) -> SoupConfig {
        SoupConfig {
            max_depth: self.max_depth.unwrap_or(512),
            strict_mode: self.strict_mode.unwrap_or(false),
            preserve_whitespace: self.preserve_whitespace.unwrap_or(false),
            include_comments: self.include_comments.unwrap_or(false),
        }
    }
}

/// A parsed HTML document.
///
/// `Soup` is the main entry point for parsing and querying HTML documents.
/// It provides methods for finding elements by tag name, CSS selector, or
/// other criteria.
///
/// # Examples
///
/// ## Basic Parsing
///
/// ```rust,ignore
/// use scrape_core::Soup;
///
/// let html = "<html><body><h1>Hello, World!</h1></body></html>";
/// let soup = Soup::parse(html);
///
/// if let Some(h1) = soup.find("h1") {
///     assert_eq!(h1.text(), "Hello, World!");
/// }
/// ```
///
/// ## CSS Selectors
///
/// ```rust,ignore
/// use scrape_core::Soup;
///
/// let html = r#"
///     <div class="container">
///         <span class="item">One</span>
///         <span class="item">Two</span>
///     </div>
/// "#;
/// let soup = Soup::parse(html);
///
/// let items: Vec<_> = soup.select("div.container > span.item").collect();
/// assert_eq!(items.len(), 2);
/// ```
#[derive(Debug)]
pub struct Soup {
    document: Document,
    _config: SoupConfig,
}

impl Soup {
    /// Parses an HTML string into a `Soup` document.
    ///
    /// This uses the default configuration. For custom configuration,
    /// use [`Soup::parse_with_config`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<html><body>Hello</body></html>");
    /// ```
    #[must_use]
    pub fn parse(html: &str) -> Self {
        Self::parse_with_config(html, SoupConfig::default())
    }

    /// Parses an HTML string with custom configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::{Soup, SoupConfig};
    ///
    /// let config = SoupConfig::builder().max_depth(128).build();
    /// let soup = Soup::parse_with_config("<html>...</html>", config);
    /// ```
    #[must_use]
    pub fn parse_with_config(html: &str, config: SoupConfig) -> Self {
        let parser = Html5everParser;
        let parse_config = ParseConfig {
            max_depth: config.max_depth,
            preserve_whitespace: config.preserve_whitespace,
            include_comments: config.include_comments,
        };

        let document =
            parser.parse_with_config(html, &parse_config).unwrap_or_else(|_| Document::new());

        Self { document, _config: config }
    }

    /// Returns a reference to the underlying document.
    #[must_use]
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Parses HTML from a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub fn from_file(_path: &std::path::Path) -> Result<Self> {
        // TODO: implement file reading
        todo!("Soup::from_file")
    }

    /// Finds the first element matching the given tag name or CSS selector.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div><span>Hello</span></div>");
    /// let span = soup.find("span").unwrap();
    /// ```
    #[must_use]
    pub fn find(&self, _selector: &str) -> Option<Tag> {
        // TODO: implement find
        todo!("Soup::find")
    }

    /// Finds all elements matching the given tag name or CSS selector.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<ul><li>A</li><li>B</li></ul>");
    /// let items: Vec<_> = soup.find_all("li").collect();
    /// assert_eq!(items.len(), 2);
    /// ```
    pub fn find_all(&self, _selector: &str) -> impl Iterator<Item = Tag> {
        // TODO: implement find_all
        std::iter::empty()
    }

    /// Selects elements using a CSS selector.
    ///
    /// This is an alias for [`Soup::find_all`] for users familiar with
    /// the CSS selector API.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div class=\"a\"><span class=\"b\">Text</span></div>");
    /// let results: Vec<_> = soup.select("div.a > span.b").collect();
    /// ```
    pub fn select(&self, selector: &str) -> impl Iterator<Item = Tag> {
        self.find_all(selector)
    }

    /// Returns the document's title, if present.
    #[must_use]
    pub fn title(&self) -> Option<String> {
        // TODO: implement title extraction
        todo!("Soup::title")
    }

    /// Returns the document's text content with tags stripped.
    #[must_use]
    pub fn text(&self) -> String {
        // TODO: implement text extraction
        todo!("Soup::text")
    }

    /// Returns the document as an HTML string.
    #[must_use]
    pub fn to_html(&self) -> String {
        // TODO: implement HTML serialization
        todo!("Soup::to_html")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soup_config_default() {
        let config = SoupConfig::default();
        assert_eq!(config.max_depth, 512);
        assert!(!config.strict_mode);
        assert!(!config.preserve_whitespace);
        assert!(!config.include_comments);
    }

    #[test]
    fn test_soup_config_builder() {
        let config = SoupConfig::builder()
            .max_depth(128)
            .strict_mode(true)
            .preserve_whitespace(true)
            .include_comments(true)
            .build();
        assert_eq!(config.max_depth, 128);
        assert!(config.strict_mode);
        assert!(config.preserve_whitespace);
        assert!(config.include_comments);
    }

    #[test]
    fn test_soup_parse_creates_document() {
        let soup = Soup::parse("<html><body>Hello</body></html>");
        assert!(soup.document().root().is_some());
    }

    #[test]
    fn test_soup_parse_empty_creates_empty_document() {
        let soup = Soup::parse("");
        assert!(soup.document().is_empty());
    }

    #[test]
    fn test_soup_parse_with_config() {
        let config = SoupConfig::builder().max_depth(256).build();
        let soup = Soup::parse_with_config("<div>Test</div>", config);
        assert!(soup.document().root().is_some());
    }
}
