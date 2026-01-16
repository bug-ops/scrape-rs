//! Output formatting for extraction results.

mod csv;
mod html;
mod json;
mod text;

use std::{
    collections::HashMap,
    io::{self, Write},
};

pub use self::{csv::CsvOutput, html::HtmlOutput, json::JsonOutput, text::TextOutput};
use crate::extract::Extraction;

/// Trait for output formatters.
///
/// Uses `&mut dyn Write` instead of generics to allow dynamic dispatch.
pub trait Output {
    /// Format single selector results.
    ///
    /// # Errors
    ///
    /// Returns an IO error if writing fails.
    fn format_single(
        &self,
        writer: &mut dyn Write,
        results: &[Extraction],
        filename: Option<&str>,
    ) -> io::Result<()>;

    /// Format named selector results.
    ///
    /// # Errors
    ///
    /// Returns an IO error if writing fails.
    fn format_named(
        &self,
        writer: &mut dyn Write,
        results: &HashMap<String, Vec<Extraction>>,
        filename: Option<&str>,
    ) -> io::Result<()>;
}
