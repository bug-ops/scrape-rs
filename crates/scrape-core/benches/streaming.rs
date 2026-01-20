//! Phase 18 streaming performance benchmarks.
//!
//! Measures performance characteristics of streaming HTML parsing:
//! - StreamingSoup initialization overhead
//! - Handler registration performance
//! - Memory overhead (typestate pattern should be zero-cost)
//! - State transition overhead

#[cfg(feature = "streaming")]
use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
#[cfg(feature = "streaming")]
use scrape_core::StreamingSoup;

#[cfg(feature = "streaming")]
fn bench_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_init");

    group.bench_function("new_default", |b| {
        b.iter(|| {
            let streaming = StreamingSoup::new();
            black_box(streaming);
        });
    });

    group.bench_function("with_config", |b| {
        b.iter(|| {
            let config = scrape_core::streaming::StreamingConfig::new().buffer_size(16384);
            let streaming = StreamingSoup::with_config(config);
            black_box(streaming);
        });
    });

    group.finish();
}

#[cfg(feature = "streaming")]
fn bench_handler_registration(c: &mut Criterion) {
    let mut group = c.benchmark_group("handler_registration");

    group.bench_function("single_element", |b| {
        b.iter(|| {
            let mut streaming = StreamingSoup::new();
            streaming.on_element("div", |_el| Ok(())).unwrap();
            black_box(streaming);
        });
    });

    group.bench_function("single_text", |b| {
        b.iter(|| {
            let mut streaming = StreamingSoup::new();
            streaming.on_text("p", |_text| Ok(())).unwrap();
            black_box(streaming);
        });
    });

    group.bench_function("single_end_tag", |b| {
        b.iter(|| {
            let mut streaming = StreamingSoup::new();
            streaming.on_end_tag("div", |_tag| Ok(())).unwrap();
            black_box(streaming);
        });
    });

    // Multiple handlers
    for count in [1, 5, 10, 25, 50] {
        group.bench_with_input(BenchmarkId::new("multiple_element", count), &count, |b, &count| {
            b.iter(|| {
                let mut streaming = StreamingSoup::new();
                for i in 0..count {
                    streaming.on_element(&format!("div.class-{i}"), |_el| Ok(())).unwrap();
                }
                black_box(streaming);
            });
        });
    }

    group.finish();
}

#[cfg(feature = "streaming")]
fn bench_state_transitions(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_transitions");

    group.bench_function("idle_to_processing", |b| {
        b.iter(|| {
            let streaming = StreamingSoup::new();
            let processor = streaming.start();
            black_box(processor);
        });
    });

    group.bench_function("processing_to_finished", |b| {
        b.iter(|| {
            let streaming = StreamingSoup::new();
            let processor = streaming.start();
            let finished = processor.end().unwrap();
            black_box(finished);
        });
    });

    group.bench_function("full_lifecycle_empty", |b| {
        b.iter(|| {
            let streaming = StreamingSoup::new();
            let processor = streaming.start();
            let finished = processor.end().unwrap();
            let stats = finished.stats();
            black_box(stats);
        });
    });

    group.finish();
}

#[cfg(feature = "streaming")]
fn bench_write_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_write");

    let small_chunk = b"<div>test</div>";
    group.throughput(Throughput::Bytes(small_chunk.len() as u64));
    group.bench_function("single_small_chunk", |b| {
        b.iter(|| {
            let streaming = StreamingSoup::new();
            let mut processor = streaming.start();
            processor.write(black_box(small_chunk)).unwrap();
            black_box(processor);
        });
    });

    // Test varying chunk sizes
    for size in [100, 1024, 8192, 16384] {
        let chunk = vec![b'<'; size];
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("single_chunk", format!("{size}b")),
            &chunk,
            |b, chunk| {
                b.iter(|| {
                    let streaming = StreamingSoup::new();
                    let mut processor = streaming.start();
                    processor.write(black_box(chunk)).unwrap();
                    black_box(processor);
                });
            },
        );
    }

    // Test multiple chunks
    let chunks: Vec<&[u8]> =
        vec![b"<html>", b"<body>", b"<div>content</div>", b"</body>", b"</html>"];
    let total_size: usize = chunks.iter().map(|c| c.len()).sum();
    group.throughput(Throughput::Bytes(total_size as u64));
    group.bench_function("multiple_small_chunks", |b| {
        b.iter(|| {
            let streaming = StreamingSoup::new();
            let mut processor = streaming.start();
            for chunk in &chunks {
                processor.write(black_box(chunk)).unwrap();
            }
            black_box(processor);
        });
    });

    group.finish();
}

#[cfg(feature = "streaming")]
fn bench_memory_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_overhead");

    // Measure allocation overhead of typestate pattern
    group.bench_function("typestate_idle", |b| {
        b.iter(|| {
            let streaming = StreamingSoup::new();
            black_box(streaming);
        });
    });

    group.bench_function("typestate_processing", |b| {
        b.iter(|| {
            let streaming = StreamingSoup::new();
            let processor = streaming.start();
            black_box(processor);
        });
    });

    group.bench_function("typestate_finished", |b| {
        b.iter(|| {
            let streaming = StreamingSoup::new();
            let processor = streaming.start();
            let finished = processor.end().unwrap();
            black_box(finished);
        });
    });

    group.finish();
}

#[cfg(feature = "streaming")]
criterion_group!(
    benches,
    bench_initialization,
    bench_handler_registration,
    bench_state_transitions,
    bench_write_operations,
    bench_memory_overhead,
);

#[cfg(not(feature = "streaming"))]
fn dummy_bench(_c: &mut Criterion) {}

#[cfg(not(feature = "streaming"))]
criterion_group!(benches, dummy_bench);

#[cfg(feature = "streaming")]
criterion_main!(benches);

#[cfg(not(feature = "streaming"))]
criterion_main!(benches);
