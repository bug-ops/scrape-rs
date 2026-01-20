//! Streaming HTML parsing and rewriting.
//!
//! This module provides streaming HTML parsing capabilities that allow processing
//! large documents with constant memory usage. Unlike the DOM-based parser which
//! builds the entire document tree in memory, the streaming parser processes HTML
//! incrementally using callbacks.
//!
//! # Features
//!
//! - **Constant memory usage**: Process GB-scale documents without loading everything into RAM
//! - **Callback-based**: Register handlers for elements, text, and end tags
//! - **HTML rewriting**: Modify HTML content on-the-fly during streaming
//! - **Typestate safety**: Compile-time enforcement of valid state transitions
//!
//! # Example
//!
//! ```ignore
//! use scrape_core::{StreamingSoup, StreamingConfig};
//!
//! let mut streaming = StreamingSoup::with_config(
//!     StreamingConfig::default().buffer_size(8192)
//! );
//!
//! streaming.on_element("a[href]", |el| {
//!     if let Some(href) = el.get_attribute("href") {
//!         println!("Found link: {}", href);
//!     }
//!     Ok(())
//! })?;
//!
//! let mut processor = streaming.start();
//! processor.write(b"<html><body><a href='example.com'>Link</a></body></html>")?;
//! let finished = processor.end()?;
//! ```

pub mod config;
pub mod element;
pub(crate) mod handlers;
pub mod parser;
pub mod rewriter;

pub use config::{RewriterConfig, StreamingConfig};
pub use element::{ContentType, StreamingElement};
pub use parser::{StreamingSoup, StreamingStats, state};
pub use rewriter::HtmlRewriter;
