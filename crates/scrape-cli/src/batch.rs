//! Parallel batch file processing.

use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::Result;
use rayon::prelude::*;

use crate::extract::{Extraction, extract, extract_named};

/// Result of processing a single file.
pub struct FileResult {
    /// The filename that was processed.
    pub filename: String,
    /// The extraction result or error.
    pub result: Result<Vec<Extraction>>,
}

/// Result of processing a single file with named selectors.
pub struct FileNamedResult {
    /// The filename that was processed.
    pub filename: String,
    /// The extraction results or error.
    pub result: Result<HashMap<String, Vec<Extraction>>>,
}

/// Process multiple files in parallel with a single selector.
pub fn process_files(
    files: &[PathBuf],
    selector: &str,
    attribute: Option<&str>,
    first_only: bool,
    threads: Option<usize>,
) -> Vec<FileResult> {
    if let Some(n) = threads {
        rayon::ThreadPoolBuilder::new().num_threads(n).build_global().ok();
    }

    files
        .par_iter()
        .map(|path| {
            let filename = path.display().to_string();

            let result = fs::read_to_string(path)
                .map_err(anyhow::Error::from)
                .and_then(|html| extract(&html, selector, attribute, first_only, false));

            FileResult { filename, result }
        })
        .collect()
}

/// Process multiple files in parallel with named selectors.
pub fn process_files_named(
    files: &[PathBuf],
    selectors: &[(String, String)],
    attribute: Option<&str>,
    first_only: bool,
    threads: Option<usize>,
) -> Vec<FileNamedResult> {
    if let Some(n) = threads {
        rayon::ThreadPoolBuilder::new().num_threads(n).build_global().ok();
    }

    files
        .par_iter()
        .map(|path| {
            let filename = path.display().to_string();

            let result = fs::read_to_string(path)
                .map_err(anyhow::Error::from)
                .and_then(|html| extract_named(&html, selectors, attribute, first_only));

            FileNamedResult { filename, result }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_process_files() {
        let dir = TempDir::new().unwrap();
        let path_a = dir.path().join("a.html");
        let path_b = dir.path().join("b.html");

        let mut f1 = File::create(&path_a).unwrap();
        writeln!(f1, "<h1>File A</h1>").unwrap();

        let mut f2 = File::create(&path_b).unwrap();
        writeln!(f2, "<h1>File B</h1>").unwrap();

        let files = vec![path_a, path_b];
        let results = process_files(&files, "h1", None, false, None);

        assert_eq!(results.len(), 2);

        for result in results {
            let extractions = result.result.unwrap();
            assert_eq!(extractions.len(), 1);
            assert!(
                extractions[0].text == "File A" || extractions[0].text == "File B",
                "Unexpected text: {}",
                extractions[0].text
            );
        }
    }

    #[test]
    fn test_process_files_with_error() {
        let files = vec![PathBuf::from("/nonexistent/file.html")];
        let results = process_files(&files, "h1", None, false, None);

        assert_eq!(results.len(), 1);
        assert!(results[0].result.is_err());
    }

    #[test]
    fn test_process_files_named() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.html");

        let mut f = File::create(&file).unwrap();
        writeln!(f, "<h1>Title</h1><a href=\"/\">Link</a>").unwrap();

        let files = vec![file];
        let selectors = vec![("title".into(), "h1".into()), ("link".into(), "a".into())];
        let results = process_files_named(&files, &selectors, None, false, None);

        assert_eq!(results.len(), 1);

        let extractions = results[0].result.as_ref().unwrap();
        assert_eq!(extractions["title"][0].text, "Title");
        assert_eq!(extractions["link"][0].text, "Link");
    }

    #[test]
    fn test_process_files_with_threads() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.html");

        let mut f = File::create(&file).unwrap();
        writeln!(f, "<h1>Test</h1>").unwrap();

        let files = vec![file];
        let results = process_files(&files, "h1", None, false, Some(2));

        assert_eq!(results.len(), 1);
        assert!(results[0].result.is_ok());
    }
}
