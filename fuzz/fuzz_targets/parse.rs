//! Fuzzing target for HTML parsing.
//!
//! This target exercises the HTML parser with random input to find edge cases
//! and potential crashes.

#![no_main]

use libfuzzer_sys::fuzz_target;
use scrape_core::Soup;

fuzz_target!(|data: &[u8]| {
    if let Ok(html) = std::str::from_utf8(data) {
        // Parse with default config
        let soup = Soup::parse(html);

        // Exercise query methods
        let _ = soup.find("div");
        let _ = soup.find_all("p");
        let _ = soup.select("a[href]");

        // Exercise navigation and extraction
        if let Ok(Some(tag)) = soup.find("body") {
            let _ = tag.parent();
            let _ = tag.children().count();
            let _ = tag.text();
            let _ = tag.inner_html();
        }

        // Parse with different configs
        let config = scrape_core::SoupConfig::builder().strict_mode(true).build();
        let _ = Soup::parse_with_config(html, config);

        let config = scrape_core::SoupConfig::builder().max_depth(32).build();
        let _ = Soup::parse_with_config(html, config);

        let config = scrape_core::SoupConfig::builder()
            .preserve_whitespace(true)
            .include_comments(true)
            .build();
        let _ = Soup::parse_with_config(html, config);
    }
});
