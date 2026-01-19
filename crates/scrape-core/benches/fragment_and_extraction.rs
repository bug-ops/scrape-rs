//! Benchmarks for Phase 13b query enhancement features.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use scrape_core::{Soup, compile_selector};

/// Sample HTML fragments for benchmarking.
mod samples {
    use std::fmt::Write;

    pub const SIMPLE_FRAGMENT: &str = r"<div><span>Hello</span><span>World</span></div>";

    pub const MULTI_ROOT_FRAGMENT: &str = r"
        <div class='item'>Item 1</div>
        <div class='item'>Item 2</div>
        <div class='item'>Item 3</div>
    ";

    pub const TABLE_FRAGMENT: &str = r"
        <tr><td>A</td><td>B</td></tr>
        <tr><td>C</td><td>D</td></tr>
        <tr><td>E</td><td>F</td></tr>
    ";

    /// Generate HTML with many text nodes.
    pub fn generate_text_heavy(element_count: usize) -> String {
        let mut html = String::from("<html><body>");
        for i in 0..element_count {
            let _ = write!(html, r#"<p class="text-{i}">Text content {i}</p>"#);
        }
        html.push_str("</body></html>");
        html
    }

    /// Generate HTML with many attributes.
    pub fn generate_attr_heavy(element_count: usize) -> String {
        let mut html = String::from("<html><body>");
        for i in 0..element_count {
            let _ =
                write!(html, r#"<a href="/link-{i}" title="Link {i}" data-id="{i}">Link {i}</a>"#);
        }
        html.push_str("</body></html>");
        html
    }

    /// Generate nested HTML for text_nodes testing.
    pub fn generate_nested(depth: usize) -> String {
        let mut html = String::from("<html><body>");
        for _ in 0..depth {
            html.push_str("<div>Outer ");
        }
        html.push_str("Deep content");
        for _ in 0..depth {
            html.push_str(" Inner</div>");
        }
        html.push_str("</body></html>");
        html
    }
}

/// Benchmark fragment parsing vs full document parsing.
fn bench_fragment_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("fragment_parsing");

    // Simple fragment
    group.throughput(Throughput::Bytes(samples::SIMPLE_FRAGMENT.len() as u64));
    group.bench_function("parse_fragment/simple", |b| {
        b.iter(|| Soup::parse_fragment(black_box(samples::SIMPLE_FRAGMENT)));
    });

    // Full parse for comparison
    group.bench_function("parse/simple_fragment", |b| {
        b.iter(|| Soup::parse(black_box(samples::SIMPLE_FRAGMENT)));
    });

    // Multi-root fragment
    group.throughput(Throughput::Bytes(samples::MULTI_ROOT_FRAGMENT.len() as u64));
    group.bench_function("parse_fragment/multi_root", |b| {
        b.iter(|| Soup::parse_fragment(black_box(samples::MULTI_ROOT_FRAGMENT)));
    });

    // Table fragment with context
    group.throughput(Throughput::Bytes(samples::TABLE_FRAGMENT.len() as u64));
    group.bench_function("parse_fragment/table_context", |b| {
        b.iter(|| Soup::parse_fragment_with_context(black_box(samples::TABLE_FRAGMENT), "tbody"));
    });

    group.finish();
}

/// Benchmark CompiledSelector vs repeated string selectors.
fn bench_compiled_selector(c: &mut Criterion) {
    let mut group = c.benchmark_group("compiled_selector");

    let html = samples::generate_text_heavy(1000);
    let soup = Soup::parse(&html);

    // String selector (baseline)
    group.bench_function("string_selector/single_query", |b| {
        b.iter(|| {
            let _ = soup.find_all(black_box("p.text-500")).unwrap();
        });
    });

    // String selector repeated (current approach)
    group.bench_function("string_selector/10_queries", |b| {
        b.iter(|| {
            for _ in 0..10 {
                let _ = soup.find_all(black_box("p.text-500")).unwrap();
            }
        });
    });

    // Compiled selector (Phase 13b optimization)
    let compiled = compile_selector("p.text-500").unwrap();
    group.bench_function("compiled_selector/single_query", |b| {
        b.iter(|| {
            let _ = soup.select_compiled(black_box(&compiled));
        });
    });

    group.bench_function("compiled_selector/10_queries", |b| {
        b.iter(|| {
            for _ in 0..10 {
                let _ = soup.select_compiled(black_box(&compiled));
            }
        });
    });

    // More complex selector
    let complex_selector = "p.text-500[class*='text']";
    group.bench_function("string_selector_complex/10_queries", |b| {
        b.iter(|| {
            for _ in 0..10 {
                let _ = soup.find_all(black_box(complex_selector)).unwrap();
            }
        });
    });

    let compiled_complex = compile_selector(complex_selector).unwrap();
    group.bench_function("compiled_selector_complex/10_queries", |b| {
        b.iter(|| {
            for _ in 0..10 {
                let _ = soup.select_compiled(black_box(&compiled_complex));
            }
        });
    });

    group.finish();
}

/// Benchmark select_text vs manual iteration.
fn bench_select_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("select_text");

    for size in [10, 100, 1000] {
        let html = samples::generate_text_heavy(size);
        let soup = Soup::parse(&html);

        // select_text (Phase 13b)
        group.bench_with_input(BenchmarkId::new("select_text", size), &soup, |b, soup| {
            b.iter(|| {
                let _ = soup.select_text(black_box("p")).unwrap();
            });
        });

        // Manual approach (baseline)
        group.bench_with_input(BenchmarkId::new("manual_text", size), &soup, |b, soup| {
            b.iter(|| {
                let elements = soup.find_all(black_box("p")).unwrap();
                let texts: Vec<String> = elements.iter().map(scrape_core::Tag::text).collect();
                black_box(texts);
            });
        });
    }

    group.finish();
}

/// Benchmark select_attr vs manual iteration.
fn bench_select_attr(c: &mut Criterion) {
    let mut group = c.benchmark_group("select_attr");

    for size in [10, 100, 1000] {
        let html = samples::generate_attr_heavy(size);
        let soup = Soup::parse(&html);

        // select_attr (Phase 13b)
        group.bench_with_input(BenchmarkId::new("select_attr", size), &soup, |b, soup| {
            b.iter(|| {
                let _ = soup.select_attr(black_box("a"), black_box("href")).unwrap();
            });
        });

        // Manual approach (baseline)
        group.bench_with_input(BenchmarkId::new("manual_attr", size), &soup, |b, soup| {
            b.iter(|| {
                let elements = soup.find_all(black_box("a")).unwrap();
                let hrefs: Vec<Option<String>> =
                    elements.iter().map(|e| e.get(black_box("href")).map(String::from)).collect();
                black_box(hrefs);
            });
        });
    }

    group.finish();
}

/// Benchmark text_nodes vs text().
fn bench_text_nodes(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_nodes");

    // Simple nested structure
    let html = samples::generate_nested(5);
    let soup = Soup::parse(&html);
    let root = soup.find("body").unwrap().unwrap();

    // text_nodes (Phase 13b)
    group.bench_function("text_nodes/nested", |b| {
        b.iter(|| {
            let _ = black_box(root.text_nodes().collect::<Vec<_>>());
        });
    });

    // text() for comparison
    group.bench_function("text/nested", |b| {
        b.iter(|| {
            let _ = black_box(root.text());
        });
    });

    // Many shallow nodes
    let html = samples::generate_text_heavy(100);
    let soup = Soup::parse(&html);
    let root = soup.find("body").unwrap().unwrap();

    group.bench_function("text_nodes/shallow_many", |b| {
        b.iter(|| {
            let _ = black_box(root.text_nodes().collect::<Vec<_>>());
        });
    });

    group.finish();
}

/// Benchmark filtered iterators.
fn bench_filtered_iterators(c: &mut Criterion) {
    let mut group = c.benchmark_group("filtered_iterators");

    let html = r#"
        <ul>
            <li class="active">Item 1</li>
            <span>Not a list item</span>
            <li>Item 2</li>
            <div>Also not a list item</div>
            <li class="active">Item 3</li>
            <li>Item 4</li>
            <li class="active">Item 5</li>
        </ul>
    "#;
    let soup = Soup::parse(html);
    let ul = soup.find("ul").unwrap().unwrap();

    // children_by_name (Phase 13b)
    group.bench_function("children_by_name/li", |b| {
        b.iter(|| {
            let _ = black_box(ul.children_by_name("li").collect::<Vec<_>>());
        });
    });

    // Manual filter (baseline)
    group.bench_function("manual_filter/by_name", |b| {
        b.iter(|| {
            let filtered: Vec<_> = ul.children().filter(|c| c.name() == Some("li")).collect();
            black_box(filtered);
        });
    });

    // children_by_class (Phase 13b)
    group.bench_function("children_by_class/active", |b| {
        b.iter(|| {
            let _ = black_box(ul.children_by_class("active").collect::<Vec<_>>());
        });
    });

    // Manual filter (baseline)
    group.bench_function("manual_filter/by_class", |b| {
        b.iter(|| {
            let filtered: Vec<_> = ul.children().filter(|c| c.has_class("active")).collect();
            black_box(filtered);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_fragment_parsing,
    bench_compiled_selector,
    bench_select_text,
    bench_select_attr,
    bench_text_nodes,
    bench_filtered_iterators,
);
criterion_main!(benches);
