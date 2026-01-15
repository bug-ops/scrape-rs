//! Benchmarks for HTML parsing performance.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use scrape_core::Soup;

/// Sample HTML documents of varying sizes for benchmarking.
mod samples {
    use std::fmt::Write;

    pub const SMALL: &str = r#"
        <html>
            <head><title>Small Page</title></head>
            <body>
                <div class="container">
                    <h1>Hello World</h1>
                    <p>This is a small test page.</p>
                </div>
            </body>
        </html>
    "#;

    pub const MEDIUM: &str = include_str!("samples/medium.html");

    /// Generate a large HTML document with many elements.
    pub fn generate_large(element_count: usize) -> String {
        let mut html = String::from("<html><head><title>Large</title></head><body>");
        for i in 0..element_count {
            let _ = write!(
                html,
                r#"<div class="item-{i}" id="item-{i}"><span>Item {i}</span></div>"#
            );
        }
        html.push_str("</body></html>");
        html
    }
}

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    // Small document
    group.throughput(Throughput::Bytes(samples::SMALL.len() as u64));
    group.bench_function("small", |b| {
        b.iter(|| Soup::parse(black_box(samples::SMALL)));
    });

    // Large documents of varying sizes
    for size in [100, 1000, 10000] {
        let html = samples::generate_large(size);
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("large", size), &html, |b, html| {
            b.iter(|| Soup::parse(black_box(html)));
        });
    }

    group.finish();
}

fn bench_find(c: &mut Criterion) {
    let mut group = c.benchmark_group("find");

    let html = samples::generate_large(1000);
    let soup = Soup::parse(&html);

    group.bench_function("by_tag", |b| {
        b.iter(|| soup.find(black_box("div")));
    });

    group.bench_function("by_class", |b| {
        b.iter(|| soup.find(black_box(".item-500")));
    });

    group.bench_function("by_id", |b| {
        b.iter(|| soup.find(black_box("#item-500")));
    });

    group.finish();
}

fn bench_find_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_all");

    let html = samples::generate_large(1000);
    let soup = Soup::parse(&html);

    group.bench_function("all_divs", |b| {
        b.iter(|| {
            let _: Vec<_> = soup.find_all(black_box("div")).collect();
        });
    });

    group.bench_function("all_spans", |b| {
        b.iter(|| {
            let _: Vec<_> = soup.find_all(black_box("span")).collect();
        });
    });

    group.finish();
}

criterion_group!(benches, bench_parse, bench_find, bench_find_all);
criterion_main!(benches);
