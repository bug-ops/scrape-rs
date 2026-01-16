//! Plain text output formatter.

use std::{
    collections::HashMap,
    io::{self, Write},
};

use super::{Extraction, Output};

/// Plain text output formatter.
pub struct TextOutput {
    /// Line delimiter (newline or NUL).
    pub delimiter: u8,
    /// Whether to colorize output.
    pub color: bool,
}

impl Output for TextOutput {
    fn format_single(
        &self,
        writer: &mut dyn Write,
        results: &[Extraction],
        filename: Option<&str>,
    ) -> io::Result<()> {
        for result in results {
            if let Some(name) = filename {
                if self.color {
                    write!(writer, "\x1b[35m{name}\x1b[0m: ")?;
                } else {
                    write!(writer, "{name}: ")?;
                }
            }
            writer.write_all(result.text.as_bytes())?;
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
        let mut keys: Vec<_> = results.keys().collect();
        keys.sort();

        for name in keys {
            let extractions = &results[name];
            for extraction in extractions {
                if let Some(fname) = filename {
                    if self.color {
                        write!(writer, "\x1b[35m{fname}\x1b[0m: ")?;
                    } else {
                        write!(writer, "{fname}: ")?;
                    }
                }
                if self.color {
                    writeln!(writer, "\x1b[36m{name}\x1b[0m: {}", extraction.text)?;
                } else {
                    writeln!(writer, "{name}: {}", extraction.text)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_single() {
        let output = TextOutput { delimiter: b'\n', color: false };
        let results = vec![
            Extraction { text: "Hello".into(), attrs: None, html: None },
            Extraction { text: "World".into(), attrs: None, html: None },
        ];

        let mut buf = Vec::new();
        output.format_single(&mut buf, &results, None).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "Hello\nWorld\n");
    }

    #[test]
    fn test_format_single_with_filename() {
        let output = TextOutput { delimiter: b'\n', color: false };
        let results = vec![Extraction { text: "Hello".into(), attrs: None, html: None }];

        let mut buf = Vec::new();
        output.format_single(&mut buf, &results, Some("test.html")).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "test.html: Hello\n");
    }

    #[test]
    fn test_format_single_null_delimiter() {
        let output = TextOutput { delimiter: b'\0', color: false };
        let results = vec![
            Extraction { text: "A".into(), attrs: None, html: None },
            Extraction { text: "B".into(), attrs: None, html: None },
        ];

        let mut buf = Vec::new();
        output.format_single(&mut buf, &results, None).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "A\0B\0");
    }

    #[test]
    fn test_format_named() {
        let output = TextOutput { delimiter: b'\n', color: false };
        let mut results = HashMap::new();
        results.insert(
            "title".into(),
            vec![Extraction { text: "Hello".into(), attrs: None, html: None }],
        );

        let mut buf = Vec::new();
        output.format_named(&mut buf, &results, None).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "title: Hello\n");
    }
}
