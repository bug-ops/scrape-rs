//! Benchmarks for SIMD-accelerated operations.
//!
//! This benchmark suite compares SIMD implementations against scalar baselines
//! to verify performance improvements.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
#[cfg(feature = "simd")]
use scrape_core::simd;

/// Generate HTML-like content with tags for scanning benchmarks.
fn generate_html_content(tag_count: usize) -> Vec<u8> {
    let mut content = Vec::with_capacity(tag_count * 50);
    for i in 0..tag_count {
        content.extend_from_slice(b"<div class=\"item-");
        content.extend_from_slice(i.to_string().as_bytes());
        content.extend_from_slice(b"\">Content ");
        content.extend_from_slice(i.to_string().as_bytes());
        content.extend_from_slice(b"</div>\n");
    }
    content
}

/// Generate text content for substring search benchmarks.
fn generate_text_content(word_count: usize) -> Vec<u8> {
    let words = ["hello", "world", "rust", "simd", "performance", "benchmark"];
    let mut content = Vec::with_capacity(word_count * 12);
    for i in 0..word_count {
        content.extend_from_slice(words[i % words.len()].as_bytes());
        content.push(b' ');
    }
    content
}

/// Scalar baseline: find first occurrence of `<`.
fn scalar_find_tag_start(bytes: &[u8]) -> Option<usize> {
    bytes.iter().position(|&b| b == b'<')
}

/// Scalar baseline: class matching using split_whitespace.
fn scalar_contains_class(class_attr: &str, target: &str) -> bool {
    class_attr.split_whitespace().any(|c| c == target)
}

/// Scalar baseline: substring search using windows.
fn scalar_find_text(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if needle.len() > haystack.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

#[cfg(feature = "simd")]
fn bench_tag_scanning(c: &mut Criterion) {
    let mut group = c.benchmark_group("tag_scanning");

    for tag_count in [100, 1000, 10000] {
        let content = generate_html_content(tag_count);
        group.throughput(Throughput::Bytes(content.len() as u64));

        group.bench_with_input(BenchmarkId::new("simd", tag_count), &content, |b, bytes| {
            b.iter(|| simd::find_tag_start(black_box(bytes)));
        });

        group.bench_with_input(BenchmarkId::new("scalar", tag_count), &content, |b, bytes| {
            b.iter(|| scalar_find_tag_start(black_box(bytes)));
        });
    }

    group.finish();
}

#[cfg(feature = "simd")]
fn bench_close_tag_scanning(c: &mut Criterion) {
    let mut group = c.benchmark_group("close_tag_scanning");

    for tag_count in [100, 1000, 10000] {
        let content = generate_html_content(tag_count);
        group.throughput(Throughput::Bytes(content.len() as u64));

        group.bench_with_input(BenchmarkId::new("simd", tag_count), &content, |b, bytes| {
            b.iter(|| simd::find_close_tag(black_box(bytes)));
        });

        group.bench_with_input(BenchmarkId::new("scalar", tag_count), &content, |b, bytes| {
            b.iter(|| bytes.windows(2).position(|w| w == b"</"));
        });
    }

    group.finish();
}

#[cfg(feature = "simd")]
fn bench_class_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("class_matching");

    // Typical class attributes with varying sizes
    let class_attrs = [
        ("small", "container flex"),
        ("medium", "container flex-row justify-center items-center"),
        (
            "large",
            "container flex-row justify-center items-center bg-gray-100 text-lg font-bold \
             shadow-md rounded-lg p-4 m-2",
        ),
        (
            "tailwind",
            "relative flex flex-col items-center justify-center min-h-screen bg-gradient-to-br \
             from-indigo-500 via-purple-500 to-pink-500 text-white",
        ),
    ];

    for (name, classes) in class_attrs {
        // Search for a class in the middle
        let target = classes
            .split_whitespace()
            .nth(classes.split_whitespace().count() / 2)
            .unwrap_or("missing");

        group.bench_with_input(BenchmarkId::new("simd", name), &(classes, target), |b, (c, t)| {
            b.iter(|| simd::contains_class(black_box(c), black_box(t)));
        });

        group.bench_with_input(
            BenchmarkId::new("scalar", name),
            &(classes, target),
            |b, (c, t)| {
                b.iter(|| scalar_contains_class(black_box(c), black_box(t)));
            },
        );
    }

    // Benchmark miss case (class not found)
    let classes = "container flex-row justify-center items-center bg-gray-100";
    group.bench_function("simd_miss", |b| {
        b.iter(|| simd::contains_class(black_box(classes), black_box("nonexistent")));
    });

    group.bench_function("scalar_miss", |b| {
        b.iter(|| scalar_contains_class(black_box(classes), black_box("nonexistent")));
    });

    group.finish();
}

#[cfg(feature = "simd")]
fn bench_text_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_search");

    for word_count in [100, 1000, 10000] {
        let content = generate_text_content(word_count);
        let needle = b"performance";
        group.throughput(Throughput::Bytes(content.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("simd", word_count),
            &(&content, needle),
            |b, (haystack, needle)| {
                b.iter(|| simd::find_text(black_box(haystack), black_box(*needle)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("scalar", word_count),
            &(&content, needle),
            |b, (haystack, needle)| {
                b.iter(|| scalar_find_text(black_box(haystack), black_box(*needle)));
            },
        );
    }

    group.finish();
}

#[cfg(feature = "simd")]
fn bench_text_finder_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_finder_reuse");

    let content = generate_text_content(10000);
    let needle = b"performance";

    // Single search
    group.bench_function("simd_single", |b| {
        b.iter(|| simd::find_text(black_box(&content), black_box(needle)));
    });

    // Repeated searches with TextFinder
    let finder = simd::TextFinder::new(needle);
    group.bench_function("simd_finder", |b| {
        b.iter(|| finder.find(black_box(&content)));
    });

    group.finish();
}

#[cfg(feature = "simd")]
fn bench_text_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_count");

    let content = generate_text_content(10000);
    let needle = b"hello";

    group.bench_function("simd", |b| {
        b.iter(|| simd::count_text(black_box(&content), black_box(needle)));
    });

    group.bench_function("scalar", |b| {
        b.iter(|| {
            let mut count = 0;
            let mut pos = 0;
            while pos + needle.len() <= content.len() {
                if &content[pos..pos + needle.len()] == needle {
                    count += 1;
                    pos += needle.len();
                } else {
                    pos += 1;
                }
            }
            count
        });
    });

    group.finish();
}

#[cfg(feature = "simd")]
fn bench_split_classes(c: &mut Criterion) {
    let mut group = c.benchmark_group("split_classes");

    let class_attrs = [
        ("small", "foo bar baz"),
        ("medium", "container flex-row justify-center items-center bg-gray-100"),
        (
            "large",
            "relative flex flex-col items-center justify-center min-h-screen bg-gradient-to-br \
             from-indigo-500 via-purple-500 to-pink-500 text-white font-bold shadow-lg rounded-xl \
             p-8",
        ),
    ];

    for (name, classes) in class_attrs {
        group.bench_with_input(BenchmarkId::new("simd", name), classes, |b, c| {
            b.iter(|| simd::split_classes(black_box(c)).count());
        });

        group.bench_with_input(BenchmarkId::new("scalar", name), classes, |b, c| {
            b.iter(|| black_box(c).split_whitespace().count());
        });
    }

    group.finish();
}

#[cfg(feature = "simd")]
criterion_group!(
    benches,
    bench_tag_scanning,
    bench_close_tag_scanning,
    bench_class_matching,
    bench_text_search,
    bench_text_finder_reuse,
    bench_text_count,
    bench_split_classes,
);

#[cfg(not(feature = "simd"))]
fn bench_placeholder(_c: &mut criterion::Criterion) {
    // Empty benchmark - SIMD feature is disabled
}

#[cfg(not(feature = "simd"))]
criterion_group!(benches, bench_placeholder);

criterion_main!(benches);
