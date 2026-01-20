//! CLI integration tests.
#![allow(missing_docs)]

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn scrape() -> Command {
    #[allow(deprecated)]
    Command::cargo_bin("scrape").unwrap()
}

#[test]
fn test_basic_extraction() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.html");
    fs::write(&file, "<html><body><h1>Hello World</h1></body></html>").unwrap();

    scrape().arg("h1").arg(&file).assert().success().stdout("Hello World\n");
}

#[test]
fn test_stdin_input() {
    scrape().arg("h1").write_stdin("<h1>From Stdin</h1>").assert().success().stdout("From Stdin\n");
}

#[test]
fn test_json_output() {
    scrape()
        .args(["-o", "json", "h1"])
        .write_stdin("<h1>Test</h1>")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"text\":\"Test\""));
}

#[test]
fn test_attribute_extraction() {
    scrape()
        .args(["-a", "href", "a"])
        .write_stdin("<a href=\"/page\">Link</a>")
        .assert()
        .success()
        .stdout("/page\n");
}

#[test]
fn test_named_selectors() {
    scrape()
        .args(["-o", "json", "-s", "title=h1", "-s", "link=a"])
        .write_stdin("<h1>Title</h1><a href=\"/\">Link</a>")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\""));
}

#[test]
fn test_first_only() {
    scrape()
        .args(["-1", "p"])
        .write_stdin("<p>First</p><p>Second</p>")
        .assert()
        .success()
        .stdout("First\n");
}

#[test]
fn test_invalid_selector() {
    scrape()
        .arg("div[[[")
        .write_stdin("<div>Test</div>")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("selector"));
}

#[test]
fn test_no_matches() {
    scrape().arg(".nonexistent").write_stdin("<div>Test</div>").assert().code(1);
}

#[test]
fn test_multiple_files() {
    let dir = TempDir::new().unwrap();
    let path_a = dir.path().join("a.html");
    let path_b = dir.path().join("b.html");
    fs::write(&path_a, "<h1>File A</h1>").unwrap();
    fs::write(&path_b, "<h1>File B</h1>").unwrap();

    scrape()
        .arg("h1")
        .arg(&path_a)
        .arg(&path_b)
        .assert()
        .success()
        .stdout(predicate::str::contains("a.html"))
        .stdout(predicate::str::contains("File A"))
        .stdout(predicate::str::contains("b.html"))
        .stdout(predicate::str::contains("File B"));
}

#[test]
fn test_null_delimiter() {
    scrape().args(["-0", "p"]).write_stdin("<p>A</p><p>B</p>").assert().success().stdout("A\0B\0");
}

#[test]
fn test_html_output() {
    scrape()
        .args(["-o", "html", "span"])
        .write_stdin("<span class=\"test\">Hello</span>")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"));
}

#[test]
fn test_csv_named_selectors() {
    scrape()
        .args(["-o", "csv", "-s", "name=td:first-child", "-s", "value=td:last-child"])
        .write_stdin("<table><tr><td>A</td><td>1</td></tr></table>")
        .assert()
        .success()
        .stdout(predicate::str::contains("name"));
}

#[test]
fn test_pretty_json() {
    scrape()
        .args(["-o", "json", "-p", "h1"])
        .write_stdin("<h1>Test</h1>")
        .assert()
        .success()
        .stdout(predicate::str::contains("\n"));
}

#[test]
fn test_missing_selector() {
    scrape().assert().code(4).stderr(predicate::str::contains("SELECTOR"));
}

#[test]
fn test_conflicting_selector_and_select() {
    scrape()
        .args(["h1", "-s", "title=h1"])
        .write_stdin("<h1>Test</h1>")
        .assert()
        .code(4)
        .stderr(predicate::str::contains("Cannot use both"));
}

#[test]
fn test_csv_requires_named_selectors() {
    scrape()
        .args(["-o", "csv", "h1"])
        .write_stdin("<h1>Test</h1>")
        .assert()
        .code(4)
        .stderr(predicate::str::contains("CSV"));
}

#[test]
fn test_quiet_mode() {
    scrape()
        .args(["-q", "div[[["])
        .write_stdin("<div>Test</div>")
        .assert()
        .code(2)
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_no_filename_flag() {
    let dir = TempDir::new().unwrap();
    let path_a = dir.path().join("a.html");
    let path_b = dir.path().join("b.html");
    fs::write(&path_a, "<h1>File A</h1>").unwrap();
    fs::write(&path_b, "<h1>File B</h1>").unwrap();

    scrape()
        .arg("--no-filename")
        .arg("h1")
        .arg(&path_a)
        .arg(&path_b)
        .assert()
        .success()
        .stdout(predicate::str::contains("File A"))
        .stdout(predicate::str::contains("File B"))
        .stdout(predicate::str::contains("a.html").not());
}

#[test]
fn test_with_filename_flag() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.html");
    fs::write(&file, "<h1>Test</h1>").unwrap();

    scrape()
        .arg("-H")
        .arg("h1")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("test.html"));
}

#[test]
fn test_multiple_matches() {
    scrape()
        .arg("li")
        .write_stdin("<ul><li>One</li><li>Two</li><li>Three</li></ul>")
        .assert()
        .success()
        .stdout("One\nTwo\nThree\n");
}

#[test]
fn test_complex_selector() {
    scrape()
        .arg("div.container > ul#list > li.item")
        .write_stdin(
            r#"<div class="container"><ul id="list"><li class="item">Match</li></ul></div>"#,
        )
        .assert()
        .success()
        .stdout("Match\n");
}

#[test]
fn test_nonexistent_file() {
    scrape()
        .arg("h1")
        .arg("/nonexistent/file.html")
        .assert()
        .code(1)
        .stderr(predicate::str::contains("nonexistent"));
}

// Phase 19 Tests - Selector Explanation

#[test]
fn test_explain_simple_selector() {
    scrape()
        .args(["--explain", "div.container"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Selector: div.container"))
        .stdout(predicate::str::contains("Specificity:"));
}

#[test]
fn test_explain_complex_selector() {
    scrape()
        .args(["--explain", "#main > ul.nav li a[href]"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Selector:"))
        .stdout(predicate::str::contains("Specificity:"));
}

#[test]
fn test_explain_invalid_selector() {
    scrape()
        .args(["--explain", "div[[["])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Invalid selector"));
}

#[test]
fn test_explain_requires_selector() {
    scrape().args(["--explain"]).assert().code(2).stderr(predicate::str::contains("requires"));
}

#[test]
fn test_explain_with_attribute_selector() {
    scrape()
        .args(["--explain", "a[href^='https']"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Selector:"));
}

#[test]
fn test_explain_with_pseudo_class() {
    scrape()
        .args(["--explain", "li:first-child"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Selector:"));
}

#[test]
fn test_explain_universal_selector() {
    scrape()
        .args(["--explain", "*"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Selector:"));
}

#[test]
fn test_explain_descendant_combinator() {
    scrape()
        .args(["--explain", "div span"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Selector:"));
}

#[test]
fn test_explain_child_combinator() {
    scrape()
        .args(["--explain", "ul > li"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Selector:"));
}

#[test]
fn test_explain_adjacent_sibling() {
    scrape()
        .args(["--explain", "h1 + p"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Selector:"));
}

#[test]
fn test_explain_multiple_classes() {
    scrape()
        .args(["--explain", ".class1.class2.class3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Selector:"));
}

// Edge case tests for complex selectors

#[test]
fn test_deeply_nested_selector() {
    scrape()
        .arg("html > body > div > main > section > article > p > span")
        .write_stdin(
            "<html><body><div><main><section><article><p><span>Deep</span></p></article></\
             section></main></div></body></html>",
        )
        .assert()
        .success()
        .stdout("Deep\n");
}

#[test]
fn test_selector_with_unicode_content() {
    scrape()
        .arg("p.emoji")
        .write_stdin(r#"<p class="emoji">🚀 Unicode Test 你好</p>"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("🚀 Unicode Test 你好"));
}

#[test]
fn test_empty_selector_error() {
    scrape().arg("").write_stdin("<div>Test</div>").assert().code(2);
}

#[test]
fn test_malformed_html_with_warnings() {
    scrape()
        .arg("p")
        .write_stdin("<html><body><p>Unclosed paragraph<div>Nested div</div>")
        .assert()
        .success()
        .stdout(predicate::str::contains("Unclosed paragraph"));
}

#[test]
fn test_large_document_performance() {
    use std::fmt::Write;
    let items = (0..1000).fold(String::new(), |mut acc, i| {
        let _ = write!(acc, "<div class='item{i}'>Item {i}</div>");
        acc
    });
    let large_html = format!("<html><body>{items}</body></html>");
    scrape().arg("div").write_stdin(large_html).assert().success();
}

#[test]
fn test_very_large_document() {
    use std::fmt::Write;
    let items = (0..10000).fold(String::new(), |mut acc, i| {
        let _ = write!(acc, "<p id='p{i}'>Para {i}</p>");
        acc
    });
    let very_large = format!("<html><body>{items}</body></html>");
    scrape().arg("p").write_stdin(very_large).assert().success();
}
