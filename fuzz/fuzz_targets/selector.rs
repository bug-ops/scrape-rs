//! Fuzzing target for CSS selector parsing.
//!
//! This target exercises the CSS selector parser with random input strings
//! against a stable HTML document. The goal is to ensure the selector parser
//! does not panic on any input.

#![no_main]

use libfuzzer_sys::fuzz_target;
use scrape_core::Soup;

const STABLE_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <title>Test Page</title>
</head>
<body>
    <header id="header" class="site-header">
        <nav class="main-nav" role="navigation">
            <a href="/" class="logo">Logo</a>
            <ul class="nav-menu">
                <li class="nav-item active"><a href="/home">Home</a></li>
                <li class="nav-item"><a href="/about">About</a></li>
                <li class="nav-item"><a href="/contact">Contact</a></li>
            </ul>
        </nav>
    </header>
    <main id="content" class="main-content">
        <article class="post" data-id="1" data-category="tech">
            <h1 class="title">Article Title</h1>
            <p class="excerpt">Article content here.</p>
            <div class="tags">
                <span class="tag">tag1</span>
                <span class="tag">tag2</span>
            </div>
        </article>
        <aside class="sidebar">
            <section class="widget" id="recent-posts">
                <h2>Recent Posts</h2>
                <ul>
                    <li><a href="/post-1">Post 1</a></li>
                    <li><a href="/post-2">Post 2</a></li>
                </ul>
            </section>
        </aside>
    </main>
    <footer id="footer" class="site-footer">
        <p>&copy; 2026</p>
    </footer>
</body>
</html>
"#;

fuzz_target!(|selector: &str| {
    let soup = Soup::parse(STABLE_HTML);

    // These should not panic regardless of selector content
    let _ = soup.find(selector);
    let _ = soup.find_all(selector);
    let _ = soup.select(selector);

    // Also test scoped queries
    if let Ok(Some(main)) = soup.find("main") {
        let _ = main.find(selector);
        let _ = main.find_all(selector);
        let _ = main.select(selector);
    }
});
