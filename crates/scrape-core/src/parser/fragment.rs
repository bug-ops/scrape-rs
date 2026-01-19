//! HTML fragment parsing without wrapping in html/body.

use std::collections::HashMap;

use html5ever::{ParseOpts, parse_fragment as html5ever_parse_fragment, tendril::TendrilSink};
use markup5ever::QualName;
use markup5ever_rcdom::{Handle, NodeData, RcDom};

use super::{ParseConfig, ParseError, ParseResult};
use crate::dom::{Document, NodeId};

/// Parses an HTML fragment with body context.
///
/// Unlike full document parsing, fragment parsing does not wrap content in
/// html/head/body tags. The fragment is parsed as if it appeared inside
/// a `<body>` element.
///
/// Users should use [`crate::Soup::parse_fragment`] instead of this function directly.
///
/// # Errors
///
/// Returns [`ParseError::EmptyInput`] if the input is empty or whitespace-only.
pub fn parse_fragment(html: &str) -> ParseResult<Document> {
    parse_fragment_with_context(html, "body")
}

/// Parses an HTML fragment with a custom context element.
///
/// The context element determines parsing behavior:
/// - `"body"`: Standard HTML elements (default)
/// - `"table"`: Allows tr/td without explicit tbody
/// - `"tbody"`: Allows tr directly
/// - etc.
///
/// Users should use [`crate::Soup::parse_fragment_with_context`] instead of this function directly.
///
/// # Errors
///
/// Returns [`ParseError::EmptyInput`] if the input is empty or whitespace-only.
pub fn parse_fragment_with_context(html: &str, context: &str) -> ParseResult<Document> {
    parse_fragment_impl(html, context, &ParseConfig::default())
}

/// Internal fragment parsing implementation with configuration.
pub fn parse_fragment_impl(
    html: &str,
    context: &str,
    config: &ParseConfig,
) -> ParseResult<Document> {
    if html.trim().is_empty() {
        return Err(ParseError::EmptyInput);
    }

    let context_name =
        QualName::new(None, html5ever::ns!(html), html5ever::LocalName::from(context));

    let dom = html5ever_parse_fragment(
        RcDom::default(),
        ParseOpts::default(),
        context_name,
        vec![],
        false, // quirks mode
    )
    .from_utf8()
    .read_from(&mut html.as_bytes())
    .map_err(|e| ParseError::InternalError(e.to_string()))?;

    convert_fragment_to_document(&dom, config)
}

/// Converts an html5ever fragment `RcDom` to our Document representation.
fn convert_fragment_to_document(dom: &RcDom, config: &ParseConfig) -> ParseResult<Document> {
    let mut document = Document::new();
    let mut depth = 0;

    // html5ever fragment parsing may wrap content in html/body elements
    // We need to unwrap these to get the actual fragment content
    let fragment_children = extract_fragment_children(&dom.document);

    if fragment_children.is_empty() {
        return Ok(document);
    }

    if fragment_children.len() == 1 {
        // Single root node - use it directly
        if let Some(root_id) =
            convert_node(&fragment_children[0], &mut document, None, &mut depth, config)?
        {
            document.set_root(root_id);
        }
    } else {
        // Multiple root nodes - create synthetic container
        let container_id = document.create_element("fragment".to_string(), HashMap::new());
        document.set_root(container_id);

        for child in &fragment_children {
            convert_node(child, &mut document, Some(container_id), &mut depth, config)?;
        }
    }

    Ok(document)
}

/// Extracts the actual fragment children, unwrapping html/body wrappers.
fn extract_fragment_children(document_node: &Handle) -> Vec<Handle> {
    let children: Vec<_> = document_node.children.borrow().iter().cloned().collect();

    // If we have a single html element, unwrap it
    if children.len() == 1
        && let NodeData::Element { name, .. } = &children[0].data
        && name.local.as_ref() == "html"
    {
        // Get children of html element
        let html_children: Vec<_> = children[0].children.borrow().iter().cloned().collect();

        // If we have a single body element, unwrap it
        if html_children.len() == 1
            && let NodeData::Element { name, .. } = &html_children[0].data
            && name.local.as_ref() == "body"
        {
            // Return children of body element
            return html_children[0].children.borrow().iter().cloned().collect();
        }

        // Return children of html element
        return html_children;
    }

    children
}

/// Recursively converts a fragment node and its children to our DOM representation.
fn convert_node(
    handle: &Handle,
    document: &mut Document,
    parent: Option<NodeId>,
    depth: &mut usize,
    config: &ParseConfig,
) -> ParseResult<Option<NodeId>> {
    if *depth > config.max_depth {
        return Err(ParseError::MaxDepthExceeded { max_depth: config.max_depth });
    }
    *depth = depth.saturating_add(1);

    let result = match &handle.data {
        NodeData::Document => {
            // Skip document nodes in fragments
            *depth = depth.saturating_sub(1);
            return Ok(None);
        }

        NodeData::Element { name, attrs, .. } => {
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

            let node_id = document.create_element(tag_name, attributes);

            if let Some(parent_id) = parent {
                document.append_child(parent_id, node_id);
            }

            // Process children
            for child in handle.children.borrow().iter() {
                convert_node(child, document, Some(node_id), depth, config)?;
            }

            Some(node_id)
        }

        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();

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
            *depth = depth.saturating_sub(1);
            return Ok(None);
        }
    };

    *depth = depth.saturating_sub(1);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fragment_simple() {
        let doc = parse_fragment("<span>Hello</span>").unwrap();
        assert!(doc.root().is_some());
    }

    #[test]
    fn test_parse_fragment_multiple_roots() {
        let doc = parse_fragment("<span>A</span><span>B</span>").unwrap();
        let root = doc.root().unwrap();
        let node = doc.get(root).unwrap();

        // Should have synthetic container
        if let crate::dom::NodeKind::Element { name, .. } = &node.kind {
            assert_eq!(name, "fragment");
        } else {
            panic!("Expected element node");
        }
    }

    #[test]
    fn test_parse_fragment_text_only() {
        let doc = parse_fragment("Just text").unwrap();
        assert!(doc.root().is_some());
    }

    #[test]
    fn test_parse_fragment_empty_returns_error() {
        let result = parse_fragment("");
        assert!(matches!(result, Err(ParseError::EmptyInput)));
    }

    #[test]
    fn test_parse_fragment_whitespace_returns_error() {
        let result = parse_fragment("   ");
        assert!(matches!(result, Err(ParseError::EmptyInput)));
    }

    #[test]
    fn test_parse_fragment_nested() {
        let doc = parse_fragment("<div><p><span>deep</span></p></div>").unwrap();
        assert!(doc.root().is_some());
    }

    #[test]
    fn test_parse_fragment_self_closing() {
        let doc = parse_fragment("<br><hr><img src='test'>").unwrap();
        assert!(doc.root().is_some());
    }

    #[test]
    fn test_parse_fragment_with_context_table() {
        let doc = parse_fragment_with_context("<tr><td>A</td></tr>", "tbody").unwrap();
        assert!(doc.root().is_some());
    }

    #[test]
    fn test_parse_fragment_with_context_body() {
        let doc = parse_fragment_with_context("<div>Test</div>", "body").unwrap();
        assert!(doc.root().is_some());
    }

    #[test]
    fn test_parse_fragment_malformed() {
        let doc = parse_fragment("<div><span>no close").unwrap();
        assert!(doc.root().is_some());
    }

    #[test]
    fn test_parse_fragment_preserves_attributes() {
        let doc = parse_fragment("<div class='test' id='main'>text</div>").unwrap();
        let root = doc.root().unwrap();
        let node = doc.get(root).unwrap();

        if let crate::dom::NodeKind::Element { attributes, .. } = &node.kind {
            assert_eq!(attributes.get("class"), Some(&"test".to_string()));
            assert_eq!(attributes.get("id"), Some(&"main".to_string()));
        } else {
            panic!("Expected element node");
        }
    }

    #[test]
    fn test_parse_fragment_max_depth() {
        let config =
            ParseConfig { max_depth: 5, preserve_whitespace: false, include_comments: false };

        let result = parse_fragment_impl(
            "<div><div><div><div><div><div>too deep</div></div></div></div></div></div>",
            "body",
            &config,
        );

        assert!(matches!(result, Err(ParseError::MaxDepthExceeded { .. })));
    }
}
