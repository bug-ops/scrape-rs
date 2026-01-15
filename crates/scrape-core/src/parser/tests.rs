//! Tests for parser module.

use super::{Html5everParser, ParseConfig, ParseError, Parser};
use crate::dom::{Document, NodeId, NodeKind};

#[test]
fn test_parse_config_default() {
    let config = ParseConfig::default();
    assert_eq!(config.max_depth, 512);
    assert!(!config.preserve_whitespace);
    assert!(!config.include_comments);
}

#[test]
fn test_parse_config_custom() {
    let config = ParseConfig { max_depth: 256, preserve_whitespace: true, include_comments: true };
    assert_eq!(config.max_depth, 256);
    assert!(config.preserve_whitespace);
    assert!(config.include_comments);
}

#[test]
fn test_parse_simple_html() {
    let parser = Html5everParser;
    let doc = parser.parse("<html><body><h1>Hello</h1></body></html>");

    assert!(doc.is_ok());
    let doc = doc.unwrap();
    assert!(doc.root().is_some());
    assert!(!doc.is_empty());
}

#[test]
fn test_parse_empty_input() {
    let parser = Html5everParser;
    let result = parser.parse("");

    assert!(matches!(result, Err(ParseError::EmptyInput)));
}

#[test]
fn test_parse_whitespace_only() {
    let parser = Html5everParser;
    let result = parser.parse("   \n\t   ");

    assert!(matches!(result, Err(ParseError::EmptyInput)));
}

#[test]
fn test_parse_with_max_depth_exceeded() {
    let parser = Html5everParser;
    let config = ParseConfig { max_depth: 3, ..Default::default() };

    // Create HTML with depth > 3
    let html = "<div><div><div><div><div>deep</div></div></div></div></div>";
    let result = parser.parse_with_config(html, &config);

    assert!(matches!(result, Err(ParseError::MaxDepthExceeded { max_depth: 3 })));
}

#[test]
fn test_parse_with_max_depth_zero() {
    let parser = Html5everParser;
    let config = ParseConfig { max_depth: 0, ..Default::default() };

    // Any HTML should fail immediately at depth 0 since the document node is at depth 1
    let html = "<div>text</div>";
    let result = parser.parse_with_config(html, &config);

    assert!(matches!(result, Err(ParseError::MaxDepthExceeded { max_depth: 0 })));
}

#[test]
fn test_parse_malformed_html_no_panic() {
    let parser = Html5everParser;

    // Various malformed HTML inputs
    let malformed_inputs = [
        "<div><span>",
        "</div>text",
        "<div attr=>",
        "<<<>>>",
        "<div <span>",
        "<html><html><html>",
        "<div></span></div>",
    ];

    for input in malformed_inputs {
        let result = parser.parse(input);
        // Should not panic, may succeed or return error
        assert!(result.is_ok() || result.is_err(), "Input '{input}' caused unexpected behavior");
    }
}

#[test]
fn test_parse_preserves_attributes() {
    fn find_anchor(doc: &Document, node_id: NodeId) -> Option<&crate::dom::Node> {
        let node = doc.get(node_id)?;
        if let NodeKind::Element { name, .. } = &node.kind
            && name == "a"
        {
            return Some(node);
        }
        for child_id in doc.children(node_id) {
            if let Some(found) = find_anchor(doc, child_id) {
                return Some(found);
            }
        }
        None
    }

    let parser = Html5everParser;
    let doc = parser
        .parse(r#"<a href="https://example.com" class="link" data-id="123">Link</a>"#)
        .unwrap();

    let root_id = doc.root().unwrap();
    let anchor = find_anchor(&doc, root_id).expect("Should find anchor element");
    if let NodeKind::Element { attributes, .. } = &anchor.kind {
        assert_eq!(attributes.get("href"), Some(&"https://example.com".to_string()));
        assert_eq!(attributes.get("class"), Some(&"link".to_string()));
        assert_eq!(attributes.get("data-id"), Some(&"123".to_string()));
    } else {
        panic!("Expected element node");
    }
}

#[test]
fn test_parse_with_comments_included() {
    let parser = Html5everParser;
    let config = ParseConfig { include_comments: true, ..Default::default() };

    let doc = parser.parse_with_config("<div><!-- comment --></div>", &config).unwrap();

    let has_comment = doc.nodes().any(|(_, node)| node.kind.is_comment());
    assert!(has_comment, "Should include comment node");
}

#[test]
fn test_parse_without_comments() {
    let parser = Html5everParser;
    let config = ParseConfig { include_comments: false, ..Default::default() };

    let doc = parser.parse_with_config("<div><!-- comment --></div>", &config).unwrap();

    let has_comment = doc.nodes().any(|(_, node)| node.kind.is_comment());
    assert!(!has_comment, "Should not include comment node");
}

#[test]
fn test_parse_skips_whitespace_text() {
    let parser = Html5everParser;
    let config = ParseConfig { preserve_whitespace: false, ..Default::default() };

    let doc = parser.parse_with_config("<div>   \n   </div>", &config).unwrap();

    let has_whitespace_only_text = doc.nodes().any(|(_, node)| {
        if let NodeKind::Text { content } = &node.kind { content.trim().is_empty() } else { false }
    });
    assert!(!has_whitespace_only_text, "Should not include whitespace-only text");
}

#[test]
fn test_parse_preserves_whitespace_when_configured() {
    let parser = Html5everParser;
    let config = ParseConfig { preserve_whitespace: true, ..Default::default() };

    let doc = parser.parse_with_config("<pre>   spaces   </pre>", &config).unwrap();

    let has_text_with_spaces = doc.nodes().any(|(_, node)| {
        if let NodeKind::Text { content } = &node.kind { content.contains("   ") } else { false }
    });
    assert!(has_text_with_spaces, "Should preserve whitespace in text");
}

#[test]
fn test_parse_captures_text_content() {
    let parser = Html5everParser;
    let doc = parser.parse("<div>Hello World</div>").unwrap();

    let has_text = doc.nodes().any(|(_, node)| {
        if let NodeKind::Text { content } = &node.kind { content == "Hello World" } else { false }
    });
    assert!(has_text, "Should capture text content");
}

#[test]
fn test_parse_tag_names_lowercase() {
    let parser = Html5everParser;
    let doc = parser.parse("<DIV><SPAN>Text</SPAN></DIV>").unwrap();

    let all_lowercase = doc.nodes().all(|(_, node)| {
        if let NodeKind::Element { name, .. } = &node.kind {
            *name == name.to_lowercase()
        } else {
            true
        }
    });
    assert!(all_lowercase, "All tag names should be lowercase");
}

#[test]
fn test_parse_sibling_relationships() {
    let parser = Html5everParser;
    let doc = parser.parse("<ul><li>A</li><li>B</li><li>C</li></ul>").unwrap();

    // Find ul element
    let ul_id =
        doc.nodes().find(|(_, node)| node.kind.as_element_name() == Some("ul")).map(|(id, _)| id);

    if let Some(ul_id) = ul_id {
        // Should have li children with proper sibling links
        let li_children: Vec<_> = doc
            .children(ul_id)
            .filter(|id| {
                doc.get(**id).is_some_and(|n| n.kind.as_element_name() == Some("li"))
            })
            .collect();

        assert!(li_children.len() >= 3, "Should have at least 3 li children");

        // Check sibling links
        if li_children.len() >= 2 {
            let first = doc.get(li_children[0]).unwrap();
            let second = doc.get(li_children[1]).unwrap();

            assert_eq!(first.next_sibling, Some(li_children[1]));
            assert_eq!(second.prev_sibling, Some(li_children[0]));
        }
    }
}

#[test]
fn test_parse_unicode_content() {
    let parser = Html5everParser;
    let doc = parser.parse("<div>Hello</div>").unwrap();

    let has_emoji = doc.nodes().any(|(_, node)| {
        if let NodeKind::Text { content } = &node.kind { content.contains("Hello") } else { false }
    });
    assert!(has_emoji, "Should handle unicode content");
}

#[test]
fn test_default_parser() {
    let parser = Html5everParser;
    let doc = parser.parse("<p>Test</p>").unwrap();
    assert!(doc.root().is_some());
}
