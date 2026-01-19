"""
scrape_rs - High-performance HTML parsing library.

A Rust-powered HTML parser with BeautifulSoup-like API.

Example:
    >>> from scrape_rs import Soup
    >>> soup = Soup("<div class='hello'>World</div>")
    >>> print(soup.find("div").text)
    World
"""

from scrape_rs._core import (
    CompiledSelector,
    Soup,
    SoupConfig,
    Tag,
    __version__,
    compile_selector,
    parse_batch,
)

__all__ = [
    "CompiledSelector",
    "Soup",
    "SoupConfig",
    "Tag",
    "compile_selector",
    "parse_batch",
    "__version__",
]
