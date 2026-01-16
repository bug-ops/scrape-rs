"""Shared fixtures for scrape_rs tests."""

import pytest

from scrape_rs import Soup


@pytest.fixture
def simple_html():
    return "<html><body><div>Hello</div></body></html>"


@pytest.fixture
def complex_html():
    return """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Test Page</title>
    </head>
    <body>
        <div class="container" id="main">
            <h1>Title</h1>
            <ul class="list">
                <li class="item active">First</li>
                <li class="item">Second</li>
                <li class="item">Third</li>
            </ul>
            <a href="/link" class="nav-link">Click me</a>
        </div>
        <footer id="footer">
            <span>Copyright 2026</span>
        </footer>
    </body>
    </html>
    """


@pytest.fixture
def simple_soup(simple_html):
    return Soup(simple_html)


@pytest.fixture
def complex_soup(complex_html):
    return Soup(complex_html)
