//! Parallel batch parsing using Rayon.
//!
//! This module provides functions for parsing multiple HTML documents in parallel,
//! utilizing all available CPU cores for maximum throughput.

use rayon::prelude::*;

use crate::{Soup, SoupConfig};

/// Parses multiple HTML documents in parallel.
///
/// Uses Rayon's work-stealing scheduler for efficient parallel execution.
/// Each document is parsed independently, making this highly parallelizable.
/// Results are returned in the same order as the input.
///
/// # Performance
///
/// Expected speedup is near-linear with the number of CPU cores (e.g., 3.5-4x
/// on a 4-core system). For small documents or small batches, the overhead of
/// parallelization may outweigh the benefits.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "parallel")]
/// # fn example() {
/// use scrape_core::parallel::parse_batch;
///
/// let documents = vec![
///     "<html><body><div>Doc 1</div></body></html>",
///     "<html><body><div>Doc 2</div></body></html>",
///     "<html><body><div>Doc 3</div></body></html>",
/// ];
///
/// let soups = parse_batch(&documents);
/// assert_eq!(soups.len(), 3);
///
/// for soup in &soups {
///     let div = soup.find("div").unwrap().unwrap();
///     assert!(div.text().starts_with("Doc"));
/// }
/// # }
/// ```
#[must_use]
pub fn parse_batch(documents: &[&str]) -> Vec<Soup> {
    documents.par_iter().map(|html| Soup::parse(html)).collect()
}

/// Parses multiple HTML documents in parallel with custom configuration.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "parallel")]
/// # fn example() {
/// use scrape_core::{SoupConfig, parallel::parse_batch_with_config};
///
/// let config = SoupConfig::builder().max_depth(100).preserve_whitespace(true).build();
///
/// let documents = vec!["<div>  Whitespace  </div>", "<div>  Preserved  </div>"];
///
/// let soups = parse_batch_with_config(&documents, &config);
/// assert_eq!(soups.len(), 2);
/// # }
/// ```
#[must_use]
pub fn parse_batch_with_config(documents: &[&str], config: &SoupConfig) -> Vec<Soup> {
    documents.par_iter().map(|html| Soup::parse_with_config(html, config.clone())).collect()
}

/// Parses owned HTML strings in parallel.
///
/// This function works with owned strings without consuming them, avoiding
/// unnecessary cloning when you already have a vector of owned strings.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "parallel")]
/// # fn example() {
/// use scrape_core::parallel::parse_batch_owned;
///
/// let documents = vec![
///     String::from("<div>One</div>"),
///     String::from("<div>Two</div>"),
///     String::from("<div>Three</div>"),
/// ];
///
/// let soups = parse_batch_owned(&documents);
/// assert_eq!(soups.len(), 3);
/// # }
/// ```
#[must_use]
pub fn parse_batch_owned(documents: &[String]) -> Vec<Soup> {
    documents.par_iter().map(|html| Soup::parse(html)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_batch_basic() {
        let docs = vec!["<div>A</div>", "<div>B</div>", "<div>C</div>"];

        let results = parse_batch(&docs);
        assert_eq!(results.len(), 3);

        let texts: Vec<_> =
            results.iter().map(|soup| soup.find("div").unwrap().unwrap().text()).collect();

        assert_eq!(texts, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_parse_batch_order_preserved() {
        let docs: Vec<_> = (0..100).map(|i| format!("<div>{i}</div>")).collect();
        let refs: Vec<_> = docs.iter().map(String::as_str).collect();

        let results = parse_batch(&refs);
        assert_eq!(results.len(), 100);

        for (i, soup) in results.iter().enumerate() {
            let div = soup.find("div").unwrap().unwrap();
            assert_eq!(div.text(), i.to_string());
        }
    }

    #[test]
    fn test_parse_batch_with_config_preserves_whitespace() {
        let config = SoupConfig::builder().preserve_whitespace(true).build();

        let docs = vec!["<div>  Space  </div>", "<div>  More  </div>"];

        let results = parse_batch_with_config(&docs, &config);
        assert_eq!(results.len(), 2);

        for soup in &results {
            let text = soup.find("div").unwrap().unwrap().text();
            assert!(text.contains("  "));
        }
    }

    #[test]
    fn test_parse_batch_with_config_max_depth() {
        let config = SoupConfig::builder().max_depth(2).build();

        let docs = vec!["<div><div><div>Deep</div></div></div>", "<div><div>Shallow</div></div>"];

        let results = parse_batch_with_config(&docs, &config);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_parse_batch_owned() {
        let docs = vec![
            String::from("<div>One</div>"),
            String::from("<div>Two</div>"),
            String::from("<div>Three</div>"),
        ];

        let results = parse_batch_owned(&docs);
        assert_eq!(results.len(), 3);

        let texts: Vec<_> =
            results.iter().map(|soup| soup.find("div").unwrap().unwrap().text()).collect();

        assert_eq!(texts, vec!["One", "Two", "Three"]);
    }

    #[test]
    fn test_parse_batch_empty() {
        let docs: Vec<&str> = vec![];
        let results = parse_batch(&docs);
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_batch_single_document() {
        let docs = vec!["<div>Single</div>"];
        let results = parse_batch(&docs);
        assert_eq!(results.len(), 1);

        let text = results[0].find("div").unwrap().unwrap().text();
        assert_eq!(text, "Single");
    }

    #[test]
    fn test_parse_batch_complex_documents() {
        let docs = vec![
            r#"<html><head><title>Page 1</title></head><body><div id="content">Content 1</div></body></html>"#,
            r#"<html><head><title>Page 2</title></head><body><div id="content">Content 2</div></body></html>"#,
            r#"<html><head><title>Page 3</title></head><body><div id="content">Content 3</div></body></html>"#,
        ];

        let results = parse_batch(&docs);
        assert_eq!(results.len(), 3);

        for (i, soup) in results.iter().enumerate() {
            let title = soup.find("title").unwrap().unwrap().text();
            assert_eq!(title, format!("Page {}", i + 1));

            let content = soup.find("#content").unwrap().unwrap().text();
            assert_eq!(content, format!("Content {}", i + 1));
        }
    }

    #[test]
    fn test_parse_batch_large_batch() {
        let docs: Vec<_> = (0..1000).map(|i| format!("<div id='item-{i}'>{i}</div>")).collect();
        let refs: Vec<_> = docs.iter().map(String::as_str).collect();

        let results = parse_batch(&refs);
        assert_eq!(results.len(), 1000);

        for (i, soup) in results.iter().enumerate() {
            let div = soup.find("div").unwrap().unwrap();
            assert_eq!(div.text(), i.to_string());
            assert_eq!(div.get("id"), Some(format!("item-{i}").as_str()));
        }
    }

    #[test]
    fn test_parse_batch_with_malformed_html() {
        let docs = vec![
            "<div>Valid</div>",
            "<div>Unclosed",
            "<div><div><div>Deep</div></div></div>",
            "<!DOCTYPE html><html><body>Full</body></html>",
            "",
        ];

        let results = parse_batch(&docs);
        assert_eq!(results.len(), 5);

        assert!(results[0].find("div").unwrap().is_some());
        assert!(results[1].find("div").unwrap().is_some());
        assert!(results[2].find("div").unwrap().is_some());
        assert!(results[3].find("body").unwrap().is_some());
    }

    #[test]
    fn test_parse_batch_memory_stress() {
        let large_doc = format!("<div>{}</div>", "x".repeat(100_000));
        let docs: Vec<_> = (0..100).map(|_| large_doc.as_str()).collect();

        let results = parse_batch(&docs);
        assert_eq!(results.len(), 100);

        for soup in &results {
            let div = soup.find("div").unwrap().unwrap();
            assert_eq!(div.text().len(), 100_000);
        }
    }

    #[test]
    fn test_parse_batch_concurrent_calls() {
        use std::sync::Arc;
        use std::thread;

        let docs = Arc::new(vec!["<div>1</div>", "<div>2</div>", "<div>3</div>"]);

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let docs_clone = Arc::clone(&docs);
                thread::spawn(move || parse_batch(&docs_clone))
            })
            .collect();

        for handle in handles {
            let results = handle.join().unwrap();
            assert_eq!(results.len(), 3);
        }
    }

    #[test]
    fn test_parse_batch_unicode_content() {
        let docs = vec![
            "<div>Hello 世界</div>",
            "<div>Привет мир</div>",
            "<div>مرحبا بالعالم</div>",
            "<div>🌍🌎🌏</div>",
        ];

        let results = parse_batch(&docs);
        assert_eq!(results.len(), 4);

        assert_eq!(results[0].find("div").unwrap().unwrap().text(), "Hello 世界");
        assert_eq!(results[1].find("div").unwrap().unwrap().text(), "Привет мир");
        assert_eq!(results[2].find("div").unwrap().unwrap().text(), "مرحبا بالعالم");
        assert_eq!(results[3].find("div").unwrap().unwrap().text(), "🌍🌎🌏");
    }

    #[test]
    fn test_parse_batch_mixed_document_sizes() {
        let small = "<div>Small</div>";
        let medium = format!("<div>{}</div>", "Medium ".repeat(1000));
        let large = format!("<div>{}</div>", "Large ".repeat(10_000));
        let tiny = "<p>Tiny</p>";

        let docs = vec![small, medium.as_str(), large.as_str(), tiny];

        let results = parse_batch(&docs);
        assert_eq!(results.len(), 4);

        assert_eq!(results[0].find("div").unwrap().unwrap().text(), "Small");
        assert!(results[1].find("div").unwrap().unwrap().text().len() > 5000);
        assert!(results[2].find("div").unwrap().unwrap().text().len() > 50_000);
        assert_eq!(results[3].find("p").unwrap().unwrap().text(), "Tiny");
    }
}
