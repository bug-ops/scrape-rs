"""Tests for Soup class."""

import pytest

from scrape_rs import Soup, SoupConfig


class TestSoupParsing:
    def test_parse_simple(self, simple_html):
        soup = Soup(simple_html)
        assert soup.root is not None

    def test_parse_with_config(self, simple_html):
        config = SoupConfig(max_depth=100, preserve_whitespace=True)
        soup = Soup(simple_html, config=config)
        assert soup.root is not None

    def test_parse_empty_returns_empty_soup(self):
        soup = Soup("")
        assert len(soup) == 0

    def test_config_repr(self):
        config = SoupConfig(max_depth=100)
        assert "max_depth=100" in repr(config)

    def test_config_all_fields(self):
        config = SoupConfig(
            max_depth=256, strict_mode=True, preserve_whitespace=True, include_comments=True
        )
        assert config.max_depth == 256
        assert config.strict_mode is True
        assert config.preserve_whitespace is True
        assert config.include_comments is True


class TestSoupProperties:
    def test_title(self, complex_soup):
        assert complex_soup.title == "Test Page"

    def test_title_missing(self, simple_soup):
        assert simple_soup.title is None

    def test_text_strips_tags(self, simple_soup):
        assert "Hello" in simple_soup.text
        assert "<div>" not in simple_soup.text

    def test_root_is_html(self, complex_soup):
        assert complex_soup.root.name == "html"

    def test_to_html(self, simple_soup):
        html = simple_soup.to_html()
        assert "<div>" in html
        assert "</div>" in html

    def test_len(self, simple_soup):
        assert len(simple_soup) > 0

    def test_repr(self, simple_soup):
        r = repr(simple_soup)
        assert "Soup(nodes=" in r

    def test_str(self, simple_soup):
        s = str(simple_soup)
        assert "<div>" in s


class TestSoupFind:
    def test_find_by_tag(self, complex_soup):
        div = complex_soup.find("div")
        assert div is not None
        assert div.name == "div"

    def test_find_by_class(self, complex_soup):
        item = complex_soup.find(".item")
        assert item is not None
        assert "First" in item.text

    def test_find_by_id(self, complex_soup):
        main = complex_soup.find("#main")
        assert main is not None
        assert main.get("id") == "main"

    def test_find_compound_selector(self, complex_soup):
        active = complex_soup.find("li.item.active")
        assert active is not None
        assert "First" in active.text

    def test_find_descendant(self, complex_soup):
        li = complex_soup.find("ul li")
        assert li is not None

    def test_find_child_combinator(self, complex_soup):
        items = complex_soup.find_all("ul.list > li")
        assert len(items) == 3

    def test_find_attribute_selector(self, complex_soup):
        link = complex_soup.find("a[href='/link']")
        assert link is not None
        assert link.text == "Click me"

    def test_find_not_found_returns_none(self, complex_soup):
        result = complex_soup.find(".nonexistent")
        assert result is None

    def test_find_all_returns_list(self, complex_soup):
        items = complex_soup.find_all(".item")
        assert isinstance(items, list)
        assert len(items) == 3

    def test_select_alias(self, complex_soup):
        items1 = complex_soup.find_all(".item")
        items2 = complex_soup.select(".item")
        assert len(items1) == len(items2)


class TestSoupErrors:
    def test_invalid_selector_raises_value_error(self, simple_soup):
        with pytest.raises(ValueError) as exc_info:
            simple_soup.find("div[[[")
        assert "selector" in str(exc_info.value).lower()


class TestSoupFromFile:
    def test_from_file(self, tmp_path):
        html_file = tmp_path / "test.html"
        html_file.write_text("<div>Test</div>")

        soup = Soup.from_file(str(html_file))
        assert soup.find("div").text == "Test"

    def test_from_file_with_config(self, tmp_path):
        html_file = tmp_path / "test.html"
        html_file.write_text("<div>Test</div>")

        config = SoupConfig(max_depth=256)
        soup = Soup.from_file(str(html_file), config=config)
        assert soup.find("div") is not None

    def test_from_file_not_found_raises(self):
        with pytest.raises(ValueError) as exc_info:
            Soup.from_file("/nonexistent/path.html")
        assert "read" in str(exc_info.value).lower()
