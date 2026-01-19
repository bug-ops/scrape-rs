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


class TestParentsAndAncestors:
    """Test parents() and ancestors() methods from Phase 12."""

    def test_parents_returns_ancestor_list(self):
        html = "<html><body><div><span><a>link</a></span></div></body></html>"
        soup = Soup(html)
        link = soup.find("a")

        parents = link.parents
        assert len(parents) == 4  # span, div, body, html
        assert parents[0].name == "span"
        assert parents[1].name == "div"
        assert parents[2].name == "body"
        assert parents[3].name == "html"

    def test_ancestors_is_alias_for_parents(self):
        html = "<html><body><div><span>text</span></div></body></html>"
        soup = Soup(html)
        span = soup.find("span")

        parents = span.parents
        ancestors = span.ancestors

        assert len(parents) == len(ancestors)
        for p, a in zip(parents, ancestors, strict=True):
            assert p.name == a.name

    def test_parents_empty_for_root(self):
        html = "<html><body><div>text</div></body></html>"
        soup = Soup(html)
        root = soup.root

        assert len(root.parents) == 0

    def test_parents_partial_chain(self):
        html = '<div id="outer"><div id="middle"><div id="inner">text</div></div></div>'
        soup = Soup(html)
        inner = soup.find("#inner")

        parents = inner.parents
        assert len(parents) == 4  # middle, outer, body, html
        assert parents[0].get("id") == "middle"
        assert parents[1].get("id") == "outer"
        assert parents[2].name == "body"
        assert parents[3].name == "html"


class TestClosest:
    """Test closest() method from Phase 12."""

    def test_closest_finds_matching_ancestor(self):
        html = '<div class="outer"><div class="middle"><span>text</span></div></div>'
        soup = Soup(html)
        span = soup.find("span")

        result = span.closest(".outer")
        assert result is not None
        assert result.get("class") == "outer"

    def test_closest_finds_nearest_match(self):
        html = '<div class="target"><div class="target"><span>text</span></div></div>'
        soup = Soup(html)
        span = soup.find("span")

        result = span.closest(".target")
        assert result is not None
        # Should be the inner div (nearest)
        parent = span.parent
        assert result.outer_html == parent.outer_html

    def test_closest_returns_none_when_not_found(self):
        html = "<div><span>text</span></div>"
        soup = Soup(html)
        span = soup.find("span")

        result = span.closest(".nonexistent")
        assert result is None

    def test_closest_raises_on_invalid_selector(self):
        html = "<div><span>text</span></div>"
        soup = Soup(html)
        span = soup.find("span")

        with pytest.raises(ValueError):
            span.closest("[[[invalid")

    def test_closest_excludes_self(self):
        html = '<div class="target"><span class="target">text</span></div>'
        soup = Soup(html)
        span = soup.find("span")

        result = span.closest(".target")
        assert result is not None
        assert result.name == "div"  # Parent, not self


class TestNextSiblings:
    """Test next_siblings() method from Phase 12."""

    def test_next_siblings_returns_following_elements(self):
        html = '<div><span id="a">A</span><span id="b">B</span><span id="c">C</span></div>'
        soup = Soup(html)
        first = soup.find("#a")

        siblings = first.next_siblings
        assert len(siblings) == 2
        assert siblings[0].get("id") == "b"
        assert siblings[1].get("id") == "c"

    def test_next_siblings_empty_for_last(self):
        html = '<div><span id="a">A</span><span id="b">B</span></div>'
        soup = Soup(html)
        last = soup.find("#b")

        assert len(last.next_siblings) == 0

    def test_next_siblings_skips_text_nodes(self):
        html = '<div><span id="a">A</span>text<span id="b">B</span></div>'
        soup = Soup(html)
        first = soup.find("#a")

        siblings = first.next_siblings
        assert len(siblings) == 1
        assert siblings[0].get("id") == "b"


class TestPrevSiblings:
    """Test prev_siblings() method from Phase 12."""

    def test_prev_siblings_returns_preceding_elements(self):
        html = '<div><span id="a">A</span><span id="b">B</span><span id="c">C</span></div>'
        soup = Soup(html)
        last = soup.find("#c")

        siblings = last.prev_siblings
        assert len(siblings) == 2
        # Note: prev_siblings returns in reverse order
        assert siblings[0].get("id") == "b"
        assert siblings[1].get("id") == "a"

    def test_prev_siblings_empty_for_first(self):
        html = '<div><span id="a">A</span><span id="b">B</span></div>'
        soup = Soup(html)
        first = soup.find("#a")

        assert len(first.prev_siblings) == 0


class TestSiblings:
    """Test siblings() method from Phase 12."""

    def test_siblings_returns_all_except_self(self):
        html = '<div><span id="a">A</span><span id="b">B</span><span id="c">C</span></div>'
        soup = Soup(html)
        middle = soup.find("#b")

        siblings = middle.siblings
        assert len(siblings) == 2
        assert siblings[0].get("id") == "a"
        assert siblings[1].get("id") == "c"

    def test_siblings_empty_for_only_child(self):
        html = '<div><span id="only">text</span></div>'
        soup = Soup(html)
        only = soup.find("#only")

        assert len(only.siblings) == 0

    def test_siblings_skips_text_nodes(self):
        html = '<div>text1<span id="a">A</span>text2<span id="b">B</span>text3</div>'
        soup = Soup(html)
        first = soup.find("#a")

        siblings = first.siblings
        assert len(siblings) == 1
        assert siblings[0].get("id") == "b"
