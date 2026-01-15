/**
 * scrape-rs - High-performance HTML parsing library for Node.js
 *
 * @module scrape-rs
 */

/** Configuration options for HTML parsing. */
export interface SoupConfig {
	/** Maximum nesting depth for DOM tree. Default: 256 */
	maxDepth?: number;
	/** Enable strict parsing mode. Default: false */
	strictMode?: boolean;
}

/** An HTML element in the DOM tree. */
export declare class Tag {
	/** Returns the tag name (e.g., 'div', 'span'). */
	readonly name: string;

	/** Returns the text content of this element. */
	readonly text: string;

	/** Returns the inner HTML of this element. */
	readonly innerHTML: string;

	/** Returns the parent element, if any. */
	readonly parent: Tag | null;

	/** Returns an iterator over direct child elements. */
	readonly children: Tag[];

	/** Returns the next sibling element. */
	readonly nextSibling: Tag | null;

	/** Returns the previous sibling element. */
	readonly prevSibling: Tag | null;

	/** Returns the value of an attribute, if present. */
	get(attr: string): string | null;

	/** Alias for get(). */
	attr(attr: string): string | null;

	/** Checks if this element has the specified class. */
	hasClass(className: string): boolean;

	/** Finds the first descendant matching the selector. */
	find(selector: string): Tag | null;

	/** Finds all descendants matching the selector. */
	findAll(selector: string): Tag[];

	/** Selects descendants using a CSS selector. */
	select(selector: string): Tag[];
}

/** A parsed HTML document. */
export declare class Soup {
	/**
	 * Parses an HTML string into a Soup document.
	 * @param html - The HTML string to parse
	 * @param config - Optional configuration options
	 */
	constructor(html: string, config?: SoupConfig);

	/**
	 * Parses HTML from a file.
	 * @param path - Path to the HTML file
	 */
	static fromFile(path: string): Promise<Soup>;

	/**
	 * Fetches and parses HTML from a URL.
	 * @param url - The URL to fetch
	 */
	static fromUrl(url: string): Promise<Soup>;

	/** Finds the first element matching the selector. */
	find(selector: string): Tag | null;

	/** Finds all elements matching the selector. */
	findAll(selector: string): Tag[];

	/** Selects elements using a CSS selector. */
	select(selector: string): Tag[];

	/** Returns the document's title, if present. */
	readonly title: string | null;

	/** Returns the document's text content with tags stripped. */
	readonly text: string;
}

/**
 * Parse multiple HTML documents in parallel.
 * @param documents - Array of HTML strings to parse
 * @param config - Optional configuration options
 * @returns Array of Soup documents
 */
export declare function parseBatch(
	documents: string[],
	config?: SoupConfig,
): Soup[];
