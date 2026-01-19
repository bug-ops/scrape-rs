//! Benchmarks for Phase 13 query engine features.
//!
//! Tests performance of:
//! - CompiledSelector vs string selectors
//! - TextNodesIter vs .text()
//! - Filtered iterators (.children_by_name, .children_by_class)

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use scrape_core::{Soup, query::CompiledSelector};

/// Generate HTML with many nested elements for benchmarking.
fn generate_nested_html(depth: usize, breadth: usize) -> String {
    fn build_tree(depth: usize, breadth: usize, level: usize) -> String {
        if level >= depth {
            return String::from("<span class='leaf'>Text content here</span>");
        }

        let mut html = String::new();
        for i in 0..breadth {
            use std::fmt::Write;
            let _ = write!(html, "<div class='level-{level} item-{i}' id='div-{level}-{i}'>");
            html.push_str(&build_tree(depth, breadth, level + 1));
            html.push_str("</div>");
        }
        html
    }

    let mut html = String::from("<html><head><title>Nested</title></head><body>");
    html.push_str(&build_tree(depth, breadth, 0));
    html.push_str("</body></html>");
    html
}

/// Generate HTML with many text nodes for text iteration benchmarks.
fn generate_text_heavy_html(text_nodes: usize) -> String {
    use std::fmt::Write;
    let mut html = String::from("<html><body><div>");
    for i in 0..text_nodes {
        if i % 3 == 0 {
            let _ = write!(html, "<span>Text {i}</span>");
        } else if i % 3 == 1 {
            let _ = write!(html, "<b>Bold {i}</b>");
        } else {
            let _ = write!(html, "<i>Italic {i}</i>");
        }
    }
    html.push_str("</div></body></html>");
    html
}

/// Generate HTML with many children for filtered iterator benchmarks.
fn generate_children_html(child_count: usize) -> String {
    use std::fmt::Write;
    let mut html = String::from("<html><body><div id='container'>");
    for i in 0..child_count {
        let tag = if i % 3 == 0 { "div" } else { "span" };
        let class = if i % 2 == 0 { "even" } else { "odd" };
        let _ = write!(html, "<{tag} class='{class}'>Item {i}</{tag}>");
    }
    html.push_str("</div></body></html>");
    html
}

// ============================================================================
// Benchmark 1: Compiled Selectors
// ============================================================================

fn bench_compiled_vs_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("compiled_selector");

    let html = generate_nested_html(5, 5);
    let soup = Soup::parse(&html);

    // Simple selector
    let simple_selector = "div";
    let compiled_simple = CompiledSelector::compile(simple_selector).unwrap();

    group.bench_function("string/simple/single", |b| {
        b.iter(|| soup.find(black_box(simple_selector)));
    });

    group.bench_function("compiled/simple/single", |b| {
        b.iter(|| soup.find_compiled(black_box(&compiled_simple)));
    });

    group.bench_function("string/simple/all", |b| {
        b.iter(|| soup.select(black_box(simple_selector)));
    });

    group.bench_function("compiled/simple/all", |b| {
        b.iter(|| soup.select_compiled(black_box(&compiled_simple)));
    });

    // Complex selector
    let complex_selector = "div.level-2 > span.leaf";
    let compiled_complex = CompiledSelector::compile(complex_selector).unwrap();

    group.bench_function("string/complex/single", |b| {
        b.iter(|| soup.find(black_box(complex_selector)));
    });

    group.bench_function("compiled/complex/single", |b| {
        b.iter(|| soup.find_compiled(black_box(&compiled_complex)));
    });

    group.bench_function("string/complex/all", |b| {
        b.iter(|| soup.select(black_box(complex_selector)));
    });

    group.bench_function("compiled/complex/all", |b| {
        b.iter(|| soup.select_compiled(black_box(&compiled_complex)));
    });

    group.finish();
}

fn bench_compiled_selector_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("compiled_selector_reuse");

    let html = generate_nested_html(5, 5);
    let selector = "div.level-2 > span";
    let compiled = CompiledSelector::compile(selector).unwrap();

    // Benchmark repeated queries
    for iterations in [10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("string", iterations),
            &iterations,
            |b, &iterations| {
                b.iter(|| {
                    for _ in 0..iterations {
                        let soup = Soup::parse(black_box(&html));
                        let _ = soup.find(black_box(selector));
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("compiled", iterations),
            &iterations,
            |b, &iterations| {
                b.iter(|| {
                    for _ in 0..iterations {
                        let soup = Soup::parse(black_box(&html));
                        let _ = soup.find_compiled(black_box(&compiled));
                    }
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark 2: Text Node Access
// ============================================================================

fn bench_text_nodes_vs_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_access");

    for text_count in [10, 100, 1000] {
        let html = generate_text_heavy_html(text_count);
        let soup = Soup::parse(&html);
        let div = soup.find("div").unwrap().unwrap();

        group.bench_with_input(BenchmarkId::new("text_nodes", text_count), &div, |b, div| {
            b.iter(|| {
                let texts: Vec<_> = div.text_nodes().collect();
                black_box(texts);
            });
        });

        group.bench_with_input(BenchmarkId::new("text", text_count), &div, |b, div| {
            b.iter(|| {
                let text = div.text();
                black_box(text);
            });
        });
    }

    group.finish();
}

fn bench_text_nodes_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_nodes_scalability");

    for depth in [5, 10, 15] {
        let html = generate_nested_html(depth, 3);
        let soup = Soup::parse(&html);
        let body = soup.find("body").unwrap().unwrap();

        group.bench_with_input(BenchmarkId::new("depth", depth), &body, |b, body| {
            b.iter(|| {
                let texts: Vec<_> = body.text_nodes().collect();
                black_box(texts);
            });
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark 3: Filtered Iterators
// ============================================================================

fn bench_filtered_iterators(c: &mut Criterion) {
    let mut group = c.benchmark_group("filtered_iterators");

    for child_count in [10, 100, 1000] {
        let html = generate_children_html(child_count);
        let soup = Soup::parse(&html);
        let container = soup.find("#container").unwrap().unwrap();

        // children_by_name
        group.bench_with_input(
            BenchmarkId::new("by_name", child_count),
            &container,
            |b, container| {
                b.iter(|| {
                    let divs: Vec<_> = container.children_by_name(black_box("div")).collect();
                    black_box(divs);
                });
            },
        );

        // Manual filtering for comparison
        group.bench_with_input(
            BenchmarkId::new("manual_name", child_count),
            &container,
            |b, container| {
                b.iter(|| {
                    let divs: Vec<_> = container
                        .children()
                        .filter(|tag| tag.name() == Some(black_box("div")))
                        .collect();
                    black_box(divs);
                });
            },
        );

        // children_by_class
        group.bench_with_input(
            BenchmarkId::new("by_class", child_count),
            &container,
            |b, container| {
                b.iter(|| {
                    let even: Vec<_> = container.children_by_class(black_box("even")).collect();
                    black_box(even);
                });
            },
        );

        // Manual filtering for comparison
        group.bench_with_input(
            BenchmarkId::new("manual_class", child_count),
            &container,
            |b, container| {
                b.iter(|| {
                    let even: Vec<_> = container
                        .children()
                        .filter(|tag| tag.has_class(black_box("even")))
                        .collect();
                    black_box(even);
                });
            },
        );
    }

    group.finish();
}

fn bench_filtered_iterator_chaining(c: &mut Criterion) {
    let mut group = c.benchmark_group("filtered_iterator_chaining");

    let html = generate_children_html(1000);
    let soup = Soup::parse(&html);
    let container = soup.find("#container").unwrap().unwrap();

    // Single filter
    group.bench_function("single_filter", |b| {
        b.iter(|| {
            let divs: Vec<_> = container.children_by_name("div").collect();
            black_box(divs);
        });
    });

    // Chained filters
    group.bench_function("chained_filters", |b| {
        b.iter(|| {
            let divs: Vec<_> =
                container.children_by_name("div").filter(|tag| tag.has_class("even")).collect();
            black_box(divs);
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark 4: Memory Usage
// ============================================================================

fn bench_compiled_selector_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory/compiled_selector");

    // Measure compilation overhead
    group.bench_function("compile", |b| {
        b.iter(|| {
            let selector =
                CompiledSelector::compile(black_box("div.class > span[attr='value']")).unwrap();
            black_box(selector);
        });
    });

    // Measure clone overhead
    let selector = CompiledSelector::compile("div.class > span").unwrap();
    group.bench_function("clone", |b| {
        b.iter(|| {
            let cloned = selector.clone();
            black_box(cloned);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_compiled_vs_string,
    bench_compiled_selector_reuse,
    bench_text_nodes_vs_text,
    bench_text_nodes_scalability,
    bench_filtered_iterators,
    bench_filtered_iterator_chaining,
    bench_compiled_selector_memory,
);
criterion_main!(benches);
