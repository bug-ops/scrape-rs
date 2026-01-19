//! Main document container type.
//!
//! The [`Soup`] struct is the primary entry point for parsing and querying HTML documents.

use crate::{
    Result, Tag,
    dom::{Document, NodeId, NodeKind},
    parser::{Html5everParser, ParseConfig, Parser},
    query::{
        CompiledSelector, QueryResult, find, find_all, find_all_compiled, find_compiled,
        select_attr, select_text,
    },
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
/// It provides methods for finding elements by CSS selector or tag name.
///
/// # Examples
///
/// ## Basic Parsing
///
/// ```rust
/// use scrape_core::Soup;
///
/// let html = "<html><body><h1>Hello, World!</h1></body></html>";
/// let soup = Soup::parse(html);
///
/// if let Ok(Some(h1)) = soup.find("h1") {
///     assert_eq!(h1.text(), "Hello, World!");
/// }
/// ```
///
/// ## CSS Selectors
///
/// ```rust
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
/// let items = soup.select("div.container > span.item").unwrap();
/// assert_eq!(items.len(), 2);
/// ```
#[derive(Debug)]
pub struct Soup {
    document: Document,
    #[allow(dead_code)]
    config: SoupConfig,
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

        Self { document, config }
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
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    ///
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::from_file(Path::new("index.html")).unwrap();
    /// ```
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let html = std::fs::read_to_string(path)?;
        Ok(Self::parse(&html))
    }

    /// Parses an HTML fragment without wrapping in html/body tags.
    ///
    /// Unlike [`Soup::parse`], this does not wrap content in `<html><body>` structure.
    /// The fragment is parsed as if it appeared inside a `<body>` element.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse_fragment("<span>A</span><span>B</span>");
    /// let spans = soup.find_all("span").unwrap();
    /// assert_eq!(spans.len(), 2);
    /// ```
    #[must_use]
    pub fn parse_fragment(html: &str) -> Self {
        Self::parse_fragment_with_context(html, "body")
    }

    /// Parses an HTML fragment with a custom context element.
    ///
    /// The context element determines parsing behavior:
    /// - `"body"`: Standard HTML elements (default)
    /// - `"table"`: Allows tr/td without explicit tbody
    /// - `"tbody"`: Allows tr directly
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse_fragment_with_context("<tr><td>A</td></tr>", "tbody");
    /// let tr = soup.find("tr").unwrap();
    /// assert!(tr.is_some());
    /// ```
    #[must_use]
    pub fn parse_fragment_with_context(html: &str, context: &str) -> Self {
        Self::parse_fragment_with_config(html, context, SoupConfig::default())
    }

    /// Parses an HTML fragment with custom context and configuration.
    #[must_use]
    pub fn parse_fragment_with_config(html: &str, context: &str, config: SoupConfig) -> Self {
        let parse_config = ParseConfig {
            max_depth: config.max_depth,
            preserve_whitespace: config.preserve_whitespace,
            include_comments: config.include_comments,
        };

        let document = crate::parser::fragment::parse_fragment_impl(html, context, &parse_config)
            .unwrap_or_else(|_| Document::new());

        Self { document, config }
    }

    // ==================== Query Methods ====================

    /// Finds the first element matching the given CSS selector.
    ///
    /// # Errors
    ///
    /// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div><span class=\"item\">Hello</span></div>");
    /// let span = soup.find("span.item").unwrap().unwrap();
    /// assert_eq!(span.text(), "Hello");
    /// ```
    pub fn find(&self, selector: &str) -> QueryResult<Option<Tag<'_>>> {
        find(&self.document, selector).map(|opt| opt.map(|id| Tag::new(&self.document, id)))
    }

    /// Finds all elements matching the given CSS selector.
    ///
    /// # Errors
    ///
    /// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<ul><li>A</li><li>B</li></ul>");
    /// let items = soup.find_all("li").unwrap();
    /// assert_eq!(items.len(), 2);
    /// ```
    pub fn find_all(&self, selector: &str) -> QueryResult<Vec<Tag<'_>>> {
        find_all(&self.document, selector)
            .map(|ids| ids.into_iter().map(|id| Tag::new(&self.document, id)).collect())
    }

    /// Selects elements using a CSS selector.
    ///
    /// This is an alias for [`Soup::find_all`] for users familiar with
    /// the CSS selector API.
    ///
    /// # Errors
    ///
    /// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div class=\"a\"><span class=\"b\">Text</span></div>");
    /// let results = soup.select("div.a > span.b").unwrap();
    /// assert_eq!(results.len(), 1);
    /// ```
    pub fn select(&self, selector: &str) -> QueryResult<Vec<Tag<'_>>> {
        self.find_all(selector)
    }

    /// Finds the first element using a pre-compiled selector.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::{Soup, query::CompiledSelector};
    ///
    /// let selector = CompiledSelector::compile("div.item").unwrap();
    /// let soup = Soup::parse("<div class=\"item\">Text</div>");
    /// let result = soup.find_compiled(&selector);
    /// assert!(result.is_some());
    /// ```
    #[must_use]
    pub fn find_compiled(&self, selector: &CompiledSelector) -> Option<Tag<'_>> {
        find_compiled(&self.document, selector).map(|id| Tag::new(&self.document, id))
    }

    /// Finds all elements using a pre-compiled selector.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::{Soup, query::CompiledSelector};
    ///
    /// let selector = CompiledSelector::compile("li").unwrap();
    /// let soup = Soup::parse("<ul><li>A</li><li>B</li></ul>");
    /// let items = soup.select_compiled(&selector);
    /// assert_eq!(items.len(), 2);
    /// ```
    #[must_use]
    pub fn select_compiled(&self, selector: &CompiledSelector) -> Vec<Tag<'_>> {
        find_all_compiled(&self.document, selector)
            .into_iter()
            .map(|id| Tag::new(&self.document, id))
            .collect()
    }

    /// Extracts text content from all elements matching a CSS selector.
    ///
    /// Returns the concatenated text content of each matching element.
    ///
    /// # Errors
    ///
    /// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<ul><li>First</li><li>Second</li></ul>");
    /// let texts = soup.select_text("li").unwrap();
    /// assert_eq!(texts, vec!["First", "Second"]);
    /// ```
    pub fn select_text(&self, selector: &str) -> QueryResult<Vec<String>> {
        select_text(&self.document, selector)
    }

    /// Extracts attribute values from all elements matching a CSS selector.
    ///
    /// Returns `Some(value)` if the attribute exists, `None` if it doesn't.
    ///
    /// # Errors
    ///
    /// Returns [`QueryError::InvalidSelector`] if the selector syntax is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<a href='/a'>A</a><a>B</a>");
    /// let hrefs = soup.select_attr("a", "href").unwrap();
    /// assert_eq!(hrefs, vec![Some("/a".to_string()), None]);
    /// ```
    pub fn select_attr(&self, selector: &str, attr: &str) -> QueryResult<Vec<Option<String>>> {
        select_attr(&self.document, selector, attr)
    }

    // ==================== Document Methods ====================

    /// Returns the root element of the document.
    ///
    /// This is typically the `<html>` element.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<html><body>text</body></html>");
    /// if let Some(root) = soup.root() {
    ///     assert_eq!(root.name(), Some("html"));
    /// }
    /// ```
    #[must_use]
    pub fn root(&self) -> Option<Tag<'_>> {
        self.document.root().map(|id| Tag::new(&self.document, id))
    }

    /// Returns the document's title, if present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<html><head><title>My Page</title></head></html>");
    /// assert_eq!(soup.title(), Some("My Page".to_string()));
    /// ```
    #[must_use]
    pub fn title(&self) -> Option<String> {
        self.find("title").ok()?.map(|tag| tag.text())
    }

    /// Returns the document's text content with tags stripped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div>Hello <b>World</b></div>");
    /// let text = soup.text();
    /// assert!(text.contains("Hello"));
    /// assert!(text.contains("World"));
    /// ```
    #[must_use]
    pub fn text(&self) -> String {
        let Some(root) = self.document.root() else {
            return String::new();
        };
        let mut result = String::new();
        collect_text(&self.document, root, &mut result);
        result
    }

    /// Returns the document as an HTML string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrape_core::Soup;
    ///
    /// let soup = Soup::parse("<div><span>text</span></div>");
    /// let html = soup.to_html();
    /// assert!(html.contains("<div>"));
    /// assert!(html.contains("<span>"));
    /// ```
    #[must_use]
    pub fn to_html(&self) -> String {
        self.root().map(|tag| tag.outer_html()).unwrap_or_default()
    }
}

/// Recursively collects text content from a subtree.
fn collect_text(doc: &Document, id: NodeId, buf: &mut String) {
    let Some(node) = doc.get(id) else { return };

    match &node.kind {
        NodeKind::Text { content } => buf.push_str(content),
        NodeKind::Element { .. } => {
            for child_id in doc.children(id) {
                collect_text(doc, child_id, buf);
            }
        }
        NodeKind::Comment { .. } => {}
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

    #[test]
    fn test_soup_find() {
        let soup = Soup::parse("<div><span class=\"item\">text</span></div>");
        let result = soup.find("span.item").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name(), Some("span"));
    }

    #[test]
    fn test_soup_find_returns_none() {
        let soup = Soup::parse("<div>text</div>");
        let result = soup.find("span").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_soup_find_invalid_selector() {
        let soup = Soup::parse("<div>text</div>");
        let result = soup.find("[");
        assert!(result.is_err());
    }

    #[test]
    fn test_soup_find_all() {
        let soup = Soup::parse("<ul><li>A</li><li>B</li><li>C</li></ul>");
        let items = soup.find_all("li").unwrap();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_soup_select() {
        let soup = Soup::parse("<div class=\"a\"><span class=\"b\">text</span></div>");
        let results = soup.select("div.a > span.b").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_soup_root() {
        let soup = Soup::parse("<html><body>text</body></html>");
        let root = soup.root();
        assert!(root.is_some());
        assert_eq!(root.unwrap().name(), Some("html"));
    }

    #[test]
    fn test_soup_title() {
        let soup = Soup::parse("<html><head><title>Test Title</title></head></html>");
        assert_eq!(soup.title(), Some("Test Title".to_string()));
    }

    #[test]
    fn test_soup_title_missing() {
        let soup = Soup::parse("<html><body>no title</body></html>");
        assert_eq!(soup.title(), None);
    }

    #[test]
    fn test_soup_text() {
        let soup = Soup::parse("<div>Hello <b>World</b>!</div>");
        let text = soup.text();
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert!(text.contains('!'));
    }

    #[test]
    fn test_soup_to_html() {
        let soup = Soup::parse("<div><span>text</span></div>");
        let html = soup.to_html();
        assert!(html.contains("<div>"));
        assert!(html.contains("<span>text</span>"));
        assert!(html.contains("</div>"));
    }

    #[test]
    fn test_soup_empty_to_html() {
        let soup = Soup::parse("");
        let html = soup.to_html();
        assert!(html.is_empty());
    }

    #[test]
    fn test_soup_find_by_class() {
        let soup = Soup::parse("<div class=\"foo bar\">text</div>");
        let result = soup.find(".foo").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_soup_find_by_id() {
        let soup = Soup::parse("<div id=\"main\">text</div>");
        let result = soup.find("#main").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_soup_find_compound_selector() {
        let soup =
            Soup::parse("<div class=\"foo\" id=\"bar\">text</div><div class=\"foo\">other</div>");
        let result = soup.find("div.foo#bar").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_soup_find_descendant() {
        let soup = Soup::parse("<div><ul><li>item</li></ul></div>");
        let result = soup.find("div li").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name(), Some("li"));
    }

    #[test]
    fn test_soup_find_child_combinator() {
        let soup =
            Soup::parse("<div><span>direct</span></div><div><ul><span>nested</span></ul></div>");
        let results = soup.select("div > span").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_soup_find_with_attribute() {
        let soup = Soup::parse("<input type=\"text\"><input type=\"password\">");
        let result = soup.find("input[type=\"text\"]").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_soup_find_compiled() {
        use crate::query::CompiledSelector;

        let selector = CompiledSelector::compile("div.item").unwrap();
        let soup = Soup::parse("<div class=\"item\">Text</div>");
        let result = soup.find_compiled(&selector);
        assert!(result.is_some());
        assert_eq!(result.unwrap().text(), "Text");
    }

    #[test]
    fn test_soup_select_compiled() {
        use crate::query::CompiledSelector;

        let selector = CompiledSelector::compile("li").unwrap();
        let soup = Soup::parse("<ul><li>A</li><li>B</li></ul>");
        let items = soup.select_compiled(&selector);
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_compiled_selector_reuse() {
        use crate::query::CompiledSelector;

        let selector = CompiledSelector::compile("li").unwrap();

        let soup1 = Soup::parse("<ul><li>A</li></ul>");
        let soup2 = Soup::parse("<ul><li>X</li><li>Y</li></ul>");

        assert_eq!(soup1.select_compiled(&selector).len(), 1);
        assert_eq!(soup2.select_compiled(&selector).len(), 2);
    }
}
