//! Iterator over text nodes in a DOM subtree.

use crate::dom::{Document, NodeId, NodeKind};

/// Iterator over text content within an element subtree.
///
/// Returns only text node content, skipping element tags and comments.
/// Iterates in depth-first order.
///
/// # Examples
///
/// ```rust
/// use scrape_core::Soup;
///
/// let soup = Soup::parse("<div>Hello <b>World</b>!</div>");
/// if let Ok(Some(div)) = soup.find("div") {
///     let texts: Vec<_> = div.text_nodes().collect();
///     assert_eq!(texts, vec!["Hello ", "World", "!"]);
/// }
/// ```
pub struct TextNodesIter<'a> {
    doc: &'a Document,
    stack: Vec<NodeId>,
}

impl<'a> TextNodesIter<'a> {
    /// Creates a new text nodes iterator rooted at the given node.
    #[must_use]
    pub fn new(doc: &'a Document, root: NodeId) -> Self {
        Self { doc, stack: vec![root] }
    }
}

impl<'a> Iterator for TextNodesIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(id) = self.stack.pop() {
            let Some(node) = self.doc.get(id) else {
                continue;
            };

            match &node.kind {
                NodeKind::Text { content } => {
                    return Some(content.as_str());
                }
                NodeKind::Element { .. } => {
                    // Push children in reverse order for depth-first traversal
                    // Collect required: ChildrenIter does not implement DoubleEndedIterator
                    #[allow(clippy::needless_collect)]
                    let children: Vec<_> = self.doc.children(id).collect();
                    for child_id in children.into_iter().rev() {
                        self.stack.push(child_id);
                    }
                }
                NodeKind::Comment { .. } => {}
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{Soup, SoupConfig};

    #[test]
    fn test_text_nodes_single_text() {
        let soup = Soup::parse("<div>Hello</div>");
        let div = soup.find("div").unwrap().unwrap();
        let texts: Vec<_> = div.text_nodes().collect();
        assert_eq!(texts, vec!["Hello"]);
    }

    #[test]
    fn test_text_nodes_nested() {
        let soup = Soup::parse("<div>Hello <b>World</b>!</div>");
        let div = soup.find("div").unwrap().unwrap();
        let texts: Vec<_> = div.text_nodes().collect();
        assert_eq!(texts, vec!["Hello ", "World", "!"]);
    }

    #[test]
    fn test_text_nodes_empty_element() {
        let soup = Soup::parse("<div></div>");
        let div = soup.find("div").unwrap().unwrap();
        assert!(div.text_nodes().next().is_none());
    }

    #[test]
    fn test_text_nodes_skips_comments() {
        let config = SoupConfig::builder().include_comments(true).build();
        let soup = Soup::parse_with_config("<div>A<!--comment-->B</div>", config);
        let div = soup.find("div").unwrap().unwrap();
        let texts: Vec<_> = div.text_nodes().collect();
        assert_eq!(texts, vec!["A", "B"]);
    }

    #[test]
    fn test_text_nodes_deeply_nested() {
        let soup = Soup::parse("<div><p><span>A</span></p><p><span>B</span></p></div>");
        let div = soup.find("div").unwrap().unwrap();
        let texts: Vec<_> = div.text_nodes().collect();
        assert_eq!(texts, vec!["A", "B"]);
    }

    #[test]
    fn test_text_nodes_depth_first_order() {
        let soup = Soup::parse("<div>1<span>2<b>3</b>4</span>5</div>");
        let div = soup.find("div").unwrap().unwrap();
        let texts: Vec<_> = div.text_nodes().collect();
        assert_eq!(texts, vec!["1", "2", "3", "4", "5"]);
    }

    #[test]
    fn test_text_nodes_whitespace() {
        let config = SoupConfig::builder().preserve_whitespace(true).build();
        let soup = Soup::parse_with_config("<div>  A  </div>", config);
        let div = soup.find("div").unwrap().unwrap();
        let texts: Vec<_> = div.text_nodes().collect();
        assert_eq!(texts.len(), 1);
        assert!(texts[0].contains("  A  "));
    }
}
