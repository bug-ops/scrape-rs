//! Interned tag identifiers for fast comparison.

/// Interned tag identifier for common HTML5 elements.
///
/// Known tags use a u8 discriminant for O(1) comparison.
/// Unknown/custom tags fall back to string comparison.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum TagId {
    Unknown = 0,
    A = 1,
    Abbr = 2,
    Address = 3,
    Area = 4,
    Article = 5,
    Aside = 6,
    Audio = 7,
    B = 8,
    Base = 9,
    Bdi = 10,
    Bdo = 11,
    Blockquote = 12,
    Body = 13,
    Br = 14,
    Button = 15,
    Canvas = 16,
    Caption = 17,
    Cite = 18,
    Code = 19,
    Col = 20,
    Colgroup = 21,
    Data = 22,
    Datalist = 23,
    Dd = 24,
    Del = 25,
    Details = 26,
    Dfn = 27,
    Dialog = 28,
    Div = 29,
    Dl = 30,
    Dt = 31,
    Em = 32,
    Embed = 33,
    Fieldset = 34,
    Figcaption = 35,
    Figure = 36,
    Footer = 37,
    Form = 38,
    H1 = 39,
    H2 = 40,
    H3 = 41,
    H4 = 42,
    H5 = 43,
    H6 = 44,
    Head = 45,
    Header = 46,
    Hgroup = 47,
    Hr = 48,
    Html = 49,
    I = 50,
    Iframe = 51,
    Img = 52,
    Input = 53,
    Ins = 54,
    Kbd = 55,
    Label = 56,
    Legend = 57,
    Li = 58,
    Link = 59,
    Main = 60,
    Map = 61,
    Mark = 62,
    Menu = 63,
    Meta = 64,
    Meter = 65,
    Nav = 66,
    Noscript = 67,
    Object = 68,
    Ol = 69,
    Optgroup = 70,
    Option = 71,
    Output = 72,
    P = 73,
    Param = 74,
    Picture = 75,
    Pre = 76,
    Progress = 77,
    Q = 78,
    Rp = 79,
    Rt = 80,
    Ruby = 81,
    S = 82,
    Samp = 83,
    Script = 84,
    Section = 85,
    Select = 86,
    Slot = 87,
    Small = 88,
    Source = 89,
    Span = 90,
    Strong = 91,
    Style = 92,
    Sub = 93,
    Summary = 94,
    Sup = 95,
    Table = 96,
    Tbody = 97,
    Td = 98,
    Template = 99,
    Textarea = 100,
    Tfoot = 101,
    Th = 102,
    Thead = 103,
    Time = 104,
    Title = 105,
    Tr = 106,
    Track = 107,
    U = 108,
    Ul = 109,
    Var = 110,
    Video = 111,
    Wbr = 112,
}

impl TagId {
    /// Interns a tag name string to a `TagId`.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn from_name(name: &str) -> Self {
        match name {
            "a" => Self::A,
            "abbr" => Self::Abbr,
            "address" => Self::Address,
            "area" => Self::Area,
            "article" => Self::Article,
            "aside" => Self::Aside,
            "audio" => Self::Audio,
            "b" => Self::B,
            "base" => Self::Base,
            "bdi" => Self::Bdi,
            "bdo" => Self::Bdo,
            "blockquote" => Self::Blockquote,
            "body" => Self::Body,
            "br" => Self::Br,
            "button" => Self::Button,
            "canvas" => Self::Canvas,
            "caption" => Self::Caption,
            "cite" => Self::Cite,
            "code" => Self::Code,
            "col" => Self::Col,
            "colgroup" => Self::Colgroup,
            "data" => Self::Data,
            "datalist" => Self::Datalist,
            "dd" => Self::Dd,
            "del" => Self::Del,
            "details" => Self::Details,
            "dfn" => Self::Dfn,
            "dialog" => Self::Dialog,
            "div" => Self::Div,
            "dl" => Self::Dl,
            "dt" => Self::Dt,
            "em" => Self::Em,
            "embed" => Self::Embed,
            "fieldset" => Self::Fieldset,
            "figcaption" => Self::Figcaption,
            "figure" => Self::Figure,
            "footer" => Self::Footer,
            "form" => Self::Form,
            "h1" => Self::H1,
            "h2" => Self::H2,
            "h3" => Self::H3,
            "h4" => Self::H4,
            "h5" => Self::H5,
            "h6" => Self::H6,
            "head" => Self::Head,
            "header" => Self::Header,
            "hgroup" => Self::Hgroup,
            "hr" => Self::Hr,
            "html" => Self::Html,
            "i" => Self::I,
            "iframe" => Self::Iframe,
            "img" => Self::Img,
            "input" => Self::Input,
            "ins" => Self::Ins,
            "kbd" => Self::Kbd,
            "label" => Self::Label,
            "legend" => Self::Legend,
            "li" => Self::Li,
            "link" => Self::Link,
            "main" => Self::Main,
            "map" => Self::Map,
            "mark" => Self::Mark,
            "menu" => Self::Menu,
            "meta" => Self::Meta,
            "meter" => Self::Meter,
            "nav" => Self::Nav,
            "noscript" => Self::Noscript,
            "object" => Self::Object,
            "ol" => Self::Ol,
            "optgroup" => Self::Optgroup,
            "option" => Self::Option,
            "output" => Self::Output,
            "p" => Self::P,
            "param" => Self::Param,
            "picture" => Self::Picture,
            "pre" => Self::Pre,
            "progress" => Self::Progress,
            "q" => Self::Q,
            "rp" => Self::Rp,
            "rt" => Self::Rt,
            "ruby" => Self::Ruby,
            "s" => Self::S,
            "samp" => Self::Samp,
            "script" => Self::Script,
            "section" => Self::Section,
            "select" => Self::Select,
            "slot" => Self::Slot,
            "small" => Self::Small,
            "source" => Self::Source,
            "span" => Self::Span,
            "strong" => Self::Strong,
            "style" => Self::Style,
            "sub" => Self::Sub,
            "summary" => Self::Summary,
            "sup" => Self::Sup,
            "table" => Self::Table,
            "tbody" => Self::Tbody,
            "td" => Self::Td,
            "template" => Self::Template,
            "textarea" => Self::Textarea,
            "tfoot" => Self::Tfoot,
            "th" => Self::Th,
            "thead" => Self::Thead,
            "time" => Self::Time,
            "title" => Self::Title,
            "tr" => Self::Tr,
            "track" => Self::Track,
            "u" => Self::U,
            "ul" => Self::Ul,
            "var" => Self::Var,
            "video" => Self::Video,
            "wbr" => Self::Wbr,
            _ => Self::Unknown,
        }
    }

    /// Returns the canonical tag name.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub const fn as_str(self) -> Option<&'static str> {
        match self {
            Self::Unknown => None,
            Self::A => Some("a"),
            Self::Abbr => Some("abbr"),
            Self::Address => Some("address"),
            Self::Area => Some("area"),
            Self::Article => Some("article"),
            Self::Aside => Some("aside"),
            Self::Audio => Some("audio"),
            Self::B => Some("b"),
            Self::Base => Some("base"),
            Self::Bdi => Some("bdi"),
            Self::Bdo => Some("bdo"),
            Self::Blockquote => Some("blockquote"),
            Self::Body => Some("body"),
            Self::Br => Some("br"),
            Self::Button => Some("button"),
            Self::Canvas => Some("canvas"),
            Self::Caption => Some("caption"),
            Self::Cite => Some("cite"),
            Self::Code => Some("code"),
            Self::Col => Some("col"),
            Self::Colgroup => Some("colgroup"),
            Self::Data => Some("data"),
            Self::Datalist => Some("datalist"),
            Self::Dd => Some("dd"),
            Self::Del => Some("del"),
            Self::Details => Some("details"),
            Self::Dfn => Some("dfn"),
            Self::Dialog => Some("dialog"),
            Self::Div => Some("div"),
            Self::Dl => Some("dl"),
            Self::Dt => Some("dt"),
            Self::Em => Some("em"),
            Self::Embed => Some("embed"),
            Self::Fieldset => Some("fieldset"),
            Self::Figcaption => Some("figcaption"),
            Self::Figure => Some("figure"),
            Self::Footer => Some("footer"),
            Self::Form => Some("form"),
            Self::H1 => Some("h1"),
            Self::H2 => Some("h2"),
            Self::H3 => Some("h3"),
            Self::H4 => Some("h4"),
            Self::H5 => Some("h5"),
            Self::H6 => Some("h6"),
            Self::Head => Some("head"),
            Self::Header => Some("header"),
            Self::Hgroup => Some("hgroup"),
            Self::Hr => Some("hr"),
            Self::Html => Some("html"),
            Self::I => Some("i"),
            Self::Iframe => Some("iframe"),
            Self::Img => Some("img"),
            Self::Input => Some("input"),
            Self::Ins => Some("ins"),
            Self::Kbd => Some("kbd"),
            Self::Label => Some("label"),
            Self::Legend => Some("legend"),
            Self::Li => Some("li"),
            Self::Link => Some("link"),
            Self::Main => Some("main"),
            Self::Map => Some("map"),
            Self::Mark => Some("mark"),
            Self::Menu => Some("menu"),
            Self::Meta => Some("meta"),
            Self::Meter => Some("meter"),
            Self::Nav => Some("nav"),
            Self::Noscript => Some("noscript"),
            Self::Object => Some("object"),
            Self::Ol => Some("ol"),
            Self::Optgroup => Some("optgroup"),
            Self::Option => Some("option"),
            Self::Output => Some("output"),
            Self::P => Some("p"),
            Self::Param => Some("param"),
            Self::Picture => Some("picture"),
            Self::Pre => Some("pre"),
            Self::Progress => Some("progress"),
            Self::Q => Some("q"),
            Self::Rp => Some("rp"),
            Self::Rt => Some("rt"),
            Self::Ruby => Some("ruby"),
            Self::S => Some("s"),
            Self::Samp => Some("samp"),
            Self::Script => Some("script"),
            Self::Section => Some("section"),
            Self::Select => Some("select"),
            Self::Slot => Some("slot"),
            Self::Small => Some("small"),
            Self::Source => Some("source"),
            Self::Span => Some("span"),
            Self::Strong => Some("strong"),
            Self::Style => Some("style"),
            Self::Sub => Some("sub"),
            Self::Summary => Some("summary"),
            Self::Sup => Some("sup"),
            Self::Table => Some("table"),
            Self::Tbody => Some("tbody"),
            Self::Td => Some("td"),
            Self::Template => Some("template"),
            Self::Textarea => Some("textarea"),
            Self::Tfoot => Some("tfoot"),
            Self::Th => Some("th"),
            Self::Thead => Some("thead"),
            Self::Time => Some("time"),
            Self::Title => Some("title"),
            Self::Tr => Some("tr"),
            Self::Track => Some("track"),
            Self::U => Some("u"),
            Self::Ul => Some("ul"),
            Self::Var => Some("var"),
            Self::Video => Some("video"),
            Self::Wbr => Some("wbr"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_id_interning() {
        assert_eq!(TagId::from_name("div"), TagId::Div);
        assert_eq!(TagId::from_name("span"), TagId::Span);
        assert_eq!(TagId::from_name("a"), TagId::A);
        assert_eq!(TagId::from_name("custom"), TagId::Unknown);
        assert_eq!(TagId::from_name("my-element"), TagId::Unknown);
    }

    #[test]
    fn test_tag_id_as_str() {
        assert_eq!(TagId::Div.as_str(), Some("div"));
        assert_eq!(TagId::Span.as_str(), Some("span"));
        assert_eq!(TagId::A.as_str(), Some("a"));
        assert_eq!(TagId::Unknown.as_str(), None);
    }

    #[test]
    fn test_tag_id_roundtrip() {
        let tags = ["div", "span", "a", "p", "ul", "li", "table", "tr", "td"];
        for &tag in &tags {
            let id = TagId::from_name(tag);
            assert_eq!(id.as_str(), Some(tag));
        }
    }

    #[test]
    fn test_tag_id_unknown_roundtrip() {
        let id = TagId::from_name("custom-element");
        assert_eq!(id, TagId::Unknown);
        assert_eq!(id.as_str(), None);
    }

    #[test]
    fn test_tag_id_equality() {
        assert_eq!(TagId::Div, TagId::Div);
        assert_ne!(TagId::Div, TagId::Span);
        assert_eq!(TagId::Unknown, TagId::Unknown);
    }

    #[test]
    fn test_tag_id_case_sensitivity() {
        assert_eq!(TagId::from_name("div"), TagId::Div);
        assert_eq!(TagId::from_name("DIV"), TagId::Unknown);
        assert_eq!(TagId::from_name("Div"), TagId::Unknown);
        assert_eq!(TagId::from_name("dIv"), TagId::Unknown);
    }

    #[test]
    fn test_tag_id_memory_size() {
        use std::mem::size_of;
        assert_eq!(size_of::<TagId>(), 1);
    }

    #[test]
    fn test_tag_id_all_standard_tags() {
        let standard_tags = [
            ("a", TagId::A),
            ("div", TagId::Div),
            ("span", TagId::Span),
            ("p", TagId::P),
            ("h1", TagId::H1),
            ("h2", TagId::H2),
            ("ul", TagId::Ul),
            ("li", TagId::Li),
            ("table", TagId::Table),
            ("tr", TagId::Tr),
            ("td", TagId::Td),
            ("body", TagId::Body),
            ("html", TagId::Html),
            ("head", TagId::Head),
        ];

        for (name, expected_id) in standard_tags {
            assert_eq!(TagId::from_name(name), expected_id);
            assert_eq!(expected_id.as_str(), Some(name));
        }
    }
}
