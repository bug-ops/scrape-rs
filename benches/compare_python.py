#!/usr/bin/env python3
"""Compare scrape_rs performance against BeautifulSoup.

Run from project root:
    cd crates/scrape-py && uv run maturin develop
    python benches/compare_python.py
"""

import sys
import time
from pathlib import Path
from statistics import mean, stdev
from typing import Callable

FIXTURES_DIR = Path(__file__).parent / "fixtures"


def timed(fn: Callable, iterations: int = 100, warmup: int = 10) -> tuple[float, float]:
    """Run function multiple times and return mean/stdev in milliseconds."""
    # Warmup iterations to stabilize JIT and cache
    for _ in range(warmup):
        fn()

    # Actual measurement
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        fn()
        elapsed = (time.perf_counter() - start) * 1000
        times.append(elapsed)
    return mean(times), stdev(times)


def load_fixture(name: str) -> str:
    """Load HTML fixture file."""
    return (FIXTURES_DIR / name).read_text()


def run_benchmarks():
    """Run comparison benchmarks."""
    # Import libraries
    try:
        from scrape_rs import Soup
    except ImportError:
        print("Error: scrape_rs not found. Run:")
        print("  cd crates/scrape-py && uv run maturin develop")
        sys.exit(1)

    try:
        from bs4 import BeautifulSoup
    except ImportError:
        print("Error: beautifulsoup4 not found. Install with:")
        print("  pip install beautifulsoup4")
        sys.exit(1)

    try:
        import lxml.html  # noqa: F401
        has_lxml = True
    except ImportError:
        has_lxml = False
        print("Note: lxml not found, skipping lxml benchmarks\n")

    # Load fixtures
    small_html = load_fixture("small.html")
    medium_html = load_fixture("medium.html")
    large_html = load_fixture("large.html")

    results = []

    print("=" * 60)
    print("scrape_rs vs BeautifulSoup Benchmark")
    print("=" * 60)
    print()

    # Parse benchmarks
    print("=== Parse Benchmarks ===")
    print()

    for name, html in [("small", small_html), ("medium", medium_html), ("large", large_html)]:
        size_kb = len(html) / 1024

        # scrape_rs
        def parse_scrape():
            Soup(html)

        scrape_mean, scrape_std = timed(parse_scrape, iterations=100)

        # BeautifulSoup (html.parser)
        def parse_bs4():
            BeautifulSoup(html, "html.parser")

        bs4_mean, bs4_std = timed(parse_bs4, iterations=100)

        speedup = bs4_mean / scrape_mean if scrape_mean > 0 else float("inf")

        print(f"{name} ({size_kb:.1f} KB):")
        print(f"  scrape_rs:     {scrape_mean:7.3f}ms +/- {scrape_std:.3f}ms")
        print(f"  BeautifulSoup: {bs4_mean:7.3f}ms +/- {bs4_std:.3f}ms")
        print(f"  Speedup:       {speedup:.1f}x")
        print()

        results.append(("parse", name, speedup))

        # lxml comparison for parse
        if has_lxml:
            def parse_lxml():
                import lxml.html
                lxml.html.fromstring(html)

            lxml_mean, lxml_std = timed(parse_lxml, iterations=100)
            lxml_speedup = lxml_mean / scrape_mean if scrape_mean > 0 else float("inf")
            print(f"  lxml:          {lxml_mean:7.3f}ms +/- {lxml_std:.3f}ms")
            print(f"  vs lxml:       {lxml_speedup:.1f}x")
            print()

    # Query benchmarks
    print("=== Query Benchmarks (on medium.html) ===")
    print()

    soup_scrape = Soup(medium_html)
    soup_bs4 = BeautifulSoup(medium_html, "html.parser")

    query_tests = [
        ("find div", lambda s: s.find("div")),
        ("find .product-card", lambda s: s.find(".product-card") if hasattr(s, "find") and callable(getattr(s, "find")) else s.select_one(".product-card")),
        ("find #product-100", lambda s: s.find("#product-100") if hasattr(s, "find") and callable(getattr(s, "find")) else s.select_one("#product-100")),
        ("find_all div", lambda s: s.find_all("div") if hasattr(s, "find_all") else s.select("div")),
        ("select .product-card", lambda s: s.select(".product-card")),
    ]

    for name, fn in query_tests:
        # scrape_rs
        if "." in name.split()[-1] or "#" in name.split()[-1]:
            # CSS selector query
            if name.startswith("find "):
                selector = name.split()[-1]
                def scrape_fn():
                    soup_scrape.find(selector)
            else:
                selector = name.split()[-1]
                def scrape_fn():
                    soup_scrape.select(selector)
        else:
            # Tag query
            tag = name.split()[-1]
            if name.startswith("find_all"):
                def scrape_fn():
                    soup_scrape.find_all(tag)
            else:
                def scrape_fn():
                    soup_scrape.find(tag)

        scrape_mean, scrape_std = timed(scrape_fn, iterations=100)

        # BeautifulSoup
        def bs4_fn():
            fn(soup_bs4)

        bs4_mean, bs4_std = timed(bs4_fn, iterations=100)

        speedup = bs4_mean / scrape_mean if scrape_mean > 0 else float("inf")

        print(f"{name}:")
        print(f"  scrape_rs:     {scrape_mean:7.3f}ms +/- {scrape_std:.3f}ms")
        print(f"  BeautifulSoup: {bs4_mean:7.3f}ms +/- {bs4_std:.3f}ms")
        print(f"  Speedup:       {speedup:.1f}x")
        print()

        results.append(("query", name, speedup))

    # Summary
    print("=" * 60)
    print("Summary")
    print("=" * 60)
    print()

    parse_speedups = [r[2] for r in results if r[0] == "parse"]
    query_speedups = [r[2] for r in results if r[0] == "query"]

    avg_parse = mean(parse_speedups) if parse_speedups else 0
    avg_query = mean(query_speedups) if query_speedups else 0

    print(f"Average Parse Speedup:  {avg_parse:.1f}x faster than BeautifulSoup")
    print(f"Average Query Speedup:  {avg_query:.1f}x faster than BeautifulSoup")
    print()

    if avg_parse >= 10:
        print("Parse target (10x+): PASSED")
    else:
        print(f"Parse target (10x+): FAILED ({avg_parse:.1f}x)")

    if avg_query >= 10:
        print("Query target (10x+): PASSED")
    else:
        print(f"Query target (10x+): MISSED ({avg_query:.1f}x)")


if __name__ == "__main__":
    run_benchmarks()
