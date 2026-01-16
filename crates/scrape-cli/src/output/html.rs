//! HTML fragment output formatter.

use std::{
    collections::HashMap,
    io::{self, Write},
};

use super::{Extraction, Output};

/// HTML fragment output formatter.
pub struct HtmlOutput {
    /// Line delimiter.
    pub delimiter: u8,
}

impl Output for HtmlOutput {
    fn format_single(
        &self,
        writer: &mut dyn Write,
        results: &[Extraction],
        filename: Option<&str>,
    ) -> io::Result<()> {
        for result in results {
            if let Some(name) = filename {
                let safe_name = name.replace("--", "- -");
                write!(writer, "<!-- {safe_name} -->")?;
                writer.write_all(&[self.delimiter])?;
            }
            if let Some(ref html) = result.html {
                writer.write_all(html.as_bytes())?;
            } else {
                writer.write_all(result.text.as_bytes())?;
            }
            writer.write_all(&[self.delimiter])?;
        }
        Ok(())
    }

    fn format_named(
        &self,
        writer: &mut dyn Write,
        results: &HashMap<String, Vec<Extraction>>,
        filename: Option<&str>,
    ) -> io::Result<()> {
        if let Some(fname) = filename {
            let safe_fname = fname.replace("--", "- -");
            writeln!(writer, "<!-- {safe_fname} -->")?;
        }

        let mut keys: Vec<_> = results.keys().collect();
        keys.sort();

        for name in keys {
            let extractions = &results[name];
            let safe_name = name.replace("--", "- -");
            writeln!(writer, "<!-- {safe_name} -->")?;
            for extraction in extractions {
                if let Some(ref html) = extraction.html {
                    writer.write_all(html.as_bytes())?;
                } else {
                    writer.write_all(extraction.text.as_bytes())?;
                }
                writer.write_all(&[self.delimiter])?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_single_text() {
        let output = HtmlOutput { delimiter: b'\n' };
        let results = vec![Extraction { text: "Hello".into(), attrs: None, html: None }];

        let mut buf = Vec::new();
        output.format_single(&mut buf, &results, None).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "Hello\n");
    }

    #[test]
    fn test_format_single_with_html() {
        let output = HtmlOutput { delimiter: b'\n' };
        let results = vec![Extraction {
            text: "Hello".into(),
            attrs: None,
            html: Some("<span>Hello</span>".into()),
        }];

        let mut buf = Vec::new();
        output.format_single(&mut buf, &results, None).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "<span>Hello</span>\n");
    }

    #[test]
    fn test_format_single_with_filename() {
        let output = HtmlOutput { delimiter: b'\n' };
        let results = vec![Extraction { text: "Hello".into(), attrs: None, html: None }];

        let mut buf = Vec::new();
        output.format_single(&mut buf, &results, Some("test.html")).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert!(result.contains("<!-- test.html -->"));
        assert!(result.contains("Hello"));
    }

    #[test]
    fn test_format_named() {
        let output = HtmlOutput { delimiter: b'\n' };
        let mut results = HashMap::new();
        results.insert(
            "title".into(),
            vec![Extraction { text: "Hello".into(), attrs: None, html: None }],
        );

        let mut buf = Vec::new();
        output.format_named(&mut buf, &results, None).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert!(result.contains("<!-- title -->"));
        assert!(result.contains("Hello"));
    }
}
