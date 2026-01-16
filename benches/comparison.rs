//! Cross-platform benchmark suite for scrape-rs.
//!
//! This benchmark compares performance across different HTML sizes and query types.
//! Run with: `cargo bench --bench comparison`

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use scrape_core::Soup;

mod fixtures {
    use std::fmt::Write;

    pub fn small() -> String {
        r#"<!DOCTYPE html>
<html>
<head><title>Small Test Page</title></head>
<body>
    <div class="container">
        <h1 id="title">Welcome</h1>
        <p class="intro">This is a small test page.</p>
        <ul class="nav">
            <li><a href="/home">Home</a></li>
            <li><a href="/about">About</a></li>
            <li><a href="/contact">Contact</a></li>
        </ul>
    </div>
</body>
</html>"#
            .to_string()
    }

    pub fn medium() -> String {
        let mut html = String::from(
            r#"<!DOCTYPE html>
<html>
<head><title>Product Catalog</title></head>
<body>
<div class="catalog">
"#,
        );
        for i in 0..500 {
            let _ = write!(
                html,
                r#"<div class="product-card" id="product-{i}" data-category="cat-{cat}">
    <h2 class="product-title">Product {i}</h2>
    <p class="product-description">Description for product {i}</p>
    <span class="price">${price}</span>
    <button class="add-to-cart">Add to Cart</button>
</div>
"#,
                cat = i % 10,
                price = 10 + (i % 100)
            );
        }
        html.push_str("</div></body></html>");
        html
    }

    pub fn large() -> String {
        generate(10_000)
    }

    pub fn generate(element_count: usize) -> String {
        let mut html = String::from("<html><head><title>Large</title></head><body>");
        for i in 0..element_count {
            let _ =
                write!(html, r#"<div class="item-{i}" id="item-{i}"><span>Item {i}</span></div>"#);
        }
        html.push_str("</body></html>");
        html
    }
}

// ==================== Parse Benchmarks ====================

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    let small = fixtures::small();
    group.throughput(Throughput::Bytes(small.len() as u64));
    group.bench_function("small_1kb", |b| {
        b.iter(|| Soup::parse(black_box(&small)));
    });

    let medium = fixtures::medium();
    group.throughput(Throughput::Bytes(medium.len() as u64));
    group.bench_function("medium_100kb", |b| {
        b.iter(|| Soup::parse(black_box(&medium)));
    });

    let large = fixtures::large();
    group.throughput(Throughput::Bytes(large.len() as u64));
    group.bench_function("large_1mb", |b| {
        b.iter(|| Soup::parse(black_box(&large)));
    });

    group.finish();
}

// ==================== Find Benchmarks ====================

fn bench_find(c: &mut Criterion) {
    let mut group = c.benchmark_group("find");

    let medium = fixtures::medium();
    let soup = Soup::parse(&medium);

    group.bench_function("by_tag", |b| {
        b.iter(|| soup.find(black_box("div")));
    });

    group.bench_function("by_class", |b| {
        b.iter(|| soup.find(black_box(".product-card")));
    });

    group.bench_function("by_id", |b| {
        b.iter(|| soup.find(black_box("#product-100")));
    });

    group.bench_function("not_found", |b| {
        b.iter(|| soup.find(black_box(".nonexistent-class")));
    });

    group.finish();
}

// ==================== Find All Benchmarks ====================

fn bench_find_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_all");

    let medium = fixtures::medium();
    let soup = Soup::parse(&medium);

    group.bench_function("all_divs", |b| {
        b.iter(|| {
            let _ = soup.find_all(black_box("div")).unwrap();
        });
    });

    group.bench_function("all_products", |b| {
        b.iter(|| {
            let _ = soup.find_all(black_box(".product-card")).unwrap();
        });
    });

    group.bench_function("all_links", |b| {
        b.iter(|| {
            let _ = soup.find_all(black_box("a")).unwrap();
        });
    });

    group.finish();
}

// ==================== Select (CSS) Benchmarks ====================

fn bench_select(c: &mut Criterion) {
    let mut group = c.benchmark_group("select");

    let medium = fixtures::medium();
    let soup = Soup::parse(&medium);

    group.bench_function("simple_class", |b| {
        b.iter(|| {
            let _ = soup.select(black_box(".product-card")).unwrap();
        });
    });

    group.bench_function("descendant", |b| {
        b.iter(|| {
            let _ = soup.select(black_box(".product-grid .product-card")).unwrap();
        });
    });

    group.bench_function("child_combinator", |b| {
        b.iter(|| {
            let _ = soup.select(black_box(".product-grid > .product-card")).unwrap();
        });
    });

    group.bench_function("attribute", |b| {
        b.iter(|| {
            let _ = soup.select(black_box("[data-id]")).unwrap();
        });
    });

    group.bench_function("compound", |b| {
        b.iter(|| {
            let _ = soup.select(black_box("article.product-card[data-id]")).unwrap();
        });
    });

    group.bench_function("complex", |b| {
        b.iter(|| {
            let _ = soup
                .select(black_box(".main-content .product-grid > article.product-card"))
                .unwrap();
        });
    });

    group.finish();
}

// ==================== Navigation Benchmarks ====================

fn bench_navigation(c: &mut Criterion) {
    let mut group = c.benchmark_group("navigation");

    let medium = fixtures::medium();
    let soup = Soup::parse(&medium);

    group.bench_function("parent", |b| {
        let tag = soup.find("article").unwrap().unwrap();
        b.iter(|| black_box(tag.parent()));
    });

    group.bench_function("children", |b| {
        let tag = soup.find(".product-grid").unwrap().unwrap();
        b.iter(|| {
            let _: Vec<_> = tag.children().collect();
        });
    });

    group.bench_function("next_sibling", |b| {
        let tag = soup.find(".product-card").unwrap().unwrap();
        b.iter(|| black_box(tag.next_sibling()));
    });

    group.bench_function("prev_sibling", |b| {
        if let Ok(Some(tag)) = soup.find("#product-100") {
            b.iter(|| black_box(tag.prev_sibling()));
        }
    });

    group.bench_function("descendants", |b| {
        let tag = soup.find("#content").unwrap().unwrap();
        b.iter(|| {
            let _: Vec<_> = tag.descendants().collect();
        });
    });

    group.finish();
}

// ==================== Throughput Benchmarks ====================

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    for size in [100, 1_000, 10_000] {
        let html = fixtures::generate(size);
        group.throughput(Throughput::Bytes(html.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse", size), &html, |b, html| {
            b.iter(|| Soup::parse(black_box(html)));
        });
    }

    group.finish();
}

// ==================== Content Extraction Benchmarks ====================

fn bench_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("extraction");

    let medium = fixtures::medium();
    let soup = Soup::parse(&medium);
    let tag = soup.find(".product-card").unwrap().unwrap();

    group.bench_function("text", |b| {
        b.iter(|| black_box(tag.text()));
    });

    group.bench_function("inner_html", |b| {
        b.iter(|| black_box(tag.inner_html()));
    });

    group.bench_function("outer_html", |b| {
        b.iter(|| black_box(tag.outer_html()));
    });

    group.bench_function("get_attr", |b| {
        b.iter(|| black_box(tag.get("data-id")));
    });

    group.bench_function("has_class", |b| {
        b.iter(|| black_box(tag.has_class("product-card")));
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(200);
    targets = bench_parse,
              bench_find,
              bench_find_all,
              bench_select,
              bench_navigation,
              bench_throughput,
              bench_extraction
}
criterion_main!(benches);
