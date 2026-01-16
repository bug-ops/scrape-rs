//! HTML extraction logic.

use std::collections::HashMap;

use anyhow::{Context, Result};
use scrape_core::Soup;
use serde::Serialize;

/// Result of extracting data from HTML.
#[derive(Debug, Clone, Serialize)]
pub struct Extraction {
    /// The text content (or attribute value).
    pub text: String,
    /// All attributes of the matched element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<HashMap<String, String>>,
    /// The outer HTML of the matched element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
}

/// Extract data from HTML using a CSS selector.
///
/// # Errors
///
/// Returns an error if the selector is invalid.
pub fn extract(
    html: &str,
    selector: &str,
    attribute: Option<&str>,
    first_only: bool,
    include_html: bool,
) -> Result<Vec<Extraction>> {
    let soup = Soup::parse(html);

    let tags = if first_only {
        soup.find(selector).context("Invalid CSS selector")?.into_iter().collect::<Vec<_>>()
    } else {
        soup.find_all(selector).context("Invalid CSS selector")?
    };

    let mut results = Vec::with_capacity(tags.len());

    for tag in tags {
        let text = attribute.map_or_else(
            || tag.text(),
            |attr| tag.get(attr).unwrap_or_default().to_string(),
        );

        let attrs = if include_html { tag.attrs().cloned() } else { None };

        let html = if include_html { Some(tag.outer_html()) } else { None };

        results.push(Extraction { text, attrs, html });
    }

    Ok(results)
}

/// Extract multiple named selectors from HTML.
///
/// # Errors
///
/// Returns an error if any selector is invalid.
pub fn extract_named(
    html: &str,
    selectors: &[(String, String)],
    attribute: Option<&str>,
    first_only: bool,
) -> Result<HashMap<String, Vec<Extraction>>> {
    let soup = Soup::parse(html);
    let mut results = HashMap::new();

    for (name, selector) in selectors {
        let tags = if first_only {
            soup.find(selector)
                .context(format!("Invalid CSS selector for '{name}'"))?
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            soup.find_all(selector).context(format!("Invalid CSS selector for '{name}'"))?
        };

        let extractions: Vec<Extraction> = tags
            .into_iter()
            .map(|tag| {
                let text = attribute.map_or_else(
                    || tag.text(),
                    |attr| tag.get(attr).unwrap_or_default().to_string(),
                );
                Extraction { text, attrs: None, html: None }
            })
            .collect();

        results.insert(name.clone(), extractions);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_text() {
        let html = "<html><body><h1>Hello World</h1></body></html>";
        let results = extract(html, "h1", None, false, false).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "Hello World");
    }

    #[test]
    fn test_extract_attribute() {
        let html = "<a href=\"/page\">Link</a>";
        let results = extract(html, "a", Some("href"), false, false).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "/page");
    }

    #[test]
    fn test_extract_first_only() {
        let html = "<p>First</p><p>Second</p><p>Third</p>";
        let results = extract(html, "p", None, true, false).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "First");
    }

    #[test]
    fn test_extract_with_html() {
        let html = "<div class=\"item\">Content</div>";
        let results = extract(html, "div", None, false, true).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].html.is_some());
        assert!(results[0].attrs.is_some());
    }

    #[test]
    fn test_extract_no_matches() {
        let html = "<div>Content</div>";
        let results = extract(html, "span", None, false, false).unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn test_extract_named() {
        let html = "<h1>Title</h1><a href=\"/\">Link</a>";
        let selectors = vec![("title".into(), "h1".into()), ("link".into(), "a".into())];
        let results = extract_named(html, &selectors, None, false).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results["title"][0].text, "Title");
        assert_eq!(results["link"][0].text, "Link");
    }

    #[test]
    fn test_extract_invalid_selector() {
        let html = "<div>Content</div>";
        let result = extract(html, "[[[", None, false, false);

        assert!(result.is_err());
    }
}
