//! CLI argument parsing using clap derive.

use std::path::PathBuf;

use clap::{Parser, ValueEnum};

/// High-performance HTML extraction tool.
///
/// Extract data from HTML using CSS selectors. Supports multiple output
/// formats and parallel processing of multiple files.
#[derive(Parser, Debug)]
#[command(name = "scrape")]
#[command(author, version, about, long_about = None)]
#[command(after_help = "EXAMPLES:
    scrape 'h1' page.html              Extract h1 text
    scrape -o json 'a[href]' page.html Extract links as JSON
    scrape -a href 'a' page.html       Extract href attributes
    curl url | scrape 'title'          Extract from stdin
    scrape -s title='h1' -s links='a' page.html
")]
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
    /// CSS selector for extraction.
    ///
    /// If not provided, --select must be used instead.
    #[arg(value_name = "SELECTOR")]
    pub selector: Option<String>,

    /// Input HTML files.
    ///
    /// If not provided, reads from stdin.
    #[arg(value_name = "FILES")]
    pub files: Vec<PathBuf>,

    /// Named selector extraction (can be repeated).
    ///
    /// Format: NAME=SELECTOR
    /// Example: -s title='h1' -s links='a[href]'
    #[arg(short = 's', long = "select", value_name = "NAME=SELECTOR")]
    pub selects: Vec<String>,

    /// Output format.
    #[arg(short = 'o', long, value_enum, default_value_t = OutputFormat::Text)]
    pub output: OutputFormat,

    /// Extract attribute value instead of text content.
    #[arg(short = 'a', long = "attribute", value_name = "ATTR")]
    pub attribute: Option<String>,

    /// Return only the first match.
    #[arg(short = '1', long)]
    pub first: bool,

    /// Colorize output.
    #[arg(short = 'c', long, value_enum, default_value_t = ColorMode::Auto)]
    pub color: ColorMode,

    /// Pretty-print JSON output.
    #[arg(short = 'p', long)]
    pub pretty: bool,

    /// Use NUL as line delimiter (for xargs -0).
    #[arg(short = '0', long)]
    pub null: bool,

    /// Suppress error messages.
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Number of parallel threads for batch processing.
    #[arg(short = 'j', long, value_name = "N")]
    pub parallel: Option<usize>,

    /// Include filename prefix in output for multiple files.
    #[arg(short = 'H', long = "with-filename")]
    pub with_filename: bool,

    /// Suppress filename prefix in output.
    #[arg(long = "no-filename")]
    pub no_filename: bool,

    /// Fetch HTML from URL instead of file.
    #[cfg(feature = "url")]
    #[arg(short = 'u', long = "url", value_name = "URL")]
    pub url: Option<String>,

    /// Start interactive REPL mode.
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,

    /// Explain selector (show specificity and hints).
    #[arg(long = "explain")]
    pub explain: bool,

    /// Request timeout in seconds (for URL fetch).
    #[cfg(feature = "url")]
    #[arg(long = "timeout", default_value = "30", value_name = "SECONDS")]
    pub timeout: u64,
}

/// Output format for extraction results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Plain text (one result per line)
    Text,
    /// JSON array or object
    Json,
    /// HTML fragments
    Html,
    /// CSV format (for named selectors)
    Csv,
}

/// Color mode for terminal output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ColorMode {
    /// Auto-detect based on terminal
    Auto,
    /// Always colorize
    Always,
    /// Never colorize
    Never,
}

impl Args {
    /// Parse and validate arguments.
    ///
    /// # Errors
    ///
    /// Returns an error if arguments are invalid or conflicting.
    pub fn parse_and_validate() -> Result<Self, String> {
        let args = Self::parse();

        // Interactive and explain modes don't need selectors
        if args.interactive || args.explain {
            return Ok(args);
        }

        if args.selector.is_none() && args.selects.is_empty() {
            return Err("Either <SELECTOR> or --select must be provided".into());
        }

        if args.selector.is_some() && !args.selects.is_empty() {
            return Err("Cannot use both <SELECTOR> and --select".into());
        }

        if args.output == OutputFormat::Csv && args.selects.is_empty() {
            return Err("CSV output requires --select for column names".into());
        }

        for select in &args.selects {
            if !select.contains('=') {
                return Err(format!("Invalid --select format: {select}. Use NAME=SELECTOR"));
            }
        }

        Ok(args)
    }

    /// Parse named selectors into (name, selector) pairs.
    #[must_use]
    pub fn parse_selects(&self) -> Vec<(String, String)> {
        self.selects
            .iter()
            .filter_map(|s| {
                let (name, selector) = s.split_once('=')?;
                Some((name.to_string(), selector.to_string()))
            })
            .collect()
    }

    /// Determine if filenames should be shown.
    #[must_use]
    pub fn show_filename(&self) -> bool {
        if self.with_filename {
            return true;
        }
        if self.no_filename {
            return false;
        }
        self.files.len() > 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_selects() {
        let args = Args {
            selector: None,
            files: vec![],
            selects: vec!["title=h1".into(), "links=a[href]".into()],
            output: OutputFormat::Text,
            attribute: None,
            first: false,
            color: ColorMode::Auto,
            pretty: false,
            null: false,
            quiet: false,
            parallel: None,
            with_filename: false,
            no_filename: false,
            #[cfg(feature = "url")]
            url: None,
            interactive: false,
            explain: false,
            #[cfg(feature = "url")]
            timeout: 30,
        };

        let selects = args.parse_selects();
        assert_eq!(selects.len(), 2);
        assert_eq!(selects[0], ("title".into(), "h1".into()));
        assert_eq!(selects[1], ("links".into(), "a[href]".into()));
    }

    #[test]
    fn test_show_filename_explicit() {
        let mut args = Args {
            selector: Some("h1".into()),
            files: vec![],
            selects: vec![],
            output: OutputFormat::Text,
            attribute: None,
            first: false,
            color: ColorMode::Auto,
            pretty: false,
            null: false,
            quiet: false,
            parallel: None,
            with_filename: true,
            no_filename: false,
            #[cfg(feature = "url")]
            url: None,
            interactive: false,
            explain: false,
            #[cfg(feature = "url")]
            timeout: 30,
        };

        assert!(args.show_filename());

        args.with_filename = false;
        args.no_filename = true;
        assert!(!args.show_filename());
    }

    #[test]
    fn test_show_filename_auto() {
        let mut args = Args {
            selector: Some("h1".into()),
            files: vec!["a.html".into()],
            selects: vec![],
            output: OutputFormat::Text,
            attribute: None,
            first: false,
            color: ColorMode::Auto,
            pretty: false,
            null: false,
            quiet: false,
            parallel: None,
            with_filename: false,
            no_filename: false,
            #[cfg(feature = "url")]
            url: None,
            interactive: false,
            explain: false,
            #[cfg(feature = "url")]
            timeout: 30,
        };

        assert!(!args.show_filename());

        args.files.push("b.html".into());
        assert!(args.show_filename());
    }
}
