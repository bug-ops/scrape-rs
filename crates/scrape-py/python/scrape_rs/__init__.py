"""
scrape_rs - High-performance HTML parsing library.

This module provides fast HTML parsing with CSS selector support,
built on a Rust core for maximum performance.

Example:
    >>> from scrape_rs import Soup
    >>> soup = Soup("<html><body><h1>Hello!</h1></body></html>")
    >>> print(soup.title)
    None
"""

from scrape_rs._core import Soup, SoupConfig, parse_batch

__all__ = [
    "Soup",
    "SoupConfig",
    "parse_batch",
]

__version__ = "0.1.0"
