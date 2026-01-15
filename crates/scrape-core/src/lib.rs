//! # scrape-core
//!
//! High-performance HTML parsing library with CSS selector support.
//!
//! This crate provides the core functionality for parsing HTML documents
//! and querying them using CSS selectors. It is designed to be fast,
//! memory-efficient, and spec-compliant.
//!
//! ## Quick Start
//!
//! ```rust
//! use scrape_core::{Html5everParser, Parser, Soup, SoupConfig};
//!
//! // Parse HTML using Soup (high-level API)
//! let html = "<html><body><div class=\"product\">Hello</div></body></html>";
//! let soup = Soup::parse(html);
//!
//! // Or use the parser directly (low-level API)
//! let parser = Html5everParser;
//! let document = parser.parse(html).unwrap();
//! assert!(document.root().is_some());
//! ```
//!
//! ## Features
//!
//! - **Fast parsing**: Built on `html5ever` for spec-compliant HTML5 parsing
//! - **CSS selectors**: Full CSS selector support via the `selectors` crate
//! - **Memory efficient**: Arena-based allocation for DOM nodes
//! - **SIMD acceleration**: Optional SIMD support for faster byte scanning

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

mod dom;
mod error;
mod parser;
mod query;
mod soup;
mod tag;

// Error types
// DOM types
pub use dom::{Document, Node, NodeId, NodeKind};
pub use error::{Error, Result};
// Parser types
pub use parser::{Html5everParser, ParseConfig, ParseError, ParseResult, Parser};
// High-level API
pub use soup::{Soup, SoupConfig};
pub use tag::Tag;
