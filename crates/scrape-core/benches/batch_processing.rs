//! Comprehensive batch processing benchmarks.
//!
//! Measures parallel batch parsing performance with different document counts,
//! sizes, and CPU utilization patterns.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main, black_box};

#[cfg(feature = "parallel")]
use scrape_core::parallel::parse_batch;

const SMALL_DOC: &str = r#"<!DOCTYPE html>
<html>
<head><title>Small Page</title></head>
<body>
<div class="container">
<h1>Welcome</h1>
<p>This is a small test document.</p>
</div>
</body>
</html>"#;

const MEDIUM_DOC: &str = include_str!("samples/medium.html");

fn generate_large_doc() -> String {
    let mut html = String::from("<html><body>");
    for i in 0..1000 {
        html.push_str(&format!(
            r#"<div class="item" id="item-{i}">
            <h2>Item {i}</h2>
            <p>Description for item {i} with some text content.</p>
            <ul>
                <li>Feature 1</li>
                <li>Feature 2</li>
                <li>Feature 3</li>
            </ul>
        </div>"#
        ));
    }
    html.push_str("</body></html>");
    html
}

#[cfg(feature = "parallel")]
fn bench_batch_small_documents(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_small_docs");

    for count in [10, 50, 100] {
        let docs: Vec<_> = (0..count).map(|_| SMALL_DOC).collect();
        let total_size = SMALL_DOC.len() * count;

        group.throughput(Throughput::Bytes(total_size as u64));
        group.bench_with_input(
            BenchmarkId::new("parallel", count),
            &docs,
            |b, docs| {
                b.iter(|| {
                    let results = parse_batch(black_box(docs));
                    black_box(results);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("sequential", count),
            &docs,
            |b, docs| {
                b.iter(|| {
                    let results: Vec<_> = docs
                        .iter()
                        .map(|html| scrape_core::Soup::parse(black_box(html)))
                        .collect();
                    black_box(results);
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "parallel")]
fn bench_batch_medium_documents(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_medium_docs");

    for count in [10, 50, 100] {
        let docs: Vec<_> = (0..count).map(|_| MEDIUM_DOC).collect();
        let total_size = MEDIUM_DOC.len() * count;

        group.throughput(Throughput::Bytes(total_size as u64));
        group.bench_with_input(
            BenchmarkId::new("parallel", count),
            &docs,
            |b, docs| {
                b.iter(|| {
                    let results = parse_batch(black_box(docs));
                    black_box(results);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("sequential", count),
            &docs,
            |b, docs| {
                b.iter(|| {
                    let results: Vec<_> = docs
                        .iter()
                        .map(|html| scrape_core::Soup::parse(black_box(html)))
                        .collect();
                    black_box(results);
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "parallel")]
fn bench_batch_large_documents(c: &mut Criterion) {
    let large_doc = generate_large_doc();
    let mut group = c.benchmark_group("batch_large_docs");

    for count in [10, 50] {
        let docs: Vec<_> = (0..count).map(|_| large_doc.as_str()).collect();
        let total_size = large_doc.len() * count;

        group.throughput(Throughput::Bytes(total_size as u64));
        group.bench_with_input(
            BenchmarkId::new("parallel", count),
            &docs,
            |b, docs| {
                b.iter(|| {
                    let results = parse_batch(black_box(docs));
                    black_box(results);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("sequential", count),
            &docs,
            |b, docs| {
                b.iter(|| {
                    let results: Vec<_> = docs
                        .iter()
                        .map(|html| scrape_core::Soup::parse(black_box(html)))
                        .collect();
                    black_box(results);
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "parallel")]
fn bench_batch_mixed_sizes(c: &mut Criterion) {
    let large_doc = generate_large_doc();
    let mut group = c.benchmark_group("batch_mixed_sizes");

    let mixed_docs: Vec<&str> = vec![
        SMALL_DOC,
        MEDIUM_DOC,
        large_doc.as_str(),
        SMALL_DOC,
        MEDIUM_DOC,
        large_doc.as_str(),
        SMALL_DOC,
        MEDIUM_DOC,
        large_doc.as_str(),
        SMALL_DOC,
    ];

    let total_size: usize = mixed_docs.iter().map(|s| s.len()).sum();
    group.throughput(Throughput::Bytes(total_size as u64));

    group.bench_function("parallel", |b| {
        b.iter(|| {
            let results = parse_batch(black_box(&mixed_docs));
            black_box(results);
        });
    });

    group.bench_function("sequential", |b| {
        b.iter(|| {
            let results: Vec<_> = mixed_docs
                .iter()
                .map(|html| scrape_core::Soup::parse(black_box(html)))
                .collect();
            black_box(results);
        });
    });

    group.finish();
}

#[cfg(feature = "parallel")]
fn bench_batch_query_after_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_parse_and_query");

    let count = 50;
    let docs: Vec<_> = (0..count).map(|_| MEDIUM_DOC).collect();

    group.bench_function("parallel_parse_then_query", |b| {
        b.iter(|| {
            let results = parse_batch(black_box(&docs));
            let query_results: Vec<_> = results
                .iter()
                .map(|soup| soup.find(".product-card").unwrap())
                .collect();
            black_box(query_results);
        });
    });

    group.bench_function("sequential_parse_then_query", |b| {
        b.iter(|| {
            let results: Vec<_> = docs
                .iter()
                .map(|html| scrape_core::Soup::parse(black_box(html)))
                .collect();
            let query_results: Vec<_> = results
                .iter()
                .map(|soup| soup.find(".product-card").unwrap())
                .collect();
            black_box(query_results);
        });
    });

    group.finish();
}

#[cfg(not(feature = "parallel"))]
fn dummy_bench(_c: &mut Criterion) {}

#[cfg(feature = "parallel")]
criterion_group!(
    benches,
    bench_batch_small_documents,
    bench_batch_medium_documents,
    bench_batch_large_documents,
    bench_batch_mixed_sizes,
    bench_batch_query_after_parse,
);

#[cfg(not(feature = "parallel"))]
criterion_group!(benches, dummy_bench);

criterion_main!(benches);
