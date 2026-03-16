//! Custom html5ever `TreeSink` that builds a `DocumentImpl` directly.
//!
//! This replaces the dependency on `markup5ever_rcdom` by providing a
//! purpose-built sink that is compatible with `html5ever 0.39`.

use std::{
    borrow::Cow,
    cell::{Ref, RefCell},
    collections::HashMap,
    sync::OnceLock,
};

use html5ever::{
    QualName,
    interface::{Attribute, ElementFlags, NodeOrText, QuirksMode, TreeSink},
    tendril::StrTendril,
};

use super::{ParseConfig, ParseError, ParseResult};
use crate::dom::{Building, DocumentImpl, DocumentIndex, NodeId, NodeKind};

// ── Handle ───────────────────────────────────────────────────────────────────

/// A reference to a node inside `DocBuilderSink`.
///
/// - `Document` — virtual parse-tree root (no `NodeId`)
/// - `Node` — element, text, or comment
/// - `Template` — template element and its shadow contents node
/// - `Phantom` — filtered-out node (e.g. comment with `include_comments = false`)
#[derive(Clone, Debug)]
pub enum SinkHandle {
    /// Virtual document root.
    Document,
    /// Regular element, text, or comment node.
    Node(NodeId),
    /// Template element: (`element_id`, `template_contents_id`).
    Template(NodeId, NodeId),
    /// Filtered-out node; never inserted into the tree.
    Phantom,
}

impl SinkHandle {
    fn node_id(&self) -> Option<NodeId> {
        match self {
            Self::Document | Self::Phantom => None,
            Self::Node(id) | Self::Template(id, _) => Some(*id),
        }
    }

    fn template_contents_id(&self) -> Option<NodeId> {
        match self {
            Self::Template(_, contents) => Some(*contents),
            _ => None,
        }
    }
}

// ── Inner mutable state ───────────────────────────────────────────────────────

struct SinkInner {
    document: DocumentImpl<Building>,
    index: DocumentIndex,
    config: ParseConfig,
    /// Depth of each `NodeId`, used for max-depth enforcement.
    depth_map: HashMap<NodeId, usize>,
    /// Set to `true` when max depth was exceeded during parsing.
    depth_exceeded: bool,
    /// Set of nodes that are `MathML` annotation-xml integration points.
    mathml_annotation_integration_points: std::collections::HashSet<NodeId>,
}

impl SinkInner {
    fn new(config: ParseConfig, capacity: usize) -> Self {
        Self {
            document: DocumentImpl::<Building>::with_capacity(capacity),
            index: DocumentIndex::new(),
            config,
            depth_map: HashMap::new(),
            depth_exceeded: false,
            mathml_annotation_integration_points: std::collections::HashSet::new(),
        }
    }

    /// Recursively recalculates `depth_map` for `node` and all its descendants,
    /// given that `node` is now at `new_depth`.
    fn recalc_subtree_depths(&mut self, node: NodeId, new_depth: usize) {
        self.depth_map.insert(node, new_depth);
        let mut stack = vec![node];
        while let Some(current) = stack.pop() {
            let child_depth = self.depth_map.get(&current).copied().unwrap_or(new_depth) + 1;
            let mut child = self.document.get(current).and_then(|n| n.first_child);
            while let Some(child_id) = child {
                self.depth_map.insert(child_id, child_depth);
                stack.push(child_id);
                child = self.document.get(child_id).and_then(|n| n.next_sibling);
            }
        }
    }

    /// Appends a child to `parent`, checking max depth.
    fn attach(&mut self, parent: &SinkHandle, child: NodeId) -> Result<(), ParseError> {
        let parent_depth = match parent {
            SinkHandle::Document => 0,
            SinkHandle::Node(id) | SinkHandle::Template(_, id) => {
                *self.depth_map.get(id).unwrap_or(&0)
            }
            SinkHandle::Phantom => return Ok(()),
        };

        let child_depth = parent_depth + 1;
        if child_depth > self.config.max_depth {
            self.depth_exceeded = true;
            return Err(ParseError::MaxDepthExceeded {
                max_depth: self.config.max_depth,
                span: None,
            });
        }
        self.depth_map.insert(child, child_depth);

        match parent {
            SinkHandle::Document => {
                if self.document.root().is_none() {
                    self.document.set_root(child);
                }
            }
            SinkHandle::Node(parent_id) => {
                self.document.append_child(*parent_id, child);
            }
            SinkHandle::Template(_, contents_id) => {
                self.document.append_child(*contents_id, child);
            }
            SinkHandle::Phantom => {}
        }
        Ok(())
    }

    /// Creates an element node from html5ever attributes, registers it in the
    /// id/class index, and stores its `QualName` for `elem_name` lookups.
    fn make_element(
        &mut self,
        name: &QualName,
        attrs: &[Attribute],
        flags: &ElementFlags,
        qual_names: &RefCell<HashMap<NodeId, QualName>>,
    ) -> SinkHandle {
        let tag_name = name.local.to_string();
        let mut attributes = HashMap::with_capacity(attrs.len());
        for attr in attrs {
            let key = if attr.name.ns.is_empty() {
                attr.name.local.to_string()
            } else {
                format!("{}:{}", attr.name.ns, attr.name.local)
            };
            attributes.insert(key, attr.value.to_string());
        }

        let node_id = self.document.create_element(tag_name, attributes.clone());
        qual_names.borrow_mut().insert(node_id, name.clone());

        if let Some(id_attr) = attributes.get("id") {
            self.index.register_id(id_attr.clone(), node_id);
        }
        if let Some(class_attr) = attributes.get("class") {
            self.index.register_classes(class_attr, node_id);
        }

        if flags.mathml_annotation_xml_integration_point {
            self.mathml_annotation_integration_points.insert(node_id);
        }

        if flags.template {
            let contents_id =
                self.document.create_element("template-contents".to_string(), HashMap::new());
            SinkHandle::Template(node_id, contents_id)
        } else {
            SinkHandle::Node(node_id)
        }
    }
}

// ── Public sink ───────────────────────────────────────────────────────────────

/// A `TreeSink` that builds a `DocumentImpl<Building>` directly.
///
/// Uses `RefCell` for interior mutability because `TreeSink` methods take `&self`.
/// `qual_names` is a separate `RefCell` so `elem_name` can borrow it while other
/// `TreeSink` calls hold a borrow on `inner`.
pub struct DocBuilderSink {
    inner: RefCell<SinkInner>,
    /// Maps `NodeId` to the original `QualName` from html5ever.
    ///
    /// Kept separate from `inner` so `elem_name` can borrow it independently.
    /// Not cleared after parsing — this is intentional for a single-use sink.
    qual_names: RefCell<HashMap<NodeId, QualName>>,
}

impl DocBuilderSink {
    /// Creates a new sink with the given parse configuration and arena capacity.
    pub fn new(config: ParseConfig, capacity: usize) -> Self {
        Self {
            inner: RefCell::new(SinkInner::new(config, capacity)),
            qual_names: RefCell::new(HashMap::new()),
        }
    }

    /// Consumes the sink and returns the finished document.
    ///
    /// # Errors
    ///
    /// Returns `MaxDepthExceeded` if the HTML exceeded `config.max_depth`.
    pub fn finish_document(self) -> ParseResult<crate::dom::Document> {
        let inner = self.inner.into_inner();
        if inner.depth_exceeded {
            return Err(ParseError::MaxDepthExceeded {
                max_depth: inner.config.max_depth,
                span: None,
            });
        }
        let mut doc = inner.document.build();
        doc.set_index(inner.index);
        Ok(doc)
    }
}

// ── Placeholder QualName ──────────────────────────────────────────────────────

fn placeholder_qual_name() -> &'static QualName {
    static PLACEHOLDER: OnceLock<QualName> = OnceLock::new();
    PLACEHOLDER
        .get_or_init(|| QualName::new(None, html5ever::ns!(html), html5ever::local_name!("")))
}

// ── TreeSink impl ─────────────────────────────────────────────────────────────

impl TreeSink for DocBuilderSink {
    type Handle = SinkHandle;
    type Output = Self;
    /// `Ref<'a, QualName>` implements `ElemName` in markup5ever 0.39.
    type ElemName<'a>
        = Ref<'a, QualName>
    where
        Self: 'a;

    fn finish(self) -> Self::Output {
        self
    }

    fn parse_error(&self, _msg: Cow<'static, str>) {
        // html5ever parse errors are informational; error recovery is automatic.
    }

    fn get_document(&self) -> Self::Handle {
        SinkHandle::Document
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        Ref::map(self.qual_names.borrow(), |map| {
            if let Some(id) = target.node_id()
                && let Some(qn) = map.get(&id)
            {
                return qn;
            }
            placeholder_qual_name()
        })
    }

    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<Attribute>,
        flags: ElementFlags,
    ) -> Self::Handle {
        self.inner.borrow_mut().make_element(&name, &attrs, &flags, &self.qual_names)
    }

    fn create_comment(&self, text: StrTendril) -> Self::Handle {
        let mut inner = self.inner.borrow_mut();
        if !inner.config.include_comments {
            return SinkHandle::Phantom;
        }
        let node_id = inner.document.create_comment(text.to_string());
        SinkHandle::Node(node_id)
    }

    fn create_pi(&self, _target: StrTendril, _data: StrTendril) -> Self::Handle {
        // Processing instructions are not represented in our DOM.
        SinkHandle::Phantom
    }

    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        let mut inner = self.inner.borrow_mut();
        match child {
            NodeOrText::AppendText(text) => {
                if !inner.config.preserve_whitespace && text.trim().is_empty() {
                    return;
                }
                let merged = parent
                    .node_id()
                    .is_some_and(|id| inner.document.try_append_text_to_last_child(id, &text));
                if !merged {
                    let node_id = inner.document.create_text(text.to_string());
                    let _ = inner.attach(parent, node_id);
                }
            }
            NodeOrText::AppendNode(handle) => {
                if matches!(handle, SinkHandle::Phantom) {
                    return;
                }
                let Some(node_id) = handle.node_id() else { return };
                let _ = inner.attach(parent, node_id);
            }
        }
    }

    fn append_before_sibling(&self, sibling: &Self::Handle, new_node: NodeOrText<Self::Handle>) {
        let mut inner = self.inner.borrow_mut();
        let Some(sibling_id) = sibling.node_id() else { return };

        match new_node {
            NodeOrText::AppendText(text) => {
                if !inner.config.preserve_whitespace && text.trim().is_empty() {
                    return;
                }
                // Coalesce with prev_sibling if it is itself a text node (not its last child).
                let prev = inner.document.get(sibling_id).and_then(|n| n.prev_sibling);
                let merged = prev
                    .is_some_and(|prev_id| inner.document.try_append_text_to_node(prev_id, &text));
                if !merged {
                    let node_id = inner.document.create_text(text.to_string());
                    inner.document.insert_before(sibling_id, node_id);
                }
            }
            NodeOrText::AppendNode(handle) => {
                let Some(node_id) = handle.node_id() else { return };
                if inner.document.get(node_id).and_then(|n| n.parent).is_some() {
                    inner.document.remove_from_parent(node_id);
                }
                inner.document.insert_before(sibling_id, node_id);
                // Update depth_map for the inserted node and its subtree.
                let parent_depth = inner
                    .document
                    .get(sibling_id)
                    .and_then(|n| n.parent)
                    .and_then(|p| inner.depth_map.get(&p).copied())
                    .unwrap_or(0);
                inner.recalc_subtree_depths(node_id, parent_depth + 1);
            }
        }
    }

    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        let has_parent = element
            .node_id()
            .and_then(|id| self.inner.borrow().document.get(id).and_then(|n| n.parent))
            .is_some();

        if has_parent {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    fn append_doctype_to_document(
        &self,
        _name: StrTendril,
        _public_id: StrTendril,
        _system_id: StrTendril,
    ) {
        // Doctypes are not represented in our DOM.
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        match target {
            SinkHandle::Template(_, contents_id) => SinkHandle::Node(*contents_id),
            _ => panic!("get_template_contents called on non-template handle"),
        }
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        match (x, y) {
            (SinkHandle::Document, SinkHandle::Document) => true,
            (SinkHandle::Node(a), SinkHandle::Node(b))
            | (SinkHandle::Template(a, _), SinkHandle::Template(b, _)) => a == b,
            _ => false,
        }
    }

    fn set_quirks_mode(&self, _mode: QuirksMode) {}

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<Attribute>) {
        let Some(node_id) = target.node_id() else { return };
        let mut inner = self.inner.borrow_mut();
        if let Some(node) = inner.document.get_mut(node_id)
            && let NodeKind::Element { attributes, .. } = &mut node.kind
        {
            for attr in attrs {
                let key = if attr.name.ns.is_empty() {
                    attr.name.local.to_string()
                } else {
                    format!("{}:{}", attr.name.ns, attr.name.local)
                };
                attributes.entry(key).or_insert_with(|| attr.value.to_string());
            }
        }
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        if let Some(node_id) = target.node_id() {
            self.inner.borrow_mut().document.remove_from_parent(node_id);
        }
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        let (Some(src), Some(dst)) = (node.node_id(), new_parent.node_id()) else { return };
        let mut inner = self.inner.borrow_mut();
        inner.document.reparent_children(src, dst);
        // Recalculate depths for all moved children.
        let dst_depth = inner.depth_map.get(&dst).copied().unwrap_or(0);
        let mut child = inner.document.get(dst).and_then(|n| n.first_child);
        while let Some(child_id) = child {
            inner.recalc_subtree_depths(child_id, dst_depth + 1);
            child = inner.document.get(child_id).and_then(|n| n.next_sibling);
        }
    }

    fn is_mathml_annotation_xml_integration_point(&self, handle: &Self::Handle) -> bool {
        handle.node_id().is_some_and(|id| {
            self.inner.borrow().mathml_annotation_integration_points.contains(&id)
        })
    }
}

// ── Convenience functions ─────────────────────────────────────────────────────

/// Builds a `DocBuilderSink`, parses a full HTML document, and returns the result.
pub fn parse_html_document(
    html: &str,
    config: &ParseConfig,
    capacity: usize,
) -> ParseResult<crate::dom::Document> {
    use html5ever::{ParseOpts, parse_document, tendril::TendrilSink};

    let sink = DocBuilderSink::new(config.clone(), capacity);
    let sink = parse_document(sink, ParseOpts::default())
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .map_err(|e| ParseError::InternalError(e.to_string()))?;
    sink.finish_document()
}

/// Builds a `DocBuilderSink`, parses an HTML fragment, and returns the result.
pub fn parse_html_fragment(
    html: &str,
    context: &str,
    config: &ParseConfig,
) -> ParseResult<crate::dom::Document> {
    use html5ever::{ParseOpts, parse_fragment as html5ever_parse_fragment, tendril::TendrilSink};
    use markup5ever::QualName;

    let context_name =
        QualName::new(None, html5ever::ns!(html), html5ever::LocalName::from(context));

    let sink = DocBuilderSink::new(config.clone(), 64);
    let sink = html5ever_parse_fragment(sink, ParseOpts::default(), context_name, vec![], false)
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .map_err(|e| ParseError::InternalError(e.to_string()))?;

    finish_fragment(sink, config)
}

/// Finalises a fragment sink by unwrapping the html/body wrappers that
/// html5ever adds around fragment content, then building the final `Document`.
fn finish_fragment(
    sink: DocBuilderSink,
    config: &ParseConfig,
) -> ParseResult<crate::dom::Document> {
    let inner = sink.inner.into_inner();
    if inner.depth_exceeded {
        return Err(ParseError::MaxDepthExceeded { max_depth: inner.config.max_depth, span: None });
    }
    let mut doc = inner.document.build();
    doc.set_index(inner.index);

    // html5ever fragment parsing wraps the content in <html><body>; unwrap it.
    let Some(root) = doc.root() else { return Ok(doc) };

    let body = unwrap_single_element_child(&doc, root, "html")
        .and_then(|html_id| unwrap_single_element_child(&doc, html_id, "body"))
        .or_else(|| unwrap_single_element_child(&doc, root, "html"));

    let fragment_root = body.unwrap_or(root);

    let real_children: Vec<_> = doc
        .children(fragment_root)
        .filter(|&id| {
            doc.get(id).is_some_and(|node| match &node.kind {
                NodeKind::Comment { .. } => config.include_comments,
                NodeKind::Text { content } => {
                    config.preserve_whitespace || !content.trim().is_empty()
                }
                NodeKind::Element { .. } => true,
            })
        })
        .collect();

    if real_children.is_empty() {
        return Ok(crate::dom::DocumentImpl::default());
    }

    if real_children.len() == 1 {
        doc.set_root(real_children[0]);
    } else {
        doc.set_root(fragment_root);
    }

    Ok(doc)
}

/// Returns the first child of `parent` that is an element named `tag`.
fn unwrap_single_element_child(
    doc: &crate::dom::Document,
    parent: NodeId,
    tag: &str,
) -> Option<NodeId> {
    for child_id in doc.children(parent) {
        if let Some(node) = doc.get(child_id)
            && let NodeKind::Element { name, .. } = &node.kind
            && name == tag
        {
            return Some(child_id);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sink_handle_node_id() {
        let id = NodeId::new(5);
        let h = SinkHandle::Node(id);
        assert_eq!(h.node_id(), Some(id));
        assert!(h.template_contents_id().is_none());

        let h_doc = SinkHandle::Document;
        assert!(h_doc.node_id().is_none());
    }

    #[test]
    fn test_sink_handle_template() {
        let elem = NodeId::new(1);
        let contents = NodeId::new(2);
        let h = SinkHandle::Template(elem, contents);
        assert_eq!(h.node_id(), Some(elem));
        assert_eq!(h.template_contents_id(), Some(contents));
    }

    #[test]
    fn test_same_node() {
        let sink = DocBuilderSink::new(ParseConfig::default(), 16);
        let a = SinkHandle::Node(NodeId::new(1));
        let b = SinkHandle::Node(NodeId::new(1));
        let c = SinkHandle::Node(NodeId::new(2));
        assert!(sink.same_node(&a, &b));
        assert!(!sink.same_node(&a, &c));
        assert!(sink.same_node(&SinkHandle::Document, &SinkHandle::Document));
        assert!(!sink.same_node(&SinkHandle::Document, &a));
    }

    /// Verifies that consecutive text runs inside an element are coalesced into
    /// a single text node.  html5ever may emit multiple `AppendText` calls for
    /// the same parent (e.g. for `<h1>Hello World</h1>`), and the sink must
    /// merge them rather than creating sibling text nodes.
    fn find_h1_node(doc: &crate::dom::Document, id: NodeId) -> Option<NodeId> {
        let node = doc.get(id)?;
        if let NodeKind::Element { name, .. } = &node.kind
            && name == "h1"
        {
            return Some(id);
        }
        doc.children(id).find_map(|child| find_h1_node(doc, child))
    }

    #[test]
    fn test_text_coalescing_in_h1() {
        let config = ParseConfig { preserve_whitespace: true, ..ParseConfig::default() };
        let doc =
            parse_html_document("<html><body><h1>Hello World</h1></body></html>", &config, 64)
                .expect("parse failed");

        let root = doc.root().expect("document has no root");
        let h1 = find_h1_node(&doc, root).expect("h1 not found");

        let children: Vec<_> = doc.children(h1).collect();
        assert_eq!(children.len(), 1, "expected single text child, got {}", children.len());
        let text_node = doc.get(children[0]).expect("child node missing");
        match &text_node.kind {
            NodeKind::Text { content } => {
                assert_eq!(content, "Hello World");
            }
            other => panic!("expected Text node, got {other:?}"),
        }
    }
}
