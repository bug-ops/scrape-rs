//! Navigation benchmarks for scrape-rs Phase 12 features.
//!
//! Benchmarks the performance of:
//! - parents() / ancestors()
//! - closest(selector)
//! - next_siblings()
//! - prev_siblings()
//! - siblings()
//!
//! Run with: `cargo bench --bench navigation`

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use scrape_core::Soup;

mod fixtures {
    use std::fmt::Write;

    /// Generate HTML with deep nesting (for testing ancestors/closest)
    pub fn deep_nesting(depth: usize) -> String {
        let mut html = String::from("<html><body>");
        for i in 0..depth {
            let _ = write!(html, r#"<div class="level-{i}" id="div-{i}">"#);
        }
        html.push_str("<span id='target'>Target</span>");
        for _ in 0..depth {
            html.push_str("</div>");
        }
        html.push_str("</body></html>");
        html
    }

    /// Generate HTML with wide siblings (for testing sibling iteration)
    pub fn wide_siblings(count: usize) -> String {
        let mut html = String::from("<html><body><ul>");
        for i in 0..count {
            let _ = write!(html, r#"<li class="item-{i}" id="li-{i}">Item {i}</li>"#);
        }
        html.push_str("</ul></body></html>");
        html
    }

    /// Generate HTML with mixed structure (depth + width)
    pub fn mixed(depth: usize, width: usize) -> String {
        let mut html = String::from("<html><body>");
        for d in 0..depth {
            let _ = write!(html, r#"<div class="depth-{d}">"#);
            for w in 0..width {
                let _ = write!(html, r#"<span class="width-{w}">Content</span>"#);
            }
        }
        for _ in 0..depth {
            html.push_str("</div>");
        }
        html.push_str("</body></html>");
        html
    }
}

// ==================== Baseline: children() iterator ====================

fn bench_baseline_children(c: &mut Criterion) {
    let mut group = c.benchmark_group("baseline/children");

    for width in [10, 100, 1000] {
        let html = fixtures::wide_siblings(width);
        let soup = Soup::parse(&html);
        let ul = soup.find("ul").unwrap().unwrap();

        group.throughput(Throughput::Elements(width as u64));
        group.bench_with_input(
            BenchmarkId::new("iterate_all", width),
            &ul,
            |b, ul| {
                b.iter(|| {
                    let _: Vec<_> = ul.children().collect();
                });
            },
        );
    }

    group.finish();
}

// ==================== parents() / ancestors() ====================

fn bench_parents(c: &mut Criterion) {
    let mut group = c.benchmark_group("navigation/parents");

    for depth in [10, 100, 1000] {
        let html = fixtures::deep_nesting(depth);
        let soup = Soup::parse(&html);
        let target = soup.find("#target").unwrap().unwrap();

        group.throughput(Throughput::Elements(depth as u64));
        group.bench_with_input(
            BenchmarkId::new("iterate_all", depth),
            &target,
            |b, target| {
                b.iter(|| {
                    let _: Vec<_> = target.parents().collect();
                });
            },
        );
    }

    group.finish();
}

fn bench_ancestors(c: &mut Criterion) {
    let mut group = c.benchmark_group("navigation/ancestors");

    for depth in [10, 100, 1000] {
        let html = fixtures::deep_nesting(depth);
        let soup = Soup::parse(&html);
        let target = soup.find("#target").unwrap().unwrap();

        group.throughput(Throughput::Elements(depth as u64));
        group.bench_with_input(
            BenchmarkId::new("iterate_all", depth),
            &target,
            |b, target| {
                b.iter(|| {
                    let _: Vec<_> = target.ancestors().collect();
                });
            },
        );
    }

    group.finish();
}

// ==================== closest(selector) ====================

fn bench_closest(c: &mut Criterion) {
    let mut group = c.benchmark_group("navigation/closest");

    // Test at different depths - selector matches near vs far
    for (depth, target_level) in [(10, 5), (100, 50), (1000, 500)] {
        let html = fixtures::deep_nesting(depth);
        let soup = Soup::parse(&html);
        let target = soup.find("#target").unwrap().unwrap();

        let selector = format!(".level-{target_level}");
        group.bench_with_input(
            BenchmarkId::new("find_midpoint", depth),
            &target,
            |b, target| {
                b.iter(|| {
                    let _ = target.closest(black_box(&selector));
                });
            },
        );
    }

    // Test selector that doesn't match (worst case - full ancestor traversal)
    for depth in [10, 100, 1000] {
        let html = fixtures::deep_nesting(depth);
        let soup = Soup::parse(&html);
        let target = soup.find("#target").unwrap().unwrap();

        group.bench_with_input(
            BenchmarkId::new("no_match", depth),
            &target,
            |b, target| {
                b.iter(|| {
                    let _ = target.closest(black_box(".nonexistent"));
                });
            },
        );
    }

    // Test with complex selector
    for depth in [10, 100] {
        let html = fixtures::deep_nesting(depth);
        let soup = Soup::parse(&html);
        let target = soup.find("#target").unwrap().unwrap();

        group.bench_with_input(
            BenchmarkId::new("complex_selector", depth),
            &target,
            |b, target| {
                b.iter(|| {
                    let _ = target.closest(black_box("div.level-5[id]"));
                });
            },
        );
    }

    group.finish();
}

// ==================== next_siblings() ====================

fn bench_next_siblings(c: &mut Criterion) {
    let mut group = c.benchmark_group("navigation/next_siblings");

    for width in [10, 100, 1000] {
        let html = fixtures::wide_siblings(width);
        let soup = Soup::parse(&html);
        let first = soup.find("li").unwrap().unwrap();

        group.throughput(Throughput::Elements((width - 1) as u64));
        group.bench_with_input(
            BenchmarkId::new("iterate_all", width),
            &first,
            |b, first| {
                b.iter(|| {
                    let _: Vec<_> = first.next_siblings().collect();
                });
            },
        );
    }

    group.finish();
}

// ==================== prev_siblings() ====================

fn bench_prev_siblings(c: &mut Criterion) {
    let mut group = c.benchmark_group("navigation/prev_siblings");

    for width in [10, 100, 1000] {
        let html = fixtures::wide_siblings(width);
        let soup = Soup::parse(&html);
        let last_id = format!("#li-{}", width - 1);
        let last = soup.find(&last_id).unwrap().unwrap();

        group.throughput(Throughput::Elements((width - 1) as u64));
        group.bench_with_input(
            BenchmarkId::new("iterate_all", width),
            &last,
            |b, last| {
                b.iter(|| {
                    let _: Vec<_> = last.prev_siblings().collect();
                });
            },
        );
    }

    group.finish();
}

// ==================== siblings() ====================

fn bench_siblings(c: &mut Criterion) {
    let mut group = c.benchmark_group("navigation/siblings");

    for width in [10, 100, 1000] {
        let html = fixtures::wide_siblings(width);
        let soup = Soup::parse(&html);
        let middle_id = format!("#li-{}", width / 2);
        let middle = soup.find(&middle_id).unwrap().unwrap();

        group.throughput(Throughput::Elements((width - 1) as u64));
        group.bench_with_input(
            BenchmarkId::new("iterate_all", width),
            &middle,
            |b, middle| {
                b.iter(|| {
                    let _: Vec<_> = middle.siblings().collect();
                });
            },
        );
    }

    group.finish();
}

// ==================== Comparison: New vs Baseline ====================

fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison");

    let width = 100;
    let html = fixtures::wide_siblings(width);
    let soup = Soup::parse(&html);
    let ul = soup.find("ul").unwrap().unwrap();
    let first = soup.find("li").unwrap().unwrap();

    // Baseline: children()
    group.bench_function("baseline_children_100", |b| {
        b.iter(|| {
            let _: Vec<_> = ul.children().collect();
        });
    });

    // New: next_siblings()
    group.bench_function("next_siblings_100", |b| {
        b.iter(|| {
            let _: Vec<_> = first.next_siblings().collect();
        });
    });

    let depth = 100;
    let html = fixtures::deep_nesting(depth);
    let soup = Soup::parse(&html);
    let target = soup.find("#target").unwrap().unwrap();

    // New: parents()
    group.bench_function("parents_100", |b| {
        b.iter(|| {
            let _: Vec<_> = target.parents().collect();
        });
    });

    group.finish();
}

// ==================== Edge Cases ====================

fn bench_edge_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("edge_cases");

    // Empty document
    let soup = Soup::parse("<html><body><div id='only'></div></body></html>");
    let div = soup.find("#only").unwrap().unwrap();

    group.bench_function("parents_shallow", |b| {
        b.iter(|| {
            let _: Vec<_> = div.parents().collect();
        });
    });

    group.bench_function("siblings_single", |b| {
        b.iter(|| {
            let _: Vec<_> = div.siblings().collect();
        });
    });

    // Very deep nesting (stress test)
    let html = fixtures::deep_nesting(10_000);
    let soup = Soup::parse(&html);
    let target = soup.find("#target").unwrap().unwrap();

    group.bench_function("parents_very_deep_10k", |b| {
        b.iter(|| {
            let _: Vec<_> = target.parents().collect();
        });
    });

    group.bench_function("closest_very_deep_10k_no_match", |b| {
        b.iter(|| {
            let _ = target.closest(black_box(".nonexistent"));
        });
    });

    // Very wide siblings (stress test)
    let html = fixtures::wide_siblings(10_000);
    let soup = Soup::parse(&html);
    let first = soup.find("li").unwrap().unwrap();

    group.bench_function("next_siblings_very_wide_10k", |b| {
        b.iter(|| {
            let _: Vec<_> = first.next_siblings().collect();
        });
    });

    group.finish();
}

// ==================== Memory Allocation Patterns ====================

fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    // Test repeated iteration (should not allocate between iterations)
    let html = fixtures::wide_siblings(100);
    let soup = Soup::parse(&html);
    let first = soup.find("li").unwrap().unwrap();

    group.bench_function("next_siblings_repeated_100", |b| {
        b.iter(|| {
            for _ in 0..10 {
                let _: Vec<_> = first.next_siblings().collect();
            }
        });
    });

    // Test iterator chaining (should be zero-cost)
    group.bench_function("chained_iteration", |b| {
        b.iter(|| {
            let count = first
                .next_siblings()
                .filter(|t| t.name() == Some("li"))
                .take(50)
                .count();
            black_box(count);
        });
    });

    group.finish();
}

// ==================== Real-world Scenarios ====================

fn bench_real_world(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world");

    // Scenario: Find all section headers
    let html = r#"
    <html><body>
        <article>
            <h1>Title</h1>
            <section><h2>Section 1</h2><p>Content 1</p></section>
            <section><h2>Section 2</h2><p>Content 2</p></section>
            <section><h2>Section 3</h2><p>Content 3</p></section>
        </article>
    </body></html>
    "#;
    let soup = Soup::parse(html);

    group.bench_function("find_section_headers", |b| {
        b.iter(|| {
            if let Ok(Some(article)) = soup.find("article") {
                let sections: Vec<_> = article
                    .find_all("section")
                    .unwrap()
                    .into_iter()
                    .filter_map(|s| s.find("h2").ok().flatten())
                    .collect();
                black_box(sections);
            }
        });
    });

    // Scenario: Find closest form from input
    let html = r#"
    <html><body>
        <form id="outer">
            <div class="container">
                <div class="field">
                    <input type="text" id="test-input" />
                </div>
            </div>
        </form>
    </body></html>
    "#;
    let soup = Soup::parse(html);
    let input = soup.find("#test-input").unwrap().unwrap();

    group.bench_function("find_closest_form", |b| {
        b.iter(|| {
            let _ = input.closest(black_box("form"));
        });
    });

    // Scenario: Navigate sibling tabs
    let html = r#"
    <html><body>
        <div class="tabs">
            <button class="tab" data-tab="1">Tab 1</button>
            <button class="tab active" data-tab="2">Tab 2</button>
            <button class="tab" data-tab="3">Tab 3</button>
            <button class="tab" data-tab="4">Tab 4</button>
        </div>
    </body></html>
    "#;
    let soup = Soup::parse(html);
    let active = soup.find(".tab.active").unwrap().unwrap();

    group.bench_function("find_adjacent_tabs", |b| {
        b.iter(|| {
            let prev: Vec<_> = active.prev_siblings().collect();
            let next: Vec<_> = active.next_siblings().collect();
            black_box((prev, next));
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(100);
    targets = bench_baseline_children,
              bench_parents,
              bench_ancestors,
              bench_closest,
              bench_next_siblings,
              bench_prev_siblings,
              bench_siblings,
              bench_comparison,
              bench_edge_cases,
              bench_memory_patterns,
              bench_real_world
}
criterion_main!(benches);
