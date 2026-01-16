"""Tests for Tag class."""

import pytest

from scrape_rs import Soup


class TestTagContent:
    @pytest.fixture
    def tag(self):
        soup = Soup('<div class="test" data-value="42">Hello <span>World</span></div>')
        return soup.find("div")

    def test_name(self, tag):
        assert tag.name == "div"

    def test_text(self, tag):
        assert "Hello" in tag.text
        assert "World" in tag.text

    def test_inner_html(self, tag):
        html = tag.inner_html
        assert "Hello" in html
        assert "<span>" in html
        assert "<div>" not in html

    def test_outer_html(self, tag):
        html = tag.outer_html
        assert "<div" in html
        assert "</div>" in html


class TestTagAttributes:
    @pytest.fixture
    def tag(self):
        soup = Soup('<a href="/page" class="link primary" disabled>Text</a>')
        return soup.find("a")

    def test_get(self, tag):
        assert tag.get("href") == "/page"

    def test_get_missing_returns_none(self, tag):
        assert tag.get("nonexistent") is None

    def test_has_attr(self, tag):
        assert tag.has_attr("href")
        assert tag.has_attr("disabled")
        assert not tag.has_attr("title")

    def test_attrs_dict(self, tag):
        attrs = tag.attrs
        assert attrs["href"] == "/page"
        assert attrs["class"] == "link primary"

    def test_has_class(self, tag):
        assert tag.has_class("link")
        assert tag.has_class("primary")
        assert not tag.has_class("secondary")

    def test_classes_list(self, tag):
        classes = tag.classes
        assert "link" in classes
        assert "primary" in classes


class TestTagPythonProtocol:
    @pytest.fixture
    def tag(self):
        soup = Soup('<div id="main" class="container">Content</div>')
        return soup.find("div")

    def test_getitem(self, tag):
        assert tag["id"] == "main"

    def test_getitem_missing_raises_key_error(self, tag):
        with pytest.raises(KeyError):
            _ = tag["nonexistent"]

    def test_contains(self, tag):
        assert "id" in tag
        assert "class" in tag
        assert "nonexistent" not in tag

    def test_len(self):
        soup = Soup("<ul><li>A</li><li>B</li><li>C</li></ul>")
        ul = soup.find("ul")
        assert len(ul) == 3

    def test_iter(self):
        soup = Soup("<ul><li>A</li><li>B</li><li>C</li></ul>")
        ul = soup.find("ul")
        children = list(ul)
        assert len(children) == 3
        assert all(c.name == "li" for c in children)

    def test_repr(self, tag):
        assert "Tag('div')" in repr(tag)

    def test_str(self, tag):
        s = str(tag)
        assert "<div" in s
        assert "</div>" in s

    def test_eq(self):
        soup = Soup("<div><span>A</span></div>")
        tag1 = soup.find("span")
        tag2 = soup.find("span")
        assert tag1 == tag2

    def test_eq_different_elements(self):
        soup = Soup("<div><span>A</span><span>B</span></div>")
        spans = soup.find_all("span")
        assert spans[0] != spans[1]

    def test_hash(self):
        soup = Soup("<div><span>A</span></div>")
        tag = soup.find("span")
        s = {tag}
        assert tag in s

    def test_hash_same_element_same_hash(self):
        soup = Soup("<div><span>A</span></div>")
        tag1 = soup.find("span")
        tag2 = soup.find("span")
        assert hash(tag1) == hash(tag2)
