"""Type stubs for scrape_rs."""

from collections.abc import Iterator

__version__: str

class SoupConfig:
    """Configuration options for HTML parsing."""

    max_depth: int
    strict_mode: bool
    preserve_whitespace: bool
    include_comments: bool

    def __init__(
        self,
        max_depth: int = 512,
        strict_mode: bool = False,
        preserve_whitespace: bool = False,
        include_comments: bool = False,
    ) -> None: ...
    def __repr__(self) -> str: ...

class Tag:
    """An HTML element in the document."""

    @property
    def name(self) -> str | None:
        """Get the tag name (e.g., 'div', 'span')."""
        ...

    @property
    def text(self) -> str:
        """Get text content of this element and all descendants."""
        ...

    @property
    def inner_html(self) -> str:
        """Get inner HTML content (excluding this element's tags)."""
        ...

    @property
    def outer_html(self) -> str:
        """Get outer HTML (including this element's tags)."""
        ...

    @property
    def attrs(self) -> dict[str, str]:
        """Get all attributes as a dictionary."""
        ...

    @property
    def classes(self) -> list[str]:
        """Get all CSS classes as a list."""
        ...

    @property
    def parent(self) -> Tag | None:
        """Get the parent element."""
        ...

    @property
    def children(self) -> list[Tag]:
        """Get all child elements."""
        ...

    @property
    def next_sibling(self) -> Tag | None:
        """Get the next sibling element."""
        ...

    @property
    def prev_sibling(self) -> Tag | None:
        """Get the previous sibling element."""
        ...

    @property
    def descendants(self) -> list[Tag]:
        """Get all descendant elements."""
        ...

    def get(self, name: str) -> str | None:
        """Get attribute value by name."""
        ...

    def has_attr(self, name: str) -> bool:
        """Check if attribute exists."""
        ...

    def has_class(self, class_name: str) -> bool:
        """Check if element has a specific CSS class."""
        ...

    def find(self, selector: str) -> Tag | None:
        """Find first descendant matching CSS selector."""
        ...

    def find_all(self, selector: str) -> list[Tag]:
        """Find all descendants matching CSS selector."""
        ...

    def select(self, selector: str) -> list[Tag]:
        """Find all descendants matching CSS selector (alias for find_all)."""
        ...

    def __getitem__(self, name: str) -> str:
        """Get attribute value using dict-like access.

        Raises:
            KeyError: If attribute not found.
        """
        ...

    def __contains__(self, name: str) -> bool:
        """Check if attribute exists."""
        ...

    def __len__(self) -> int:
        """Get number of child elements."""
        ...

    def __iter__(self) -> Iterator[Tag]:
        """Iterate over child elements."""
        ...

    def __eq__(self, other: object) -> bool:
        """Check if two tags reference the same element."""
        ...

    def __hash__(self) -> int:
        """Get hash value for use in sets and dicts."""
        ...

    def __repr__(self) -> str:
        """Get string representation for debugging."""
        ...

class Soup:
    """A parsed HTML document."""

    def __init__(
        self,
        html: str,
        config: SoupConfig | None = None,
    ) -> None:
        """Parse an HTML string.

        Args:
            html: HTML string to parse.
            config: Optional parsing configuration.
        """
        ...

    @staticmethod
    def from_file(
        path: str,
        config: SoupConfig | None = None,
    ) -> Soup:
        """Parse HTML from a file.

        Args:
            path: Path to the HTML file.
            config: Optional parsing configuration.

        Returns:
            A new Soup instance.

        Raises:
            ValueError: If the file cannot be read.
        """
        ...

    @property
    def root(self) -> Tag | None:
        """Get the root element (usually <html>)."""
        ...

    @property
    def title(self) -> str | None:
        """Get the document title."""
        ...

    @property
    def text(self) -> str:
        """Get all text content with tags stripped."""
        ...

    def find(self, selector: str) -> Tag | None:
        """Find the first element matching a CSS selector.

        Args:
            selector: CSS selector string.

        Returns:
            The first matching Tag, or None if not found.

        Raises:
            ValueError: If selector syntax is invalid.
        """
        ...

    def find_all(self, selector: str) -> list[Tag]:
        """Find all elements matching a CSS selector.

        Args:
            selector: CSS selector string.

        Returns:
            List of matching Tag instances.

        Raises:
            ValueError: If selector syntax is invalid.
        """
        ...

    def select(self, selector: str) -> list[Tag]:
        """Find all elements matching a CSS selector (alias for find_all)."""
        ...

    def to_html(self) -> str:
        """Get the HTML representation of the document."""
        ...

    def __len__(self) -> int:
        """Get the number of nodes in the document."""
        ...

    def __repr__(self) -> str:
        """Get string representation for debugging."""
        ...

def parse_batch(
    documents: list[str],
    n_threads: int | None = None,
) -> list[Soup]:
    """Parse multiple HTML documents in parallel.

    Uses Rayon for parallel processing with automatic thread pool management.

    Args:
        documents: List of HTML strings to parse.
        n_threads: Optional number of threads (defaults to CPU count).

    Returns:
        List of Soup instances in the same order as input.
    """
    ...
