"""Tests for tree navigation."""

import pytest

from scrape_rs import Soup


@pytest.fixture
def nav_soup():
    return Soup("""
    <html>
    <body>
        <div id="container">
            <span id="first">A</span>
            <span id="second">B</span>
            <span id="third">C</span>
        </div>
        <div id="footer">Footer</div>
    </body>
    </html>
    """)


class TestParentNavigation:
    def test_parent(self, nav_soup):
        span = nav_soup.find("#first")
        parent = span.parent
        assert parent.name == "div"
        assert parent.get("id") == "container"

    def test_parent_chain(self, nav_soup):
        span = nav_soup.find("#first")
        div = span.parent
        body = div.parent
        html = body.parent
        assert html.name == "html"

    def test_root_parent_is_none(self, nav_soup):
        root = nav_soup.root
        assert root.parent is None


class TestChildNavigation:
    def test_children(self, nav_soup):
        container = nav_soup.find("#container")
        children = container.children
        assert len(children) == 3
        assert all(c.name == "span" for c in children)

    def test_children_empty_for_leaf(self, nav_soup):
        span = nav_soup.find("#first")
        assert len(span.children) == 0


class TestSiblingNavigation:
    def test_next_sibling(self, nav_soup):
        first = nav_soup.find("#first")
        second = first.next_sibling
        assert second.get("id") == "second"

    def test_prev_sibling(self, nav_soup):
        second = nav_soup.find("#second")
        first = second.prev_sibling
        assert first.get("id") == "first"

    def test_last_has_no_next(self, nav_soup):
        third = nav_soup.find("#third")
        assert third.next_sibling is None

    def test_first_has_no_prev(self, nav_soup):
        first = nav_soup.find("#first")
        assert first.prev_sibling is None

    def test_sibling_chain(self, nav_soup):
        first = nav_soup.find("#first")
        second = first.next_sibling
        third = second.next_sibling
        assert third.get("id") == "third"


class TestDescendants:
    def test_descendants(self, nav_soup):
        container = nav_soup.find("#container")
        descendants = container.descendants
        assert len(descendants) == 3

    def test_descendants_deep(self, nav_soup):
        body = nav_soup.find("body")
        descendants = body.descendants
        # container, 3 spans, footer (5 total minimum)
        assert len(descendants) >= 5


class TestScopedQueries:
    def test_find_within(self, nav_soup):
        container = nav_soup.find("#container")
        span = container.find("span")
        assert span.get("id") == "first"

    def test_find_all_within(self, nav_soup):
        container = nav_soup.find("#container")
        spans = container.find_all("span")
        assert len(spans) == 3

    def test_find_within_not_found_outside(self, nav_soup):
        container = nav_soup.find("#container")
        footer = container.find("#footer")
        assert footer is None  # footer is sibling, not descendant

    def test_select_alias_within(self, nav_soup):
        container = nav_soup.find("#container")
        spans1 = container.find_all("span")
        spans2 = container.select("span")
        assert len(spans1) == len(spans2)

    def test_find_within_invalid_selector_raises(self, nav_soup):
        container = nav_soup.find("#container")
        with pytest.raises(ValueError):
            container.find("[[[invalid")
