"""Basic tests for scrape-rs Python bindings."""


def test_import() -> None:
    """Test that the module can be imported."""
    import scrape_rs

    assert scrape_rs is not None
