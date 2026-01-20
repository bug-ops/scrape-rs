//! Interactive REPL mode for the CLI.

use std::io::{self, BufRead, Write};

use scrape_core::{Soup, query::explain};

/// REPL state.
pub struct Repl {
    soup: Option<Soup>,
    source: Option<String>,
    history: Vec<String>,
}

impl Repl {
    /// Creates a new REPL instance.
    #[must_use]
    pub fn new() -> Self {
        Self { soup: None, source: None, history: Vec::new() }
    }

    /// Loads HTML into the REPL.
    pub fn load(&mut self, html: &str) {
        self.soup = Some(Soup::parse(html));
        self.source = Some(html.to_string());
        println!("Loaded {} bytes of HTML", html.len());
    }

    /// Runs the interactive REPL loop.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if reading from stdin fails.
    pub fn run(&mut self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        println!("scrape-rs interactive mode");
        println!("Commands: :load <file>, :url <url>, :explain <selector>, :history, :help, :quit");
        println!();

        loop {
            print!("> ");
            stdout.flush()?;

            let mut line = String::new();
            if stdin.lock().read_line(&mut line)? == 0 {
                break;
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            self.history.push(line.to_string());

            if line.starts_with(':') {
                if !self.handle_command(line) {
                    break;
                }
            } else {
                self.execute_selector(line);
            }
        }

        Ok(())
    }

    fn handle_command(&mut self, line: &str) -> bool {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        let cmd = parts[0];
        let arg = parts.get(1).copied().unwrap_or("");

        match cmd {
            ":quit" | ":q" => return false,
            ":help" | ":h" => self.print_help(),
            ":history" => self.print_history(),
            ":load" => self.cmd_load(arg),
            ":url" => self.cmd_url(arg),
            ":explain" => self.cmd_explain(arg),
            ":count" => self.cmd_count(arg),
            ":tree" => self.cmd_tree(),
            _ => println!("Unknown command: {cmd}. Type :help for available commands."),
        }
        true
    }

    fn print_help(&self) {
        println!("Commands:");
        println!("  :load <file>      Load HTML from file");
        println!("  :url <url>        Fetch and load HTML from URL");
        println!("  :explain <sel>    Explain a CSS selector");
        println!("  :count <sel>      Count matches for selector");
        println!("  :tree             Show DOM tree structure");
        println!("  :history          Show command history");
        println!("  :help, :h         Show this help");
        println!("  :quit, :q         Exit");
        println!();
        println!("Or enter a CSS selector directly to find matching elements.");
    }

    fn print_history(&self) {
        for (i, cmd) in self.history.iter().enumerate() {
            println!("{:4}: {}", i + 1, cmd);
        }
    }

    fn cmd_load(&mut self, path: &str) {
        if path.is_empty() {
            println!("Usage: :load <file>");
            return;
        }
        match std::fs::read_to_string(path) {
            Ok(html) => self.load(&html),
            Err(e) => println!("Error loading file: {e}"),
        }
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    fn cmd_url(&mut self, url: &str) {
        if url.is_empty() {
            println!("Usage: :url <url>");
            return;
        }
        #[cfg(feature = "url")]
        {
            use super::fetch::{FetchConfig, fetch_url};
            match fetch_url(url, &FetchConfig::default()) {
                Ok(html) => self.load(&html),
                Err(e) => println!("Error fetching URL: {e}"),
            }
        }
        #[cfg(not(feature = "url"))]
        {
            let _ = url; // Suppress unused variable warning
            println!("URL support not available. Compile with --features url");
        }
    }

    fn cmd_explain(&self, selector: &str) {
        if selector.is_empty() {
            println!("Usage: :explain <selector>");
            return;
        }
        match explain(selector) {
            Ok(explanation) => println!("{}", explanation.format()),
            Err(e) => println!("Error: {e}"),
        }
    }

    fn cmd_count(&self, selector: &str) {
        let Some(soup) = &self.soup else {
            println!("No HTML loaded. Use :load or :url first.");
            return;
        };
        if selector.is_empty() {
            println!("Usage: :count <selector>");
            return;
        }
        match soup.find_all(selector) {
            Ok(tags) => println!("{} matches", tags.len()),
            Err(e) => println!("Error: {e}"),
        }
    }

    fn cmd_tree(&self) {
        let Some(_soup) = &self.soup else {
            println!("No HTML loaded. Use :load or :url first.");
            return;
        };
        // Simplified DOM tree display
        println!("DOM tree (feature not yet fully implemented):");
        println!("Use CSS selectors to explore the structure instead.");
    }

    fn execute_selector(&self, selector: &str) {
        let Some(soup) = &self.soup else {
            println!("No HTML loaded. Use :load or :url first.");
            return;
        };

        match soup.find_all(selector) {
            Ok(tags) => {
                if tags.is_empty() {
                    println!("No matches found.");
                } else {
                    for (i, tag) in tags.iter().enumerate().take(10) {
                        let name = tag.name().unwrap_or("?");
                        println!("[{}] <{}> {}", i + 1, name, truncate(tag.text(), 60));
                    }
                    if tags.len() > 10 {
                        println!("... and {} more", tags.len() - 10);
                    }
                }
            }
            Err(e) => println!("Error: {e}"),
        }
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

fn truncate(s: String, max_len: usize) -> String {
    if s.len() <= max_len {
        s
    } else {
        let truncate_at = max_len.saturating_sub(3);
        // Find valid UTF-8 boundary
        let mut boundary = truncate_at;
        while boundary > 0 && !s.is_char_boundary(boundary) {
            boundary -= 1;
        }
        format!("{}...", &s[..boundary])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_creation() {
        let repl = Repl::new();
        assert!(repl.soup.is_none());
        assert!(repl.source.is_none());
        assert_eq!(repl.history.len(), 0);
    }

    #[test]
    fn test_repl_load() {
        let mut repl = Repl::new();
        repl.load("<div>test</div>");
        assert!(repl.soup.is_some());
        assert!(repl.source.is_some());
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short".into(), 10), "short");
        assert_eq!(truncate("this is a very long string".into(), 10), "this is...");
    }

    #[test]
    fn test_truncate_edge_cases() {
        assert_eq!(truncate(String::new(), 10), "");
        assert_eq!(truncate("exact".into(), 5), "exact");
        assert_eq!(truncate("one more".into(), 8), "one more");
        assert_eq!(truncate("needs truncation".into(), 8), "needs...");
    }

    #[test]
    fn test_truncate_with_unicode() {
        assert_eq!(truncate("🚀 emoji".into(), 20), "🚀 emoji");

        // Test with long unicode string - note that truncate uses byte indexing
        // which can panic on multi-byte chars, but for typical usage it's acceptable
        let long_text = "Unicode: 你好世界 ".repeat(10);
        let truncated = truncate(long_text, 30);
        assert!(truncated.ends_with("...") || truncated.len() <= 30);
    }

    #[test]
    fn test_repl_load_updates_state() {
        let mut repl = Repl::new();
        let html = "<html><body><p>Test</p></body></html>";
        repl.load(html);

        assert!(repl.soup.is_some());
        assert!(repl.source.is_some());
        assert_eq!(repl.source.as_ref().unwrap(), html);
    }

    #[test]
    fn test_repl_load_empty_html() {
        let mut repl = Repl::new();
        repl.load("");
        assert!(repl.soup.is_some());
        assert_eq!(repl.source.as_ref().unwrap(), "");
    }

    #[test]
    fn test_repl_load_malformed_html() {
        let mut repl = Repl::new();
        let malformed = "<div><p>Unclosed tags<span>";
        repl.load(malformed);
        assert!(repl.soup.is_some());
    }

    #[test]
    fn test_repl_default_trait() {
        let repl = Repl::default();
        assert!(repl.soup.is_none());
        assert_eq!(repl.history.len(), 0);
    }

    #[test]
    fn test_repl_multiple_loads() {
        let mut repl = Repl::new();
        repl.load("<div>First</div>");
        let first_source = repl.source.clone();

        repl.load("<span>Second</span>");
        let second_source = repl.source.clone();

        assert_ne!(first_source, second_source);
        assert_eq!(second_source.as_ref().unwrap(), "<span>Second</span>");
    }

    #[test]
    fn test_repl_load_large_document() {
        let mut repl = Repl::new();
        let large_doc = format!("<html>{}</html>", "<div>content</div>".repeat(10000));
        repl.load(&large_doc);
        assert!(repl.soup.is_some());
    }
}
