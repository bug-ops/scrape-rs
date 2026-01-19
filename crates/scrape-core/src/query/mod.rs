//! Query engine for finding elements in the DOM.
//!
//! This module provides various ways to query the DOM tree:
//!
//! - **CSS Selectors**: Standard CSS selector syntax via the `selectors` crate
//! - **Filters**: BeautifulSoup-style attribute filtering
//!
//! # CSS Selectors
//!
//! Use [`find`] and [`find_all`] to query by CSS selector:
//!
//! ```rust
//! use scrape_core::{
//!     Html5everParser, Parser,
//!     query::{find, find_all},
//! };
//!
//! let parser = Html5everParser;
//! let doc = parser.parse("<div class=\"item\"><span>A</span></div>").unwrap();
//!
//! // Find first element matching selector
//! let span = find(&doc, "div.item span").unwrap();
//!
//! // Find all matching elements
//! let items = find_all(&doc, ".item").unwrap();
//! ```
//!
//! # Attribute Filters
//!
//! Use [`Filter`] for BeautifulSoup-style queries:
//!
//! ```rust
//! use scrape_core::{
//!     Html5everParser, Parser,
//!     query::{Filter, find_by_filter},
//! };
//!
//! let parser = Html5everParser;
//! let doc = parser.parse("<div class=\"item\" data-id=\"123\">text</div>").unwrap();
//!
//! let filter = Filter::new().tag("div").class("item").attr("data-id", "123");
//!
//! let results = find_by_filter(&doc, &filter);
//! ```
//!
//! # Supported CSS Selectors
//!
//! | Selector | Example | Description |
//! |----------|---------|-------------|
//! | Type | `div` | Matches elements by tag name |
//! | Class | `.foo` | Matches elements with class |
//! | ID | `#bar` | Matches element by ID |
//! | Universal | `*` | Matches all elements |
//! | Attribute | `[href]` | Matches elements with attribute |
//! | Attribute value | `[type="text"]` | Matches attribute with value |
//! | Descendant | `div span` | Matches descendants |
//! | Child | `div > span` | Matches direct children |
//! | Adjacent sibling | `h1 + p` | Matches adjacent sibling |
//! | General sibling | `h1 ~ p` | Matches following siblings |
//! | :first-child | `li:first-child` | First child element |
//! | :last-child | `li:last-child` | Last child element |
//! | :nth-child | `li:nth-child(2n)` | Nth child element |
//! | :empty | `div:empty` | Elements with no children |
//! | :not() | `div:not(.hidden)` | Negation |

mod compiled;
mod error;
mod filter;
mod find;
mod selector;
mod text;

pub use compiled::{CompiledSelector, compile_selector};
pub use error::{QueryError, QueryResult};
pub use filter::{Filter, find_by_filter, find_first_by_filter};
pub use find::{
    find, find_all, find_all_compiled, find_all_with_selector, find_all_within,
    find_all_within_compiled, find_all_within_with_selector, find_compiled, find_with_selector,
    find_within, find_within_compiled, find_within_with_selector,
};
pub use selector::{
    ElementWrapper, NonTSPseudoClass, PseudoElement, ScrapeSelector, matches_selector,
    matches_selector_list, matches_selector_with_caches, parse_selector,
};
pub use text::TextNodesIter;
