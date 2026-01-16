/**
 * scrape-rs - High-performance HTML parsing library for browsers (WASM)
 *
 * @module @scrape-rs/wasm
 */

/** Configuration options for HTML parsing. */
export declare class SoupConfig {
	constructor();

	/** Maximum nesting depth for DOM tree. Default: 512 */
	maxDepth: number;

	/** Enable strict parsing mode. Default: false */
	strictMode: boolean;

	/** Preserve whitespace-only text nodes. Default: false */
	preserveWhitespace: boolean;

	/** Include comment nodes in DOM. Default: false */
	includeComments: boolean;
}

/** An HTML element in the DOM tree. */
export declare class Tag {
	/** Returns the tag name (e.g., 'div', 'span'). */
	readonly name: string | undefined;

	/** Returns the text content of this element and all descendants. */
	readonly text: string;

	/** Returns the inner HTML content (excluding this element's tags). */
	readonly innerHTML: string;

	/** Returns the outer HTML (including this element's tags). */
	readonly outerHTML: string;

	/**
	 * Get an attribute value by name.
	 * @param name - The attribute name
	 * @returns The attribute value, or undefined if not present
	 */
	get(name: string): string | undefined;

	/**
	 * Get an attribute value by name (alias for get).
	 * @param name - The attribute name
	 * @returns The attribute value, or undefined if not present
	 */
	attr(name: string): string | undefined;

	/**
	 * Check if the element has an attribute.
	 * @param name - The attribute name
	 * @returns True if the attribute exists
	 */
	hasAttr(name: string): boolean;

	/** Get all attributes as an object. */
	readonly attrs: Record<string, string>;

	/**
	 * Check if the element has a specific class.
	 * @param className - The class name to check
	 * @returns True if the element has the class
	 */
	hasClass(className: string): boolean;

	/** Get all classes as an array. */
	readonly classes: string[];

	/** Get the parent element. */
	readonly parent: Tag | undefined;

	/** Get all direct child elements. */
	readonly children: Tag[];

	/** Get the next sibling element. */
	readonly nextSibling: Tag | undefined;

	/** Get the previous sibling element. */
	readonly prevSibling: Tag | undefined;

	/** Get all descendant elements. */
	readonly descendants: Tag[];

	/**
	 * Find the first descendant matching a CSS selector.
	 * @param selector - CSS selector string
	 * @returns The first matching Tag, or undefined if not found
	 * @throws Error if the selector syntax is invalid
	 */
	find(selector: string): Tag | undefined;

	/**
	 * Find all descendants matching a CSS selector.
	 * @param selector - CSS selector string
	 * @returns Array of matching Tag instances
	 * @throws Error if the selector syntax is invalid
	 */
	findAll(selector: string): Tag[];

	/**
	 * Find all descendants matching a CSS selector (alias for findAll).
	 * @param selector - CSS selector string
	 * @returns Array of matching Tag instances
	 * @throws Error if the selector syntax is invalid
	 */
	select(selector: string): Tag[];

	/** Get the number of direct child elements. */
	readonly length: number;
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
	 * Find the first element matching a CSS selector.
	 * @param selector - CSS selector string
	 * @returns The first matching Tag, or undefined if not found
	 * @throws Error if the selector syntax is invalid
	 */
	find(selector: string): Tag | undefined;

	/**
	 * Find all elements matching a CSS selector.
	 * @param selector - CSS selector string
	 * @returns Array of matching Tag instances
	 * @throws Error if the selector syntax is invalid
	 */
	findAll(selector: string): Tag[];

	/**
	 * Find all elements matching a CSS selector (alias for findAll).
	 * @param selector - CSS selector string
	 * @returns Array of matching Tag instances
	 * @throws Error if the selector syntax is invalid
	 */
	select(selector: string): Tag[];

	/** Get the root element of the document. */
	readonly root: Tag | undefined;

	/** Get the document title. */
	readonly title: string | undefined;

	/** Get the text content of the entire document. */
	readonly text: string;

	/**
	 * Get the HTML representation of the document.
	 * @returns The document as an HTML string
	 */
	toHtml(): string;

	/** Get the number of nodes in the document. */
	readonly length: number;
}

/**
 * Parse multiple HTML documents.
 *
 * Note: WASM does not support threads, so this processes documents sequentially.
 * For parallel processing in browsers, use Web Workers with separate WASM instances.
 *
 * @param documents - Array of HTML strings to parse
 * @returns Array of Soup documents
 *
 * @example
 * ```javascript
 * const soups = parseBatch(['<div>A</div>', '<div>B</div>']);
 * console.log(soups.length); // 2
 * ```
 */
export declare function parseBatch(documents: string[]): Soup[];

/**
 * Check if WASM SIMD is supported in the current environment.
 *
 * Returns true if the module was compiled with SIMD support.
 * SIMD support requires:
 * - Chrome 91+ / Firefox 89+ / Safari 16.4+
 * - Module built with RUSTFLAGS='-C target-feature=+simd128'
 *
 * @returns true if SIMD is supported
 */
export declare function hasSimdSupport(): boolean;

/**
 * Get the library version.
 * @returns Version string (e.g., "0.1.0")
 */
export declare function version(): string;

/**
 * Initialize the WASM module.
 * Must be called before using any other functions.
 *
 * @example
 * ```javascript
 * import init, { Soup } from '@scrape-rs/wasm';
 *
 * await init();
 * const soup = new Soup('<div>Hello</div>');
 * ```
 */
export default function init(): Promise<void>;
