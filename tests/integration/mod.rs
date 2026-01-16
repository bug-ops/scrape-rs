//! Cross-platform integration tests for scrape-rs.
//!
//! This module loads shared test cases from `test_cases.json` and executes them
//! against the Rust implementation. The same test cases are run by Python, Node.js,
//! and WASM test runners to ensure API consistency across all platforms.

use std::collections::HashMap;

use scrape_core::Soup;
use serde::Deserialize;

const TEST_CASES_JSON: &str = include_str!("../shared/test_cases.json");

#[derive(Debug, Deserialize)]
struct TestSuite {
    version: String,
    test_suites: Vec<Suite>,
}

#[derive(Debug, Deserialize)]
struct Suite {
    name: String,
    #[allow(dead_code)]
    description: String,
    cases: Vec<TestCase>,
}

#[derive(Debug, Deserialize)]
struct TestCase {
    id: String,
    #[allow(dead_code)]
    description: String,
    input: String,
    assertions: Vec<Assertion>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "method")]
enum Assertion {
    #[serde(rename = "find")]
    Find { selector: String, expected: FindExpected },
    #[serde(rename = "find_all")]
    FindAll { selector: String, expected: FindAllExpected },
    #[serde(rename = "find_then")]
    FindThen { selector: String, chain: String, expected: ChainExpected },
    #[serde(rename = "text")]
    Text { expected: TextExpected },
    #[serde(rename = "title")]
    Title { expected: TitleExpected },
    #[serde(rename = "scoped_find")]
    ScopedFind { scope: String, selector: String, expected: FindExpected },
    #[serde(rename = "scoped_find_all")]
    ScopedFindAll { scope: String, selector: String, expected: FindAllExpected },
}

#[derive(Debug, Deserialize)]
struct FindExpected {
    exists: Option<bool>,
    text: Option<String>,
    name: Option<String>,
    attr: Option<HashMap<String, String>>,
    attr_missing: Option<String>,
    inner_html: Option<String>,
    has_class: Option<Vec<String>>,
    not_has_class: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct FindAllExpected {
    count: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct ChainExpected {
    exists: Option<bool>,
    count: Option<usize>,
    min_count: Option<usize>,
    attr: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct TextExpected {
    contains: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TitleExpected {
    equals: Option<String>,
    is_null: Option<bool>,
}

fn load_test_suite() -> TestSuite {
    serde_json::from_str(TEST_CASES_JSON).expect("Failed to parse test_cases.json")
}

fn run_find_assertion(soup: &Soup, selector: &str, expected: &FindExpected, test_id: &str) {
    let result = soup.find(selector);

    match result {
        Ok(Some(tag)) => {
            assert!(
                expected.exists != Some(false),
                "[{test_id}] Expected selector '{selector}' to find nothing, but found element"
            );

            if let Some(expected_text) = &expected.text {
                assert_eq!(
                    tag.text(),
                    *expected_text,
                    "[{test_id}] Text mismatch for selector '{selector}'"
                );
            }

            if let Some(expected_name) = &expected.name {
                assert_eq!(
                    tag.name(),
                    Some(expected_name.as_str()),
                    "[{test_id}] Tag name mismatch for selector '{selector}'"
                );
            }

            if let Some(attrs) = &expected.attr {
                for (key, value) in attrs {
                    assert_eq!(
                        tag.get(key),
                        Some(value.as_str()),
                        "[{test_id}] Attribute '{key}' mismatch for selector '{selector}'"
                    );
                }
            }

            if let Some(missing_attr) = &expected.attr_missing {
                assert!(
                    tag.get(missing_attr).is_none(),
                    "[{test_id}] Expected attribute '{missing_attr}' to be missing for selector \
                     '{selector}'"
                );
            }

            if let Some(expected_inner_html) = &expected.inner_html {
                assert_eq!(
                    tag.inner_html(),
                    *expected_inner_html,
                    "[{test_id}] inner_html mismatch for selector '{selector}'"
                );
            }

            if let Some(classes) = &expected.has_class {
                for class in classes {
                    assert!(
                        tag.has_class(class),
                        "[{test_id}] Expected element to have class '{class}' for selector \
                         '{selector}'"
                    );
                }
            }

            if let Some(not_classes) = &expected.not_has_class {
                for class in not_classes {
                    assert!(
                        !tag.has_class(class),
                        "[{test_id}] Expected element to NOT have class '{class}' for selector \
                         '{selector}'"
                    );
                }
            }
        }
        Ok(None) => {
            assert!(
                expected.exists == Some(false),
                "[{test_id}] Expected selector '{selector}' to find element, but found nothing"
            );
        }
        Err(e) => {
            panic!("[{test_id}] Selector '{selector}' failed with error: {e}");
        }
    }
}

fn run_find_all_assertion(soup: &Soup, selector: &str, expected: &FindAllExpected, test_id: &str) {
    let result = soup.find_all(selector);

    match result {
        Ok(tags) => {
            if let Some(count) = expected.count {
                assert_eq!(
                    tags.len(),
                    count,
                    "[{test_id}] Count mismatch for find_all('{selector}')"
                );
            }
        }
        Err(e) => {
            panic!("[{test_id}] find_all('{selector}') failed with error: {e}");
        }
    }
}

fn run_find_then_assertion(
    soup: &Soup,
    selector: &str,
    chain: &str,
    expected: &ChainExpected,
    test_id: &str,
) {
    let tag = soup
        .find(selector)
        .unwrap_or_else(|e| panic!("[{test_id}] find('{selector}') failed: {e}"))
        .unwrap_or_else(|| panic!("[{test_id}] find('{selector}') returned None"));

    match chain {
        "parent" => {
            let parent = tag.parent();
            if expected.exists == Some(false) {
                assert!(parent.is_none(), "[{test_id}] Expected parent to be None");
            } else {
                let parent = parent.unwrap_or_else(|| {
                    panic!("[{test_id}] Expected parent to exist for '{selector}'")
                });
                if let Some(attrs) = &expected.attr {
                    for (key, value) in attrs {
                        assert_eq!(
                            parent.get(key),
                            Some(value.as_str()),
                            "[{test_id}] Parent attribute '{key}' mismatch"
                        );
                    }
                }
            }
        }
        "children" => {
            if let Some(count) = expected.count {
                assert_eq!(tag.children().count(), count, "[{test_id}] Children count mismatch");
            }
        }
        "next_sibling" => {
            let sibling = tag.next_sibling();
            if expected.exists == Some(false) {
                assert!(sibling.is_none(), "[{test_id}] Expected next_sibling to be None");
            } else {
                let sibling = sibling.unwrap_or_else(|| {
                    panic!("[{test_id}] Expected next_sibling to exist for '{selector}'")
                });
                if let Some(attrs) = &expected.attr {
                    for (key, value) in attrs {
                        assert_eq!(
                            sibling.get(key),
                            Some(value.as_str()),
                            "[{test_id}] next_sibling attribute '{key}' mismatch"
                        );
                    }
                }
            }
        }
        "prev_sibling" => {
            let sibling = tag.prev_sibling();
            if expected.exists == Some(false) {
                assert!(sibling.is_none(), "[{test_id}] Expected prev_sibling to be None");
            } else {
                let sibling = sibling.unwrap_or_else(|| {
                    panic!("[{test_id}] Expected prev_sibling to exist for '{selector}'")
                });
                if let Some(attrs) = &expected.attr {
                    for (key, value) in attrs {
                        assert_eq!(
                            sibling.get(key),
                            Some(value.as_str()),
                            "[{test_id}] prev_sibling attribute '{key}' mismatch"
                        );
                    }
                }
            }
        }
        "descendants" => {
            let descendants: Vec<_> = tag.descendants().collect();
            if let Some(count) = expected.count {
                assert_eq!(descendants.len(), count, "[{test_id}] Descendants count mismatch");
            }
            if let Some(min_count) = expected.min_count {
                assert!(
                    descendants.len() >= min_count,
                    "[{test_id}] Expected at least {min_count} descendants, got {}",
                    descendants.len()
                );
            }
        }
        _ => panic!("[{test_id}] Unknown chain method: {chain}"),
    }
}

fn run_text_assertion(soup: &Soup, expected: &TextExpected, test_id: &str) {
    let text = soup.text();
    if let Some(contains) = &expected.contains {
        assert!(
            text.contains(contains),
            "[{test_id}] Expected text to contain '{contains}', got '{text}'"
        );
    }
}

fn run_title_assertion(soup: &Soup, expected: &TitleExpected, test_id: &str) {
    let title = soup.title();
    if let Some(equals) = &expected.equals {
        assert_eq!(title.as_deref(), Some(equals.as_str()), "[{test_id}] Title mismatch");
    }
    if expected.is_null == Some(true) {
        assert!(title.is_none(), "[{test_id}] Expected title to be None");
    }
}

fn run_scoped_find_assertion(
    soup: &Soup,
    scope: &str,
    selector: &str,
    expected: &FindExpected,
    test_id: &str,
) {
    let scope_tag = soup
        .find(scope)
        .unwrap_or_else(|e| panic!("[{test_id}] find('{scope}') failed: {e}"))
        .unwrap_or_else(|| panic!("[{test_id}] Scope selector '{scope}' returned None"));

    let result = scope_tag.find(selector);

    match result {
        Ok(Some(tag)) => {
            assert!(
                expected.exists != Some(false),
                "[{test_id}] Expected scoped selector '{selector}' within '{scope}' to find \
                 nothing"
            );
            if let Some(expected_text) = &expected.text {
                assert_eq!(tag.text(), *expected_text, "[{test_id}] Scoped text mismatch");
            }
        }
        Ok(None) => {
            assert!(
                expected.exists == Some(false),
                "[{test_id}] Expected scoped selector '{selector}' within '{scope}' to find \
                 element"
            );
        }
        Err(e) => {
            panic!("[{test_id}] Scoped find('{selector}') failed with error: {e}");
        }
    }
}

fn run_scoped_find_all_assertion(
    soup: &Soup,
    scope: &str,
    selector: &str,
    expected: &FindAllExpected,
    test_id: &str,
) {
    let scope_tag = soup
        .find(scope)
        .unwrap_or_else(|e| panic!("[{test_id}] find('{scope}') failed: {e}"))
        .unwrap_or_else(|| panic!("[{test_id}] Scope selector '{scope}' returned None"));

    let result = scope_tag.find_all(selector);

    match result {
        Ok(tags) => {
            if let Some(count) = expected.count {
                assert_eq!(tags.len(), count, "[{test_id}] Scoped find_all count mismatch");
            }
        }
        Err(e) => {
            panic!("[{test_id}] Scoped find_all('{selector}') failed with error: {e}");
        }
    }
}

fn run_test_case(case: &TestCase) {
    let soup = Soup::parse(&case.input);

    for assertion in &case.assertions {
        match assertion {
            Assertion::Find { selector, expected } => {
                run_find_assertion(&soup, selector, expected, &case.id);
            }
            Assertion::FindAll { selector, expected } => {
                run_find_all_assertion(&soup, selector, expected, &case.id);
            }
            Assertion::FindThen { selector, chain, expected } => {
                run_find_then_assertion(&soup, selector, chain, expected, &case.id);
            }
            Assertion::Text { expected } => {
                run_text_assertion(&soup, expected, &case.id);
            }
            Assertion::Title { expected } => {
                run_title_assertion(&soup, expected, &case.id);
            }
            Assertion::ScopedFind { scope, selector, expected } => {
                run_scoped_find_assertion(&soup, scope, selector, expected, &case.id);
            }
            Assertion::ScopedFindAll { scope, selector, expected } => {
                run_scoped_find_all_assertion(&soup, scope, selector, expected, &case.id);
            }
        }
    }
}

#[test]
fn test_json_version() {
    let suite = load_test_suite();
    assert_eq!(suite.version, "1.0");
}

#[test]
fn test_parsing_suite() {
    let suite = load_test_suite();
    let parsing =
        suite.test_suites.iter().find(|s| s.name == "parsing").expect("parsing suite not found");

    for case in &parsing.cases {
        run_test_case(case);
    }
}

#[test]
fn test_selectors_suite() {
    let suite = load_test_suite();
    let selectors = suite
        .test_suites
        .iter()
        .find(|s| s.name == "selectors")
        .expect("selectors suite not found");

    for case in &selectors.cases {
        run_test_case(case);
    }
}

#[test]
fn test_navigation_suite() {
    let suite = load_test_suite();
    let navigation = suite
        .test_suites
        .iter()
        .find(|s| s.name == "navigation")
        .expect("navigation suite not found");

    for case in &navigation.cases {
        run_test_case(case);
    }
}

#[test]
fn test_content_extraction_suite() {
    let suite = load_test_suite();
    let content = suite
        .test_suites
        .iter()
        .find(|s| s.name == "content_extraction")
        .expect("content_extraction suite not found");

    for case in &content.cases {
        run_test_case(case);
    }
}

#[test]
fn test_scoped_queries_suite() {
    let suite = load_test_suite();
    let scoped = suite
        .test_suites
        .iter()
        .find(|s| s.name == "scoped_queries")
        .expect("scoped_queries suite not found");

    for case in &scoped.cases {
        run_test_case(case);
    }
}
