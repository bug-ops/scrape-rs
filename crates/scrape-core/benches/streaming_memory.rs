//! Comprehensive streaming vs single-shot parsing benchmarks.
//!
//! Measures memory efficiency and throughput for large file processing.

use std::{fmt::Write, hint::black_box};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
#[cfg(feature = "streaming")]
use scrape_core::StreamingSoup;

fn generate_large_html(size_mb: usize) -> String {
    let mut html = String::from("<html><body>");
    let target_size = size_mb * 1024 * 1024;
    let item_template = r#"<div class="product">
        <h2>Product Name with some text</h2>
        <p class="description">This is a longer description that contains multiple sentences. It describes the product features and benefits in detail. The description is quite verbose to increase the document size.</p>
        <ul class="features">
            <li>Feature one with description</li>
            <li>Feature two with description</li>
            <li>Feature three with description</li>
            <li>Feature four with description</li>
            <li>Feature five with description</li>
        </ul>
        <div class="price">$99.99</div>
    </div>"#;

    let item_size = item_template.len();
    let num_items = target_size / item_size;

    for i in 0..num_items {
        let _ = write!(html, "<div id='item-{i}'>{item_template}</div>");
    }

    html.push_str("</body></html>");
    html
}

#[cfg(feature = "streaming")]
fn bench_streaming_vs_singleshot_small(c: &mut Criterion) {
    let html = generate_large_html(1); // 1 MB
    let mut group = c.benchmark_group("streaming_vs_singleshot_1mb");
    group.throughput(Throughput::Bytes(html.len() as u64));

    group.bench_function("streaming", |b| {
        b.iter(|| {
            let streaming = StreamingSoup::new();
            let mut processor = streaming.start();
            processor.write(black_box(html.as_bytes())).unwrap();
            let finished = processor.end().unwrap();
            let stats = finished.stats();
            black_box(stats);
        });
    });

    group.bench_function("singleshot", |b| {
        b.iter(|| {
            let soup = scrape_core::Soup::parse(black_box(&html));
            let count = soup.find_all(".product").ok().map_or(0, |v| v.len());
            black_box(count);
        });
    });

    group.finish();
}

#[cfg(feature = "streaming")]
fn bench_streaming_vs_singleshot_medium(c: &mut Criterion) {
    let html = generate_large_html(10); // 10 MB
    let mut group = c.benchmark_group("streaming_vs_singleshot_10mb");
    group.throughput(Throughput::Bytes(html.len() as u64));
    group.sample_size(20);

    group.bench_function("streaming", |b| {
        b.iter(|| {
            let streaming = StreamingSoup::new();
            let mut processor = streaming.start();
            processor.write(black_box(html.as_bytes())).unwrap();
            let finished = processor.end().unwrap();
            let stats = finished.stats();
            black_box(stats);
        });
    });

    group.bench_function("singleshot", |b| {
        b.iter(|| {
            let soup = scrape_core::Soup::parse(black_box(&html));
            let count = soup.find_all(".product").ok().map_or(0, |v| v.len());
            black_box(count);
        });
    });

    group.finish();
}

#[cfg(feature = "streaming")]
fn bench_streaming_vs_singleshot_large(c: &mut Criterion) {
    let html = generate_large_html(50); // 50 MB
    let mut group = c.benchmark_group("streaming_vs_singleshot_50mb");
    group.throughput(Throughput::Bytes(html.len() as u64));
    group.sample_size(10);

    group.bench_function("streaming", |b| {
        b.iter(|| {
            let streaming = StreamingSoup::new();
            let mut processor = streaming.start();
            processor.write(black_box(html.as_bytes())).unwrap();
            let finished = processor.end().unwrap();
            let stats = finished.stats();
            black_box(stats);
        });
    });

    group.bench_function("singleshot", |b| {
        b.iter(|| {
            let soup = scrape_core::Soup::parse(black_box(&html));
            let count = soup.find_all(".product").ok().map_or(0, |v| v.len());
            black_box(count);
        });
    });

    group.finish();
}

#[cfg(feature = "streaming")]
fn bench_streaming_chunked_processing(c: &mut Criterion) {
    let html = generate_large_html(10); // 10 MB
    let mut group = c.benchmark_group("streaming_chunked");
    group.throughput(Throughput::Bytes(html.len() as u64));

    for chunk_size in [1024, 8192, 65536] {
        group.bench_with_input(
            BenchmarkId::new("chunk_size", chunk_size),
            &chunk_size,
            |b, &chunk_size| {
                b.iter(|| {
                    let streaming = StreamingSoup::new();
                    let mut processor = streaming.start();
                    let bytes = html.as_bytes();
                    for chunk in bytes.chunks(chunk_size) {
                        processor.write(black_box(chunk)).unwrap();
                    }
                    let finished = processor.end().unwrap();
                    let stats = finished.stats();
                    black_box(stats);
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "streaming")]
fn bench_streaming_text_extraction(c: &mut Criterion) {
    let html = generate_large_html(5); // 5 MB
    let mut group = c.benchmark_group("streaming_text_extraction");
    group.throughput(Throughput::Bytes(html.len() as u64));

    group.bench_function("streaming_parse", |b| {
        b.iter(|| {
            let streaming = StreamingSoup::new();
            let mut processor = streaming.start();
            processor.write(black_box(html.as_bytes())).unwrap();
            let finished = processor.end().unwrap();
            let stats = finished.stats();
            black_box(stats);
        });
    });

    group.bench_function("singleshot_parse", |b| {
        b.iter(|| {
            let soup = scrape_core::Soup::parse(black_box(&html));
            let count = soup.find_all(".description").ok().map_or(0, |v| v.len());
            black_box(count);
        });
    });

    group.finish();
}

#[cfg(feature = "streaming")]
fn bench_streaming_vs_singleshot(c: &mut Criterion) {
    let html = generate_large_html(5); // 5 MB
    let mut group = c.benchmark_group("streaming_throughput");
    group.throughput(Throughput::Bytes(html.len() as u64));

    group.bench_function("streaming_parse", |b| {
        b.iter(|| {
            let streaming = StreamingSoup::new();
            let mut processor = streaming.start();
            processor.write(black_box(html.as_bytes())).unwrap();
            let finished = processor.end().unwrap();
            let stats = finished.stats();
            black_box(stats);
        });
    });

    group.bench_function("singleshot_parse_and_queries", |b| {
        b.iter(|| {
            let soup = scrape_core::Soup::parse(black_box(&html));
            let products = soup.find_all(".product").ok().map_or(0, |v| v.len());
            let descriptions = soup.find_all(".description").ok().map_or(0, |v| v.len());
            let prices = soup.find_all(".price").ok().map_or(0, |v| v.len());
            black_box((products, descriptions, prices));
        });
    });

    group.finish();
}

#[cfg(not(feature = "streaming"))]
fn dummy_bench(_c: &mut Criterion) {}

#[cfg(feature = "streaming")]
criterion_group!(
    benches,
    bench_streaming_vs_singleshot_small,
    bench_streaming_vs_singleshot_medium,
    bench_streaming_vs_singleshot_large,
    bench_streaming_chunked_processing,
    bench_streaming_text_extraction,
    bench_streaming_vs_singleshot,
);

#[cfg(not(feature = "streaming"))]
criterion_group!(benches, dummy_bench);

criterion_main!(benches);
