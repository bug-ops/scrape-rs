//! Phase 14 performance benchmarks.
//!
//! Comprehensive benchmarks for measuring optimization improvements:
//! - P0: Arena capacity hints, inline functions, text_into()
//! - P1: SIMD class matching, tag name interning, ID/class indexes
//! - P2: Rayon batch parsing, selector matching shortcuts

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use scrape_core::{CompiledSelector, Soup};

fn small_html() -> &'static str {
    r#"<!DOCTYPE html>
<html>
<head><title>Small Page</title></head>
<body>
  <div class="container" id="main">
    <h1>Hello World</h1>
    <p class="text">Some text here</p>
    <ul>
      <li>Item 1</li>
      <li>Item 2</li>
      <li>Item 3</li>
    </ul>
  </div>
</body>
</html>"#
}

fn medium_html() -> String {
    use std::fmt::Write as _;

    let mut html = String::from("<html><body>");
    for i in 0..100 {
        write!(
            &mut html,
            r#"<div class="product" id="product-{}">
                <h2>Product {}</h2>
                <span class="price">${}.99</span>
                <p class="description">Product description for item {}</p>
              </div>"#,
            i,
            i,
            i * 10 + 50,
            i
        )
        .unwrap();
    }
    html.push_str("</body></html>");
    html
}

fn large_html() -> String {
    use std::fmt::Write as _;

    let mut html = String::from("<html><body><table>");
    for i in 0..1000 {
        write!(
            &mut html,
            r#"<tr id="row-{}">
                <td class="col1">{}</td>
                <td class="col2">Cell-{}</td>
                <td class="col3">{}</td>
              </tr>"#,
            i,
            i,
            i,
            i * 2
        )
        .unwrap();
    }
    html.push_str("</table></body></html>");
    html
}

fn bench_parse_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    let small = small_html();
    group.throughput(Throughput::Bytes(small.len() as u64));
    group.bench_with_input(BenchmarkId::new("small", "1kb"), &small, |b, html| {
        b.iter(|| Soup::parse(black_box(html)));
    });

    let medium = medium_html();
    group.throughput(Throughput::Bytes(medium.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("medium", format!("{}kb", medium.len() / 1024)),
        &medium,
        |b, html| {
            b.iter(|| Soup::parse(black_box(html)));
        },
    );

    let large = large_html();
    group.throughput(Throughput::Bytes(large.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("large", format!("{}kb", large.len() / 1024)),
        &large,
        |b, html| {
            b.iter(|| Soup::parse(black_box(html)));
        },
    );

    group.finish();
}

fn bench_find_by_selector(c: &mut Criterion) {
    let mut group = c.benchmark_group("find");

    let medium = medium_html();
    let soup = Soup::parse(&medium);

    group.bench_function("by_id", |b| {
        b.iter(|| {
            let result = soup.find(black_box("#product-50")).unwrap();
            black_box(result);
        });
    });

    group.bench_function("by_class", |b| {
        b.iter(|| {
            let result = soup.find(black_box(".product")).unwrap();
            black_box(result);
        });
    });

    group.bench_function("by_tag", |b| {
        b.iter(|| {
            let result = soup.find(black_box("div")).unwrap();
            black_box(result);
        });
    });

    group.bench_function("complex_selector", |b| {
        b.iter(|| {
            let result = soup.find(black_box("div.product > h2")).unwrap();
            black_box(result);
        });
    });

    group.finish();
}

fn bench_find_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_all");

    let medium = medium_html();
    let soup = Soup::parse(&medium);

    group.bench_function("all_divs", |b| {
        b.iter(|| {
            let results = soup.find_all(black_box("div")).unwrap();
            black_box(results.len());
        });
    });

    group.bench_function("by_class", |b| {
        b.iter(|| {
            let results = soup.find_all(black_box(".product")).unwrap();
            black_box(results.len());
        });
    });

    group.bench_function("complex_selector", |b| {
        b.iter(|| {
            let results = soup.find_all(black_box("div.product > span.price")).unwrap();
            black_box(results.len());
        });
    });

    group.finish();
}

fn bench_compiled_selectors(c: &mut Criterion) {
    let mut group = c.benchmark_group("compiled_selector");

    let medium = medium_html();
    let soup = Soup::parse(&medium);

    let selector_class = CompiledSelector::compile(".product").unwrap();
    group.bench_function("find_all_by_class", |b| {
        b.iter(|| {
            let results = soup.select_compiled(black_box(&selector_class));
            black_box(results.len());
        });
    });

    let selector_complex = CompiledSelector::compile("div.product > h2").unwrap();
    group.bench_function("find_all_complex", |b| {
        b.iter(|| {
            let results = soup.select_compiled(black_box(&selector_complex));
            black_box(results.len());
        });
    });

    group.finish();
}

fn bench_text_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_extraction");

    let medium = medium_html();
    let soup = Soup::parse(&medium);
    let divs = soup.find_all("div.product").unwrap();

    group.bench_function("text_allocating", |b| {
        b.iter(|| {
            for div in &divs {
                let text = div.text();
                black_box(text);
            }
        });
    });

    group.bench_function("text_into_reuse", |b| {
        b.iter(|| {
            let mut buffer = String::new();
            for div in &divs {
                buffer.clear();
                div.text_into(&mut buffer);
                black_box(&buffer);
            }
        });
    });

    group.finish();
}

fn bench_navigation(c: &mut Criterion) {
    let mut group = c.benchmark_group("navigation");

    let medium = medium_html();
    let soup = Soup::parse(&medium);
    let root = soup.root().unwrap();

    group.bench_function("descendants", |b| {
        b.iter(|| {
            let count = root.descendants().count();
            black_box(count);
        });
    });

    let first_div = soup.find("div").unwrap().unwrap();
    group.bench_function("ancestors", |b| {
        b.iter(|| {
            let count = first_div.parents().count();
            black_box(count);
        });
    });

    group.bench_function("siblings", |b| {
        b.iter(|| {
            let count = first_div.siblings().count();
            black_box(count);
        });
    });

    group.finish();
}

fn bench_has_class(c: &mut Criterion) {
    let mut group = c.benchmark_group("has_class");

    let html = r#"<div class="foo bar baz qux container product item active highlight"></div>"#;
    let soup = Soup::parse(html);
    let div = soup.find("div").unwrap().unwrap();

    group.bench_function("existing_first", |b| {
        b.iter(|| {
            let result = div.has_class(black_box("foo"));
            black_box(result);
        });
    });

    group.bench_function("existing_middle", |b| {
        b.iter(|| {
            let result = div.has_class(black_box("item"));
            black_box(result);
        });
    });

    group.bench_function("existing_last", |b| {
        b.iter(|| {
            let result = div.has_class(black_box("highlight"));
            black_box(result);
        });
    });

    group.bench_function("not_found", |b| {
        b.iter(|| {
            let result = div.has_class(black_box("nonexistent"));
            black_box(result);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_sizes,
    bench_find_by_selector,
    bench_find_all,
    bench_compiled_selectors,
    bench_text_extraction,
    bench_navigation,
    bench_has_class,
);
criterion_main!(benches);
