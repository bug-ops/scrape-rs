//! Web tests for scrape-wasm.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_soup_new() {
    let soup = scrape_wasm::Soup::new("<html><body></body></html>");
    // Just verify it doesn't panic
    let _ = soup;
}

#[wasm_bindgen_test]
fn test_soup_config() {
    let config = scrape_wasm::SoupConfig::new();
    assert_eq!(config.max_depth(), 256);
    assert!(!config.strict_mode());

    let config = config.set_max_depth(128).set_strict_mode(true);
    assert_eq!(config.max_depth(), 128);
    assert!(config.strict_mode());
}

#[wasm_bindgen_test]
fn test_parse_batch() {
    let docs = vec!["<html></html>".to_string(), "<div>Hello</div>".to_string()];
    let results = scrape_wasm::parse_batch(docs);
    assert_eq!(results.len(), 2);
}
