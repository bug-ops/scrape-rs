//! html5ever-based HTML parser implementation.

use std::collections::HashMap;

use html5ever::{ParseOpts, parse_document, tendril::TendrilSink};
use markup5ever_rcdom::{Handle, NodeData, RcDom};

use super::{ParseConfig, ParseError, ParseResult, Parser, private::Sealed};
use crate::dom::{Document, DocumentIndex, NodeId};

/// HTML5 spec-compliant parser using html5ever.
///
/// This parser uses the [html5ever](https://github.com/servo/html5ever) crate
/// for spec-compliant HTML5 parsing. It handles malformed HTML gracefully
/// using the HTML5 error recovery algorithm.
///
/// # Example
///
/// ```rust
/// use scrape_core::{Html5everParser, Parser};
///
/// let parser = Html5everParser;
/// let document = parser.parse("<html><body><h1>Hello</h1></body></html>").unwrap();
/// assert!(document.root().is_some());
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct Html5everParser;

impl Sealed for Html5everParser {}

impl Parser for Html5everParser {
    fn parse_with_config(&self, html: &str, config: &ParseConfig) -> ParseResult<Document> {
        self.parse_with_config_and_capacity(html, config, 256)
    }
}

impl Html5everParser {
    /// Parses HTML with the given configuration and pre-allocated capacity.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] if parsing fails.
    pub fn parse_with_config_and_capacity(
        &self,
        html: &str,
        config: &ParseConfig,
        capacity: usize,
    ) -> ParseResult<Document> {
        if html.trim().is_empty() {
            return Err(ParseError::EmptyInput);
        }

        let dom = parse_document(RcDom::default(), ParseOpts::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .map_err(|e| ParseError::InternalError(e.to_string()))?;

        convert_rcdom_to_document_with_capacity(&dom, config, capacity)
    }
}

/// Converts an html5ever `RcDom` to our Document representation.
fn convert_rcdom_to_document(dom: &RcDom, config: &ParseConfig) -> ParseResult<Document> {
    convert_rcdom_to_document_with_capacity(dom, config, 256)
}

/// Converts an html5ever `RcDom` to our Document representation with pre-allocated capacity.
fn convert_rcdom_to_document_with_capacity(
    dom: &RcDom,
    config: &ParseConfig,
    capacity: usize,
) -> ParseResult<Document> {
    let mut document = crate::dom::DocumentImpl::<crate::dom::Building>::with_capacity(capacity);
    let mut depth = 0;
    let mut index = DocumentIndex::new();

    convert_node(&dom.document, &mut document, None, &mut depth, config, &mut index)?;

    let mut document = document.build();
    document.set_index(index);
    Ok(document)
}

/// Recursively converts an `RcDom` node and its children to our DOM representation.
fn convert_node(
    handle: &Handle,
    document: &mut crate::dom::DocumentImpl<crate::dom::Building>,
    parent: Option<NodeId>,
    depth: &mut usize,
    config: &ParseConfig,
    index: &mut DocumentIndex,
) -> ParseResult<Option<NodeId>> {
    if *depth > config.max_depth {
        return Err(ParseError::MaxDepthExceeded { max_depth: config.max_depth, span: None });
    }
    *depth = depth.saturating_add(1);

    let result = match &handle.data {
        NodeData::Document => {
            // Process children of document node without creating a node
            for child in handle.children.borrow().iter() {
                if let Some(child_id) = convert_node(child, document, None, depth, config, index)?
                    && document.root().is_none()
                {
                    document.set_root(child_id);
                }
            }
            *depth = depth.saturating_sub(1);
            return Ok(None);
        }

        NodeData::Element { name, attrs, .. } => {
            // html5ever normalizes tag names to lowercase during parsing
            let tag_name = name.local.to_string();

            let attrs_ref = attrs.borrow();
            let mut attributes = HashMap::with_capacity(attrs_ref.len());
            for attr in attrs_ref.iter() {
                let key = if attr.name.ns.is_empty() {
                    attr.name.local.to_string()
                } else {
                    format!("{}:{}", attr.name.ns, attr.name.local)
                };
                attributes.insert(key, attr.value.to_string());
            }

            let node_id = document.create_element(tag_name, attributes.clone());

            if let Some(id_attr) = attributes.get("id") {
                index.register_id(id_attr.clone(), node_id);
            }
            if let Some(class_attr) = attributes.get("class") {
                index.register_classes(class_attr, node_id);
            }

            if let Some(parent_id) = parent {
                document.append_child(parent_id, node_id);
            } else if document.root().is_none() {
                document.set_root(node_id);
            }

            // Process children
            for child in handle.children.borrow().iter() {
                convert_node(child, document, Some(node_id), depth, config, index)?;
            }

            Some(node_id)
        }

        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();

            // Skip whitespace-only text nodes unless configured to preserve
            if !config.preserve_whitespace && text.trim().is_empty() {
                *depth = depth.saturating_sub(1);
                return Ok(None);
            }

            let node_id = document.create_text(text);

            if let Some(parent_id) = parent {
                document.append_child(parent_id, node_id);
            }

            Some(node_id)
        }

        NodeData::Comment { contents } => {
            if !config.include_comments {
                *depth = depth.saturating_sub(1);
                return Ok(None);
            }

            let node_id = document.create_comment(contents.to_string());

            if let Some(parent_id) = parent {
                document.append_child(parent_id, node_id);
            }

            Some(node_id)
        }

        NodeData::Doctype { .. } | NodeData::ProcessingInstruction { .. } => {
            // Skip doctype and processing instructions
            *depth = depth.saturating_sub(1);
            return Ok(None);
        }
    };

    *depth = depth.saturating_sub(1);
    Ok(result)
}
