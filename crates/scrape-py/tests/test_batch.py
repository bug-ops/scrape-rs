"""Tests for batch parsing."""

from scrape_rs import Soup, parse_batch


class TestParseBatch:
    def test_parse_batch_simple(self):
        htmls = [
            "<div>A</div>",
            "<div>B</div>",
            "<div>C</div>",
        ]
        soups = parse_batch(htmls)
        assert len(soups) == 3
        texts = [s.find("div").text for s in soups]
        assert texts == ["A", "B", "C"]

    def test_parse_batch_preserves_order(self):
        htmls = [f"<div>{i}</div>" for i in range(100)]
        soups = parse_batch(htmls)
        texts = [s.find("div").text for s in soups]
        assert texts == [str(i) for i in range(100)]

    def test_parse_batch_with_threads(self):
        htmls = ["<div>Test</div>"] * 50
        soups = parse_batch(htmls, n_threads=4)
        assert len(soups) == 50

    def test_parse_batch_empty_list(self):
        soups = parse_batch([])
        assert soups == []

    def test_parse_batch_single_document(self):
        soups = parse_batch(["<div>Single</div>"])
        assert len(soups) == 1
        assert soups[0].find("div").text == "Single"

    def test_parse_batch_large_documents(self):
        # Create larger HTML documents
        html_template = """
        <html>
        <head><title>Doc {}</title></head>
        <body>
            <div class="container">
                <ul>
                    <li>Item 1</li>
                    <li>Item 2</li>
                    <li>Item 3</li>
                </ul>
            </div>
        </body>
        </html>
        """
        htmls = [html_template.format(i) for i in range(20)]
        soups = parse_batch(htmls)
        assert len(soups) == 20
        for i, soup in enumerate(soups):
            assert soup.title == f"Doc {i}"

    def test_parse_batch_returns_soup_instances(self):
        htmls = ["<div>Test</div>"]
        soups = parse_batch(htmls)
        assert isinstance(soups[0], Soup)

    def test_parse_batch_soups_are_independent(self):
        htmls = ["<div id='a'>A</div>", "<div id='b'>B</div>"]
        soups = parse_batch(htmls)

        # Each soup should have its own document
        assert soups[0].find("#a") is not None
        assert soups[0].find("#b") is None
        assert soups[1].find("#a") is None
        assert soups[1].find("#b") is not None
