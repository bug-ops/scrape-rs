//! Phase 16 performance benchmarks - Trait Abstractions.
//!
//! Benchmarks for:
//! - HtmlSerializer trait methods vs direct function calls
//! - ElementFilter iterators (.elements()) vs manual filter_map
//! - Verify #[inline] annotations provide zero-overhead abstraction

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use scrape_core::{
    Soup,
    serialize::{HtmlSerializer, collect_text, serialize_inner_html, serialize_node},
};

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

fn nested_html() -> String {
    use std::fmt::Write as _;

    let mut html = String::from("<html><body><ul>");
    for i in 0..50 {
        write!(&mut html, r#"<li>Item {}<ul>"#, i).unwrap();
        for j in 0..5 {
            write!(&mut html, r#"<li>Subitem {}-{}</li>"#, i, j).unwrap();
        }
        html.push_str("</ul></li>");
    }
    html.push_str("</ul></body></html>");
    html
}

/// Benchmark HtmlSerializer trait methods vs direct function calls
fn bench_html_serializer_trait(c: &mut Criterion) {
    let mut group = c.benchmark_group("html_serializer");

    let html = medium_html();
    let soup = Soup::parse(&html);
    let divs = soup.find_all("div.product").unwrap();

    // Outer HTML: trait vs direct
    group.bench_function("trait_serialize_html", |b| {
        b.iter(|| {
            for div in &divs {
                let result = div.serialize_html();
                black_box(result);
            }
        });
    });

    group.bench_function("direct_serialize_node", |b| {
        b.iter(|| {
            for div in &divs {
                let mut buf = String::new();
                serialize_node(soup.document(), div.node_id(), &mut buf);
                black_box(buf);
            }
        });
    });

    // Inner HTML: trait vs direct
    group.bench_function("trait_serialize_inner", |b| {
        b.iter(|| {
            for div in &divs {
                let result = div.serialize_inner();
                black_box(result);
            }
        });
    });

    group.bench_function("direct_serialize_inner_html", |b| {
        b.iter(|| {
            for div in &divs {
                let mut buf = String::new();
                serialize_inner_html(soup.document(), div.node_id(), &mut buf);
                black_box(buf);
            }
        });
    });

    // Text extraction: trait vs direct
    group.bench_function("trait_extract_text", |b| {
        b.iter(|| {
            for div in &divs {
                let result = div.extract_text();
                black_box(result);
            }
        });
    });

    group.bench_function("direct_collect_text", |b| {
        b.iter(|| {
            for div in &divs {
                let mut buf = String::new();
                collect_text(soup.document(), div.node_id(), &mut buf);
                black_box(buf);
            }
        });
    });

    // Buffer reuse: _into methods
    group.bench_function("trait_serialize_html_into_reuse", |b| {
        b.iter(|| {
            let mut buf = String::new();
            for div in &divs {
                buf.clear();
                div.serialize_html_into(&mut buf);
                black_box(&buf);
            }
        });
    });

    group.bench_function("trait_extract_text_into_reuse", |b| {
        b.iter(|| {
            let mut buf = String::new();
            for div in &divs {
                buf.clear();
                div.extract_text_into(&mut buf);
                black_box(&buf);
            }
        });
    });

    group.finish();
}

/// Benchmark ElementFilter iterators vs manual filter_map
fn bench_element_filter_iterators(c: &mut Criterion) {
    let mut group = c.benchmark_group("element_filter");

    let html = nested_html();
    let soup = Soup::parse(&html);
    let ul = soup.find("ul").unwrap().unwrap();
    let doc = soup.document();
    let ul_id = ul.node_id();

    // Children: .elements() vs filter_map
    group.bench_function("children_elements_extension", |b| {
        b.iter(|| {
            let count = doc.children(ul_id).elements().count();
            black_box(count);
        });
    });

    group.bench_function("children_manual_filter_map", |b| {
        b.iter(|| {
            let count = doc
                .children(ul_id)
                .filter_map(|id| {
                    let node = doc.get(id)?;
                    if node.kind.is_element() { Some(id) } else { None }
                })
                .count();
            black_box(count);
        });
    });

    // Descendants: .elements() vs filter_map
    group.bench_function("descendants_elements_extension", |b| {
        b.iter(|| {
            let count = doc.descendants(ul_id).elements().count();
            black_box(count);
        });
    });

    group.bench_function("descendants_manual_filter_map", |b| {
        b.iter(|| {
            let count = doc
                .descendants(ul_id)
                .filter_map(|id| {
                    let node = doc.get(id)?;
                    if node.kind.is_element() { Some(id) } else { None }
                })
                .count();
            black_box(count);
        });
    });

    // Ancestors: .elements() vs filter_map
    let deep_li = soup.find_all("li").unwrap().last().cloned().unwrap();
    let deep_li_id = deep_li.node_id();
    group.bench_function("ancestors_elements_extension", |b| {
        b.iter(|| {
            let count = doc.ancestors(deep_li_id).elements().count();
            black_box(count);
        });
    });

    group.bench_function("ancestors_manual_filter_map", |b| {
        b.iter(|| {
            let count = doc
                .ancestors(deep_li_id)
                .filter_map(|id| {
                    let node = doc.get(id)?;
                    if node.kind.is_element() { Some(id) } else { None }
                })
                .count();
            black_box(count);
        });
    });

    group.finish();
}

/// Benchmark binding navigation methods (using ElementFilter)
#[allow(clippy::needless_collect)]
fn bench_binding_navigation(c: &mut Criterion) {
    let mut group = c.benchmark_group("binding_navigation");

    let html = nested_html();
    let soup = Soup::parse(&html);
    let ul = soup.find("ul").unwrap().unwrap();
    let doc = soup.document();
    let ul_id = ul.node_id();

    // These simulate what bindings now do with simplified code
    group.bench_function("element_children", |b| {
        b.iter(|| {
            let children: Vec<_> = doc.children(ul_id).elements().collect();
            black_box(children.len());
        });
    });

    group.bench_function("element_descendants", |b| {
        b.iter(|| {
            let descendants: Vec<_> = doc.descendants(ul_id).elements().collect();
            black_box(descendants.len());
        });
    });

    let deep_li = soup.find_all("li").unwrap().last().cloned().unwrap();
    let deep_li_id = deep_li.node_id();
    group.bench_function("element_parents", |b| {
        b.iter(|| {
            let parents: Vec<_> = doc.ancestors(deep_li_id).elements().collect();
            black_box(parents.len());
        });
    });

    let first_li = soup.find("li").unwrap().unwrap();
    let first_li_id = first_li.node_id();
    group.bench_function("element_next_siblings", |b| {
        b.iter(|| {
            let siblings: Vec<_> = doc.next_siblings(first_li_id).elements().collect();
            black_box(siblings.len());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_html_serializer_trait,
    bench_element_filter_iterators,
    bench_binding_navigation,
);
criterion_main!(benches);
