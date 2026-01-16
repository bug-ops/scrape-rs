//! CSV output formatter.

use std::{
    collections::HashMap,
    io::{self, Write},
};

use super::{Extraction, Output};

/// CSV output formatter.
pub struct CsvOutput;

impl Output for CsvOutput {
    fn format_single(
        &self,
        writer: &mut dyn Write,
        results: &[Extraction],
        filename: Option<&str>,
    ) -> io::Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        if filename.is_some() {
            wtr.write_record(["file", "value"])?;
        } else {
            wtr.write_record(["value"])?;
        }

        for result in results {
            if let Some(fname) = filename {
                wtr.write_record([fname, &result.text])?;
            } else {
                wtr.write_record([&result.text])?;
            }
        }

        wtr.flush()?;
        Ok(())
    }

    fn format_named(
        &self,
        writer: &mut dyn Write,
        results: &HashMap<String, Vec<Extraction>>,
        _filename: Option<&str>,
    ) -> io::Result<()> {
        let mut wtr = csv::Writer::from_writer(writer);

        let mut headers: Vec<_> = results.keys().cloned().collect();
        headers.sort();

        wtr.write_record(&headers)?;

        let max_rows = results.values().map(Vec::len).max().unwrap_or(0);

        for row_idx in 0..max_rows {
            let row: Vec<_> = headers
                .iter()
                .map(|h| {
                    results
                        .get(h)
                        .and_then(|v| v.get(row_idx))
                        .map_or("", |e| e.text.as_str())
                })
                .collect();
            wtr.write_record(&row)?;
        }

        wtr.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_single() {
        let output = CsvOutput;
        let results = vec![
            Extraction { text: "Hello".into(), attrs: None, html: None },
            Extraction { text: "World".into(), attrs: None, html: None },
        ];

        let mut buf = Vec::new();
        output.format_single(&mut buf, &results, None).unwrap();
        let csv_str = String::from_utf8(buf).unwrap();
        assert!(csv_str.contains("value"));
        assert!(csv_str.contains("Hello"));
        assert!(csv_str.contains("World"));
    }

    #[test]
    fn test_format_single_with_filename() {
        let output = CsvOutput;
        let results = vec![Extraction { text: "Hello".into(), attrs: None, html: None }];

        let mut buf = Vec::new();
        output.format_single(&mut buf, &results, Some("test.html")).unwrap();
        let csv_str = String::from_utf8(buf).unwrap();
        assert!(csv_str.contains("file,value"));
        assert!(csv_str.contains("test.html,Hello"));
    }

    #[test]
    fn test_format_named() {
        let output = CsvOutput;
        let mut results = HashMap::new();
        results.insert(
            "name".into(),
            vec![
                Extraction { text: "Alice".into(), attrs: None, html: None },
                Extraction { text: "Bob".into(), attrs: None, html: None },
            ],
        );
        results.insert(
            "age".into(),
            vec![
                Extraction { text: "30".into(), attrs: None, html: None },
                Extraction { text: "25".into(), attrs: None, html: None },
            ],
        );

        let mut buf = Vec::new();
        output.format_named(&mut buf, &results, None).unwrap();
        let csv_str = String::from_utf8(buf).unwrap();
        assert!(csv_str.contains("age,name"));
        assert!(csv_str.contains("30,Alice"));
        assert!(csv_str.contains("25,Bob"));
    }

    #[test]
    fn test_format_named_uneven_columns() {
        let output = CsvOutput;
        let mut results = HashMap::new();
        results.insert(
            "name".into(),
            vec![
                Extraction { text: "Alice".into(), attrs: None, html: None },
                Extraction { text: "Bob".into(), attrs: None, html: None },
            ],
        );
        results
            .insert("age".into(), vec![Extraction { text: "30".into(), attrs: None, html: None }]);

        let mut buf = Vec::new();
        output.format_named(&mut buf, &results, None).unwrap();
        let csv_str = String::from_utf8(buf).unwrap();
        assert!(csv_str.contains(",Bob"));
    }
}
