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
//! // Find elements using CSS selectors
//! if let Ok(Some(div)) = soup.find("div.product") {
//!     assert_eq!(div.text(), "Hello");
//! }
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
//!
//! ## CSS Selector Support
//!
//! The query engine supports most CSS3 selectors:
//!
//! ```rust
//! use scrape_core::Soup;
//!
//! let html = r#"
//!     <div class="container">
//!         <ul id="list">
//!             <li class="item active">One</li>
//!             <li class="item">Two</li>
//!             <li class="item">Three</li>
//!         </ul>
//!     </div>
//! "#;
//! let soup = Soup::parse(html);
//!
//! // Type selector
//! let divs = soup.find_all("div").unwrap();
//!
//! // Class selector
//! let items = soup.find_all(".item").unwrap();
//!
//! // ID selector
//! let list = soup.find("#list").unwrap();
//!
//! // Compound selector
//! let active = soup.find("li.item.active").unwrap();
//!
//! // Descendant combinator
//! let nested = soup.find_all("div li").unwrap();
//!
//! // Child combinator
//! let direct = soup.find_all("ul > li").unwrap();
//!
//! // Attribute selectors
//! let with_id = soup.find_all("[id]").unwrap();
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

mod dom;
mod error;
#[cfg(feature = "parallel")]
pub mod parallel;
mod parser;
pub mod query;
pub mod serialize;
#[cfg(feature = "simd")]
pub mod simd;
mod soup;
#[cfg(feature = "streaming")]
pub mod streaming;
mod tag;
pub mod utils;

// Error types
// DOM types
pub use dom::{
    AncestorsIter, Building, ChildrenIter, CommentMarker, DescendantsIter, Document, DocumentImpl,
    DocumentIndex, DocumentState, ElementAncestorsIter, ElementChildrenIter,
    ElementDescendantsIter, ElementMarker, ElementNextSiblingsIter, ElementPrevSiblingsIter,
    ElementSiblingsIter, MutableState, NextSiblingsIter, Node, NodeId, NodeKind, NodeType,
    PrevSiblingsIter, Queryable, QueryableState, Sealed, SiblingsIter, TagId, TextMarker,
};
pub use error::{Error, Result};
// Parser types
pub use parser::{Html5everParser, ParseConfig, ParseError, ParseResult, Parser};
// Query types
pub use query::{
    CompiledSelector, Filter, QueryError, QueryResult, TextNodesIter, compile_selector,
};
// Serialization utilities
pub use serialize::{HtmlSerializer, collect_text, serialize_inner_html, serialize_node};
// High-level API
pub use soup::{Soup, SoupConfig};
#[cfg(feature = "streaming")]
pub use streaming::{
    ContentType, HtmlRewriter, RewriterConfig, StreamingConfig, StreamingElement, StreamingSoup,
    StreamingStats, state,
};
pub use tag::Tag;
// HTML utilities
pub use utils::{escape_attr, escape_text, is_void_element};
