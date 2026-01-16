"""Cross-platform integration tests for scrape_rs.

This module loads shared test cases from test_cases.json and executes them
against the Python binding. The same test cases are run by Rust, Node.js,
and WASM test runners to ensure API consistency across all platforms.
"""

import json
from pathlib import Path

import pytest

from scrape_rs import Soup

TEST_CASES_PATH = (
    Path(__file__).parent.parent.parent.parent / "tests" / "shared" / "test_cases.json"
)


def load_test_cases():
    """Load test cases from shared JSON file."""
    with open(TEST_CASES_PATH) as f:
        return json.load(f)


TEST_DATA = load_test_cases()


def get_all_test_cases():
    """Flatten test cases for parametrization."""
    cases = []
    for suite in TEST_DATA["test_suites"]:
        for case in suite["cases"]:
            cases.append((suite["name"], case))
    return cases


@pytest.fixture
def test_data():
    return TEST_DATA


class TestSharedVersion:
    def test_json_version(self, test_data):
        assert test_data["version"] == "1.0"


def run_find_assertion(soup, selector, expected, test_id):
    """Run a find assertion."""
    try:
        result = soup.find(selector)
    except ValueError as e:
        pytest.fail(f"[{test_id}] Selector '{selector}' failed with error: {e}")

    if expected.get("exists") is False:
        assert result is None, f"[{test_id}] Expected selector '{selector}' to find nothing"
        return

    if result is None and expected.get("exists") is not False:
        pytest.fail(f"[{test_id}] Expected selector '{selector}' to find element")

    if "text" in expected:
        assert result.text == expected["text"], (
            f"[{test_id}] Text mismatch for selector '{selector}'"
        )

    if "name" in expected:
        assert result.name == expected["name"], (
            f"[{test_id}] Tag name mismatch for selector '{selector}'"
        )

    if "attr" in expected:
        for key, value in expected["attr"].items():
            assert result.get(key) == value, f"[{test_id}] Attribute '{key}' mismatch"

    if "attr_missing" in expected:
        assert result.get(expected["attr_missing"]) is None, (
            f"[{test_id}] Expected attribute to be missing"
        )

    if "inner_html" in expected:
        assert result.inner_html == expected["inner_html"], f"[{test_id}] inner_html mismatch"

    if "has_class" in expected:
        for cls in expected["has_class"]:
            assert result.has_class(cls), f"[{test_id}] Expected element to have class '{cls}'"

    if "not_has_class" in expected:
        for cls in expected["not_has_class"]:
            assert not result.has_class(cls), (
                f"[{test_id}] Expected element to NOT have class '{cls}'"
            )


def run_find_all_assertion(soup, selector, expected, test_id):
    """Run a find_all assertion."""
    try:
        results = soup.find_all(selector)
    except ValueError as e:
        pytest.fail(f"[{test_id}] find_all('{selector}') failed with error: {e}")

    if "count" in expected:
        assert len(results) == expected["count"], (
            f"[{test_id}] Count mismatch for find_all('{selector}')"
        )


def run_find_then_assertion(soup, selector, chain, expected, test_id):
    """Run a find -> chain assertion."""
    tag = soup.find(selector)
    assert tag is not None, f"[{test_id}] find('{selector}') returned None"

    if chain == "parent":
        result = tag.parent
        if expected.get("exists") is False:
            assert result is None, f"[{test_id}] Expected parent to be None"
        else:
            assert result is not None, f"[{test_id}] Expected parent to exist"
            if "attr" in expected:
                for key, value in expected["attr"].items():
                    assert result.get(key) == value, (
                        f"[{test_id}] Parent attribute '{key}' mismatch"
                    )

    elif chain == "children":
        children = tag.children
        if "count" in expected:
            assert len(children) == expected["count"], f"[{test_id}] Children count mismatch"

    elif chain == "next_sibling":
        sibling = tag.next_sibling
        if expected.get("exists") is False:
            assert sibling is None, f"[{test_id}] Expected next_sibling to be None"
        else:
            assert sibling is not None, f"[{test_id}] Expected next_sibling to exist"
            if "attr" in expected:
                for key, value in expected["attr"].items():
                    assert sibling.get(key) == value, (
                        f"[{test_id}] next_sibling attribute '{key}' mismatch"
                    )

    elif chain == "prev_sibling":
        sibling = tag.prev_sibling
        if expected.get("exists") is False:
            assert sibling is None, f"[{test_id}] Expected prev_sibling to be None"
        else:
            assert sibling is not None, f"[{test_id}] Expected prev_sibling to exist"
            if "attr" in expected:
                for key, value in expected["attr"].items():
                    assert sibling.get(key) == value, (
                        f"[{test_id}] prev_sibling attribute '{key}' mismatch"
                    )

    elif chain == "descendants":
        descendants = tag.descendants
        if "count" in expected:
            assert len(descendants) == expected["count"], f"[{test_id}] Descendants count mismatch"
        if "min_count" in expected:
            assert len(descendants) >= expected["min_count"], (
                f"[{test_id}] Expected at least {expected['min_count']} descendants, got {len(descendants)}"
            )

    else:
        pytest.fail(f"[{test_id}] Unknown chain method: {chain}")


def run_text_assertion(soup, expected, test_id):
    """Run a text assertion."""
    text = soup.text
    if "contains" in expected:
        assert expected["contains"] in text, (
            f"[{test_id}] Expected text to contain '{expected['contains']}'"
        )


def run_title_assertion(soup, expected, test_id):
    """Run a title assertion."""
    title = soup.title
    if "equals" in expected:
        assert title == expected["equals"], f"[{test_id}] Title mismatch"
    if expected.get("is_null"):
        assert title is None, f"[{test_id}] Expected title to be None"


def run_scoped_find_assertion(soup, scope, selector, expected, test_id):
    """Run a scoped find assertion."""
    scope_tag = soup.find(scope)
    assert scope_tag is not None, f"[{test_id}] Scope selector '{scope}' returned None"

    result = scope_tag.find(selector)

    if expected.get("exists") is False:
        assert result is None, f"[{test_id}] Expected scoped selector to find nothing"
        return

    if result is None and expected.get("exists") is not False:
        pytest.fail(
            f"[{test_id}] Expected scoped selector '{selector}' within '{scope}' to find element"
        )

    if "text" in expected:
        assert result.text == expected["text"], f"[{test_id}] Scoped text mismatch"


def run_scoped_find_all_assertion(soup, scope, selector, expected, test_id):
    """Run a scoped find_all assertion."""
    scope_tag = soup.find(scope)
    assert scope_tag is not None, f"[{test_id}] Scope selector '{scope}' returned None"

    results = scope_tag.find_all(selector)

    if "count" in expected:
        assert len(results) == expected["count"], f"[{test_id}] Scoped find_all count mismatch"


def run_assertion(soup, assertion, test_id):
    """Run a single assertion based on method type."""
    method = assertion["method"]

    if method == "find":
        run_find_assertion(soup, assertion["selector"], assertion["expected"], test_id)
    elif method == "find_all":
        run_find_all_assertion(soup, assertion["selector"], assertion["expected"], test_id)
    elif method == "find_then":
        run_find_then_assertion(
            soup, assertion["selector"], assertion["chain"], assertion["expected"], test_id
        )
    elif method == "text":
        run_text_assertion(soup, assertion["expected"], test_id)
    elif method == "title":
        run_title_assertion(soup, assertion["expected"], test_id)
    elif method == "scoped_find":
        run_scoped_find_assertion(
            soup, assertion["scope"], assertion["selector"], assertion["expected"], test_id
        )
    elif method == "scoped_find_all":
        run_scoped_find_all_assertion(
            soup, assertion["scope"], assertion["selector"], assertion["expected"], test_id
        )
    else:
        pytest.fail(f"[{test_id}] Unknown assertion method: {method}")


@pytest.mark.parametrize(
    "suite_name,case",
    get_all_test_cases(),
    ids=lambda x: x["id"] if isinstance(x, dict) else x,
)
def test_shared_case(suite_name, case):  # noqa: ARG001
    """Run a single shared test case."""
    soup = Soup(case["input"])

    for assertion in case["assertions"]:
        run_assertion(soup, assertion, case["id"])
