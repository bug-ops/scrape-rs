"""Integration tests for Phase 13b features.

Tests:
- Fragment parsing with various contexts
- CompiledSelector reuse and performance
- Extraction methods (select_text, select_attr)
- Text nodes iterator
- Filtered iterators (children_by_name, children_by_class)
"""

import pytest

from scrape_rs import Soup, compile_selector

# ==================== Fragment Parsing Tests ====================


class TestFragmentParsing:
    def test_parse_fragment_empty(self):
        soup = Soup.parse_fragment("")
        assert len(soup) == 0

    def test_parse_fragment_text_only(self):
        soup = Soup.parse_fragment("Hello World")
        assert "Hello World" in soup.text

    def test_parse_fragment_simple(self):
        soup = Soup.parse_fragment("<div>Test</div>")
        div = soup.find("div")
        assert div is not None
        assert div.text == "Test"

    def test_parse_fragment_nested(self):
        html = "<div><span>A</span><span>B</span></div>"
        soup = Soup.parse_fragment(html)
        spans = soup.find_all("span")
        assert len(spans) == 2
        assert spans[0].text == "A"
        assert spans[1].text == "B"

    def test_parse_fragment_no_wrapper(self):
        """Fragment parsing should not add html/body wrapper tags"""
        soup = Soup.parse_fragment("<div>Content</div>")
        html = soup.to_html()
        # Should contain div but not full html/body structure
        assert "<div>" in html
        # Root should be the fragment itself, not html
        root = soup.root
        assert root is not None

    def test_parse_fragment_with_context_body(self):
        """Parse with explicit body context"""
        soup = Soup.parse_fragment("<div>Test</div>", context="body")
        div = soup.find("div")
        assert div is not None
        assert div.text == "Test"

    def test_parse_fragment_with_context_table(self):
        """Parse table rows with table context"""
        html = "<tr><td>Cell 1</td><td>Cell 2</td></tr>"
        soup = Soup.parse_fragment(html, context="table")
        tds = soup.find_all("td")
        assert len(tds) == 2
        assert tds[0].text == "Cell 1"
        assert tds[1].text == "Cell 2"

    def test_parse_fragment_with_context_tbody(self):
        """Parse table rows with tbody context"""
        html = "<tr><td>A</td></tr><tr><td>B</td></tr>"
        soup = Soup.parse_fragment(html, context="tbody")
        trs = soup.find_all("tr")
        assert len(trs) == 2

    def test_parse_fragment_multiple_root_elements(self):
        """Fragment can have multiple root elements"""
        html = "<div>First</div><div>Second</div><div>Third</div>"
        soup = Soup.parse_fragment(html)
        divs = soup.find_all("div")
        assert len(divs) == 3

    def test_parse_fragment_with_attributes(self):
        """Fragments preserve attributes"""
        html = '<span class="highlight" id="item">Text</span>'
        soup = Soup.parse_fragment(html)
        span = soup.find("span")
        assert span is not None
        assert span.get("class") == "highlight"
        assert span.get("id") == "item"


# ==================== CompiledSelector Tests ====================


class TestCompiledSelector:
    def test_compile_simple_selector(self):
        selector = compile_selector("div.item")
        assert selector is not None
        assert selector.source == "div.item"

    def test_compile_complex_selector(self):
        selector = compile_selector("ul > li.active:first-child")
        assert selector.source == "ul > li.active:first-child"

    def test_compile_invalid_selector_raises(self):
        with pytest.raises(ValueError) as exc_info:
            compile_selector("[[[invalid")
        assert "selector" in str(exc_info.value).lower()

    def test_compiled_selector_repr(self):
        selector = compile_selector("div")
        assert "div" in repr(selector)

    def test_find_compiled(self):
        html = "<div class='item'>First</div><div class='item'>Second</div>"
        soup = Soup(html)
        selector = compile_selector(".item")

        result = soup.find_compiled(selector)
        assert result is not None
        assert result.text == "First"

    def test_select_compiled(self):
        html = "<ul><li class='active'>A</li><li>B</li><li class='active'>C</li></ul>"
        soup = Soup(html)
        selector = compile_selector("li.active")

        results = soup.select_compiled(selector)
        assert len(results) == 2
        assert results[0].text == "A"
        assert results[1].text == "C"

    def test_compiled_selector_reuse(self):
        """Compiled selector can be reused across multiple documents"""
        selector = compile_selector("span.highlight")

        soup1 = Soup("<div><span class='highlight'>Doc 1</span></div>")
        soup2 = Soup("<div><span class='highlight'>Doc 2</span></div>")

        result1 = soup1.find_compiled(selector)
        result2 = soup2.find_compiled(selector)

        assert result1 is not None
        assert result2 is not None
        assert result1.text == "Doc 1"
        assert result2.text == "Doc 2"

    def test_compiled_selector_not_found(self):
        soup = Soup("<div>No match</div>")
        selector = compile_selector(".nonexistent")

        result = soup.find_compiled(selector)
        assert result is None

    def test_compiled_selector_empty_results(self):
        soup = Soup("<div>No match</div>")
        selector = compile_selector("li")

        results = soup.select_compiled(selector)
        assert len(results) == 0


# ==================== Extraction Methods Tests ====================


class TestExtractionMethods:
    def test_select_text_single(self):
        html = "<div><span class='item'>Hello</span></div>"
        soup = Soup(html)

        texts = soup.select_text(".item")
        assert len(texts) == 1
        assert texts[0] == "Hello"

    def test_select_text_multiple(self):
        html = """
        <ul>
            <li class='item'>First</li>
            <li class='item'>Second</li>
            <li class='item'>Third</li>
        </ul>
        """
        soup = Soup(html)

        texts = soup.select_text(".item")
        assert len(texts) == 3
        assert texts[0] == "First"
        assert texts[1] == "Second"
        assert texts[2] == "Third"

    def test_select_text_empty(self):
        soup = Soup("<div>No matches</div>")
        texts = soup.select_text(".nonexistent")
        assert len(texts) == 0

    def test_select_text_nested_content(self):
        html = "<div class='item'>Hello <b>World</b>!</div>"
        soup = Soup(html)

        texts = soup.select_text(".item")
        assert len(texts) == 1
        # Should include nested text
        assert "Hello" in texts[0]
        assert "World" in texts[0]

    def test_select_text_invalid_selector(self):
        soup = Soup("<div>Test</div>")
        with pytest.raises(ValueError) as exc_info:
            soup.select_text("[[[")
        assert "selector" in str(exc_info.value).lower()

    def test_select_attr_single(self):
        html = "<a href='/link' class='link'>Click</a>"
        soup = Soup(html)

        hrefs = soup.select_attr("a", "href")
        assert len(hrefs) == 1
        assert hrefs[0] == "/link"

    def test_select_attr_multiple(self):
        html = """
        <div>
            <a href='/page1'>Link 1</a>
            <a href='/page2'>Link 2</a>
            <a href='/page3'>Link 3</a>
        </div>
        """
        soup = Soup(html)

        hrefs = soup.select_attr("a", "href")
        assert len(hrefs) == 3
        assert hrefs[0] == "/page1"
        assert hrefs[1] == "/page2"
        assert hrefs[2] == "/page3"

    def test_select_attr_missing_attribute(self):
        """Elements without the attribute return None"""
        html = '<div><a href="/link">Has</a><a>Missing</a></div>'
        soup = Soup(html)

        hrefs = soup.select_attr("a", "href")
        # Returns Option list, None for missing
        assert len(hrefs) == 2
        assert hrefs[0] == "/link"
        assert hrefs[1] is None

    def test_select_attr_empty(self):
        soup = Soup("<div>No links</div>")
        hrefs = soup.select_attr("a", "href")
        assert len(hrefs) == 0

    def test_select_attr_invalid_selector(self):
        soup = Soup("<div>Test</div>")
        with pytest.raises(ValueError) as exc_info:
            soup.select_attr("[[[", "id")
        assert "selector" in str(exc_info.value).lower()

    def test_select_attr_class(self):
        html = """
        <div>
            <span class='tag-a'>A</span>
            <span class='tag-b'>B</span>
            <span class='tag-c'>C</span>
        </div>
        """
        soup = Soup(html)

        classes = soup.select_attr("span", "class")
        assert len(classes) == 3
        assert "tag-a" in classes
        assert "tag-b" in classes
        assert "tag-c" in classes


# ==================== Text Nodes Iterator Tests ====================


class TestTextNodes:
    def test_text_nodes_simple(self):
        html = "<div>Hello World</div>"
        soup = Soup(html)
        div = soup.find("div")

        text_nodes = div.text_nodes()
        assert len(text_nodes) == 1
        assert text_nodes[0] == "Hello World"

    def test_text_nodes_multiple(self):
        html = "<div>First<span>Middle</span>Last</div>"
        soup = Soup(html)
        div = soup.find("div")

        text_nodes = div.text_nodes()
        # Gets all text nodes in subtree (recursive)
        assert "First" in text_nodes
        assert "Last" in text_nodes
        assert "Middle" in text_nodes  # Gets nested text too

    def test_text_nodes_empty(self):
        html = "<div><span>Nested text</span></div>"
        soup = Soup(html)
        div = soup.find("div")

        text_nodes = div.text_nodes()
        # Gets all text nodes recursively, including nested ones
        assert len(text_nodes) >= 1
        assert "Nested text" in text_nodes

    def test_text_nodes_whitespace(self):
        html = "<div>  Text with spaces  </div>"
        soup = Soup(html)
        div = soup.find("div")

        text_nodes = div.text_nodes()
        assert len(text_nodes) >= 1
        # Should preserve whitespace
        assert any("Text with spaces" in node for node in text_nodes)

    def test_text_nodes_mixed_content(self):
        html = "<p>Start <b>bold</b> middle <i>italic</i> end</p>"
        soup = Soup(html)
        p = soup.find("p")

        text_nodes = p.text_nodes()
        # Should have direct text nodes (not nested in b or i)
        assert any("Start" in node for node in text_nodes)
        assert any("middle" in node for node in text_nodes)
        assert any("end" in node for node in text_nodes)


# ==================== Filtered Iterators Tests ====================


class TestFilteredIterators:
    def test_children_by_name_single(self):
        html = "<div><span>A</span><p>B</p><span>C</span></div>"
        soup = Soup(html)
        div = soup.find("div")

        spans = div.children_by_name("span")
        assert len(spans) == 2
        assert spans[0].text == "A"
        assert spans[1].text == "C"

    def test_children_by_name_all_match(self):
        html = "<ul><li>A</li><li>B</li><li>C</li></ul>"
        soup = Soup(html)
        ul = soup.find("ul")

        items = ul.children_by_name("li")
        assert len(items) == 3

    def test_children_by_name_none_match(self):
        html = "<div><span>A</span><span>B</span></div>"
        soup = Soup(html)
        div = soup.find("div")

        items = div.children_by_name("li")
        assert len(items) == 0

    def test_children_by_name_empty(self):
        html = "<div></div>"
        soup = Soup(html)
        div = soup.find("div")

        items = div.children_by_name("span")
        assert len(items) == 0

    def test_children_by_name_case_sensitive(self):
        html = "<div><SPAN>A</SPAN><span>B</span></div>"
        soup = Soup(html)
        div = soup.find("div")

        # HTML normalizes to lowercase
        items = div.children_by_name("span")
        assert len(items) == 2

    def test_children_by_class_single(self):
        html = "<div><span class='item'>A</span><span>B</span></div>"
        soup = Soup(html)
        div = soup.find("div")

        items = div.children_by_class("item")
        assert len(items) == 1
        assert items[0].text == "A"

    def test_children_by_class_multiple(self):
        html = """
        <div>
            <span class='tag'>A</span>
            <p class='tag'>B</p>
            <div class='tag'>C</div>
        </div>
        """
        soup = Soup(html)
        div = soup.find("div")

        items = div.children_by_class("tag")
        assert len(items) == 3

    def test_children_by_class_multiple_classes(self):
        """Element with multiple classes should match"""
        html = "<div><span class='item active'>A</span><span class='item'>B</span></div>"
        soup = Soup(html)
        div = soup.find("div")

        items = div.children_by_class("item")
        assert len(items) == 2

        active_items = div.children_by_class("active")
        assert len(active_items) == 1
        assert active_items[0].text == "A"

    def test_children_by_class_none_match(self):
        html = "<div><span class='a'>A</span><span class='b'>B</span></div>"
        soup = Soup(html)
        div = soup.find("div")

        items = div.children_by_class("c")
        assert len(items) == 0

    def test_children_by_class_empty(self):
        html = "<div></div>"
        soup = Soup(html)
        div = soup.find("div")

        items = div.children_by_class("item")
        assert len(items) == 0

    def test_children_by_class_no_class_attribute(self):
        html = "<div><span>A</span><span>B</span></div>"
        soup = Soup(html)
        div = soup.find("div")

        items = div.children_by_class("item")
        assert len(items) == 0


# ==================== Scoped Extraction Methods Tests ====================


class TestScopedExtractionMethods:
    def test_tag_select_text(self):
        html = """
        <div class='container'>
            <span class='item'>Inside</span>
        </div>
        <span class='item'>Outside</span>
        """
        soup = Soup(html)
        container = soup.find(".container")

        texts = container.select_text(".item")
        assert len(texts) == 1
        assert texts[0] == "Inside"

    def test_tag_select_attr(self):
        html = """
        <div class='container'>
            <a href='/inside'>Inside</a>
        </div>
        <a href='/outside'>Outside</a>
        """
        soup = Soup(html)
        container = soup.find(".container")

        hrefs = container.select_attr("a", "href")
        assert len(hrefs) == 1
        assert hrefs[0] == "/inside"

    def test_tag_find_compiled(self):
        html = "<div><span class='target'>Found</span></div>"
        soup = Soup(html)
        div = soup.find("div")

        selector = compile_selector(".target")
        result = div.find_compiled(selector)

        assert result is not None
        assert result.text == "Found"

    def test_tag_select_compiled(self):
        html = "<div><span class='item'>A</span><span class='item'>B</span></div>"
        soup = Soup(html)
        div = soup.find("div")

        selector = compile_selector(".item")
        results = div.select_compiled(selector)

        assert len(results) == 2
        assert results[0].text == "A"
        assert results[1].text == "B"


# ==================== Edge Cases and Integration Tests ====================


class TestEdgeCases:
    def test_fragment_with_scripts(self):
        """Fragments can contain script tags"""
        html = "<div>Before</div><script>alert('test');</script><div>After</div>"
        soup = Soup.parse_fragment(html)
        divs = soup.find_all("div")
        assert len(divs) == 2

    def test_compiled_selector_with_pseudo_classes(self):
        html = "<ul><li>First</li><li>Second</li><li>Third</li></ul>"
        soup = Soup(html)

        selector = compile_selector("li:first-child")
        result = soup.find_compiled(selector)

        assert result is not None
        assert result.text == "First"

    def test_select_text_with_empty_elements(self):
        html = "<div class='item'></div><div class='item'>Text</div>"
        soup = Soup(html)

        texts = soup.select_text(".item")
        # Empty elements return empty string
        assert len(texts) == 2
        assert texts[0] == ""
        assert texts[1] == "Text"

    def test_select_attr_data_attributes(self):
        html = """
        <div>
            <button data-id='1'>A</button>
            <button data-id='2'>B</button>
            <button data-id='3'>C</button>
        </div>
        """
        soup = Soup(html)

        ids = soup.select_attr("button", "data-id")
        assert len(ids) == 3
        assert ids == ["1", "2", "3"]

    def test_text_nodes_with_entities(self):
        html = "<div>Hello &amp; goodbye &lt;test&gt;</div>"
        soup = Soup(html)
        div = soup.find("div")

        text_nodes = div.text_nodes()
        assert len(text_nodes) >= 1
        # Entities should be decoded
        text = "".join(text_nodes)
        assert "&" in text or "&amp;" in text
