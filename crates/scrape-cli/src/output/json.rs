//! JSON output formatter.

use std::{
    collections::HashMap,
    io::{self, Write},
};

use serde_json::{Value, json};

use super::{Extraction, Output};

/// JSON output formatter.
pub struct JsonOutput {
    /// Whether to pretty-print the JSON.
    pub pretty: bool,
}

impl Output for JsonOutput {
    fn format_single(
        &self,
        writer: &mut dyn Write,
        results: &[Extraction],
        _filename: Option<&str>,
    ) -> io::Result<()> {
        let value: Value = results
            .iter()
            .map(|e| if e.attrs.is_some() || e.html.is_some() { json!(e) } else { json!(e.text) })
            .collect();

        let output = if self.pretty {
            serde_json::to_string_pretty(&value)
        } else {
            serde_json::to_string(&value)
        }
        .map_err(io::Error::other)?;

        writeln!(writer, "{output}")
    }

    fn format_named(
        &self,
        writer: &mut dyn Write,
        results: &HashMap<String, Vec<Extraction>>,
        _filename: Option<&str>,
    ) -> io::Result<()> {
        let value: Value = results
            .iter()
            .map(|(name, extractions)| {
                let texts: Vec<&str> = extractions.iter().map(|e| e.text.as_str()).collect();
                (name.clone(), json!(texts))
            })
            .collect();

        let output = if self.pretty {
            serde_json::to_string_pretty(&value)
        } else {
            serde_json::to_string(&value)
        }
        .map_err(io::Error::other)?;

        writeln!(writer, "{output}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_single_simple() {
        let output = JsonOutput { pretty: false };
        let results = vec![
            Extraction { text: "Hello".into(), attrs: None, html: None },
            Extraction { text: "World".into(), attrs: None, html: None },
        ];

        let mut buf = Vec::new();
        output.format_single(&mut buf, &results, None).unwrap();
        let json_str = String::from_utf8(buf).unwrap();
        assert!(json_str.contains("[\"Hello\",\"World\"]"));
    }

    #[test]
    fn test_format_single_with_attrs() {
        let output = JsonOutput { pretty: false };
        let mut attrs = HashMap::new();
        attrs.insert("href".into(), "/page".into());
        let results = vec![Extraction {
            text: "Link".into(),
            attrs: Some(attrs),
            html: Some("<a href=\"/page\">Link</a>".into()),
        }];

        let mut buf = Vec::new();
        output.format_single(&mut buf, &results, None).unwrap();
        let json_str = String::from_utf8(buf).unwrap();
        assert!(json_str.contains("\"text\":\"Link\""));
        assert!(json_str.contains("\"attrs\""));
        assert!(json_str.contains("\"html\""));
    }

    #[test]
    fn test_format_single_pretty() {
        let output = JsonOutput { pretty: true };
        let results = vec![Extraction { text: "Hello".into(), attrs: None, html: None }];

        let mut buf = Vec::new();
        output.format_single(&mut buf, &results, None).unwrap();
        let json_str = String::from_utf8(buf).unwrap();
        assert!(json_str.contains('\n'));
    }

    #[test]
    fn test_format_named() {
        let output = JsonOutput { pretty: false };
        let mut results = HashMap::new();
        results.insert(
            "title".into(),
            vec![Extraction { text: "Hello".into(), attrs: None, html: None }],
        );
        results.insert(
            "links".into(),
            vec![
                Extraction { text: "A".into(), attrs: None, html: None },
                Extraction { text: "B".into(), attrs: None, html: None },
            ],
        );

        let mut buf = Vec::new();
        output.format_named(&mut buf, &results, None).unwrap();
        let json_str = String::from_utf8(buf).unwrap();
        assert!(json_str.contains("\"title\":[\"Hello\"]"));
        assert!(json_str.contains("\"links\":[\"A\",\"B\"]"));
    }
}
