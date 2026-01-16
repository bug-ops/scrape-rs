#!/usr/bin/env python3
"""Generate benchmark HTML fixtures of various sizes."""

import os
from pathlib import Path

FIXTURES_DIR = Path(__file__).parent


def generate_medium_html() -> str:
    """Generate ~100KB HTML document with typical webpage structure."""
    parts = [
        '<!DOCTYPE html>',
        '<html lang="en">',
        '<head>',
        '    <meta charset="UTF-8">',
        '    <meta name="viewport" content="width=device-width, initial-scale=1.0">',
        '    <title>Medium Test Page - Product Catalog</title>',
        '    <meta name="description" content="A medium-sized test page for benchmarking HTML parsing performance.">',
        '</head>',
        '<body>',
        '<header class="header">',
        '    <nav class="main-nav" id="navigation">',
        '        <a href="/" class="logo">Company Logo</a>',
        '        <ul class="nav-menu">',
    ]

    for i in range(20):
        parts.append(f'            <li class="nav-item"><a href="/category-{i}" class="nav-link">Category {i}</a></li>')

    parts.extend([
        '        </ul>',
        '        <div class="search-box">',
        '            <input type="search" class="search-input" placeholder="Search products...">',
        '            <button type="submit" class="search-btn">Search</button>',
        '        </div>',
        '    </nav>',
        '</header>',
        '<main id="content" class="main-content">',
        '    <aside class="sidebar" id="filters">',
        '        <h2 class="sidebar-title">Filters</h2>',
    ])

    for i in range(10):
        parts.append(f'        <div class="filter-group" data-filter="group-{i}">')
        parts.append(f'            <h3 class="filter-title">Filter {i}</h3>')
        for j in range(5):
            parts.append(f'            <label class="filter-option"><input type="checkbox" name="filter-{i}" value="{j}"> Option {j}</label>')
        parts.append('        </div>')

    parts.extend([
        '    </aside>',
        '    <section class="products" id="product-grid">',
        '        <h1 class="page-title">Products</h1>',
        '        <div class="product-grid">',
    ])

    for i in range(200):
        parts.append(f'            <article class="product-card" id="product-{i}" data-id="{i}" data-category="cat-{i % 10}">')
        parts.append(f'                <img src="/images/product-{i}.jpg" alt="Product {i}" class="product-image" loading="lazy">')
        parts.append(f'                <div class="product-info">')
        parts.append(f'                    <h2 class="product-title">Product Name {i}</h2>')
        parts.append(f'                    <p class="product-description">This is the description for product {i}. It includes detailed information about features, specifications, and benefits.</p>')
        parts.append(f'                    <div class="product-meta">')
        parts.append(f'                        <span class="product-price" data-price="{i * 10 + 99}">${i * 10 + 99}.00</span>')
        parts.append(f'                        <span class="product-rating" data-rating="{(i % 5) + 1}">{"*" * ((i % 5) + 1)}</span>')
        parts.append(f'                    </div>')
        parts.append(f'                    <div class="product-actions">')
        parts.append(f'                        <button class="btn btn-primary add-to-cart" data-product="{i}">Add to Cart</button>')
        parts.append(f'                        <button class="btn btn-secondary wishlist" data-product="{i}">Wishlist</button>')
        parts.append(f'                    </div>')
        parts.append(f'                </div>')
        parts.append(f'            </article>')

    parts.extend([
        '        </div>',
        '        <nav class="pagination" aria-label="Product pages">',
    ])

    for i in range(1, 21):
        parts.append(f'            <a href="/page/{i}" class="page-link{"  active" if i == 1 else ""}">{i}</a>')

    parts.extend([
        '        </nav>',
        '    </section>',
        '</main>',
        '<footer class="footer" id="footer">',
        '    <div class="footer-content">',
        '        <div class="footer-section">',
        '            <h3>About Us</h3>',
        '            <p>Company description and information about our services and products.</p>',
        '        </div>',
        '        <div class="footer-section">',
        '            <h3>Quick Links</h3>',
        '            <ul class="footer-links">',
    ])

    for i in range(10):
        parts.append(f'                <li><a href="/link-{i}">Footer Link {i}</a></li>')

    parts.extend([
        '            </ul>',
        '        </div>',
        '        <div class="footer-section">',
        '            <h3>Contact</h3>',
        '            <address>',
        '                <p>123 Main Street</p>',
        '                <p>City, State 12345</p>',
        '                <p><a href="mailto:contact@example.com">contact@example.com</a></p>',
        '            </address>',
        '        </div>',
        '    </div>',
        '    <div class="footer-bottom">',
        '        <p>&copy; 2026 Company Name. All rights reserved.</p>',
        '    </div>',
        '</footer>',
        '</body>',
        '</html>',
    ])

    return '\n'.join(parts)


def generate_large_html() -> str:
    """Generate ~1MB HTML document with many elements."""
    parts = [
        '<!DOCTYPE html>',
        '<html lang="en">',
        '<head>',
        '    <meta charset="UTF-8">',
        '    <meta name="viewport" content="width=device-width, initial-scale=1.0">',
        '    <title>Large Test Page - Data Table</title>',
        '</head>',
        '<body>',
        '<div id="app" class="app-container">',
    ]

    for section in range(10):
        parts.append(f'    <section class="data-section" id="section-{section}" data-section="{section}">')
        parts.append(f'        <h2 class="section-title">Data Section {section}</h2>')
        parts.append(f'        <table class="data-table" id="table-{section}">')
        parts.append('            <thead>')
        parts.append('                <tr>')
        for col in range(10):
            parts.append(f'                    <th class="col-header" data-col="{col}">Column {col}</th>')
        parts.append('                </tr>')
        parts.append('            </thead>')
        parts.append('            <tbody>')

        for row in range(500):
            row_id = section * 500 + row
            parts.append(f'                <tr class="data-row {"even" if row % 2 == 0 else "odd"}" id="row-{row_id}" data-row="{row_id}">')
            for col in range(10):
                cell_id = row_id * 10 + col
                parts.append(f'                    <td class="data-cell" id="cell-{cell_id}" data-value="{cell_id}">Data {cell_id}</td>')
            parts.append('                </tr>')

        parts.append('            </tbody>')
        parts.append('        </table>')

        parts.append(f'        <div class="card-grid" id="cards-{section}">')
        for card in range(100):
            card_id = section * 100 + card
            parts.append(f'            <div class="card" id="card-{card_id}" data-id="{card_id}">')
            parts.append(f'                <div class="card-header">')
            parts.append(f'                    <h3 class="card-title">Card {card_id}</h3>')
            parts.append(f'                    <span class="card-badge badge-{card % 5}">Badge</span>')
            parts.append(f'                </div>')
            parts.append(f'                <div class="card-body">')
            parts.append(f'                    <p class="card-text">Card content {card_id} with additional text to increase document size.</p>')
            parts.append(f'                    <ul class="card-list">')
            for item in range(3):
                parts.append(f'                        <li class="list-item">Item {item}</li>')
            parts.append(f'                    </ul>')
            parts.append(f'                </div>')
            parts.append(f'                <div class="card-footer">')
            parts.append(f'                    <button class="btn btn-action" data-action="edit" data-card="{card_id}">Edit</button>')
            parts.append(f'                    <button class="btn btn-action" data-action="delete" data-card="{card_id}">Delete</button>')
            parts.append(f'                </div>')
            parts.append(f'            </div>')
        parts.append('        </div>')

        parts.append('    </section>')

    parts.extend([
        '</div>',
        '</body>',
        '</html>',
    ])

    return '\n'.join(parts)


def main():
    medium_html = generate_medium_html()
    medium_path = FIXTURES_DIR / 'medium.html'
    medium_path.write_text(medium_html)
    print(f"Generated {medium_path}: {len(medium_html):,} bytes ({len(medium_html) / 1024:.1f} KB)")

    large_html = generate_large_html()
    large_path = FIXTURES_DIR / 'large.html'
    large_path.write_text(large_html)
    print(f"Generated {large_path}: {len(large_html):,} bytes ({len(large_html) / 1024:.1f} KB)")


if __name__ == '__main__':
    main()
