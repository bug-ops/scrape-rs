/**
 * scrape-rs - High-performance HTML parsing library for browsers (WASM)
 *
 * @module @scrape-rs/wasm
 */

/** Configuration options for HTML parsing. */
export declare class SoupConfig {
	constructor();

	/** Sets the maximum nesting depth. */
	setMaxDepth(depth: number): SoupConfig;

	/** Sets strict parsing mode. */
	setStrictMode(strict: boolean): SoupConfig;

	/** Returns the maximum depth setting. */
	readonly maxDepth: number;

	/** Returns the strict mode setting. */
	readonly strictMode: boolean;
}

/** An HTML element in the DOM tree. */
export declare class Tag {
	/** Returns the tag name (e.g., 'div', 'span'). */
	readonly name: string;

	/** Returns the text content of this element. */
	readonly text: string;

	/** Returns the inner HTML of this element. */
	readonly innerHTML: string;

	/** Returns the value of an attribute, if present. */
	get(attr: string): string | undefined;
}

/** A parsed HTML document. */
export declare class Soup {
	/**
	 * Parses an HTML string into a Soup document.
	 * @param html - The HTML string to parse
	 */
	constructor(html: string);

	/**
	 * Parses an HTML string with custom configuration.
	 * @param html - The HTML string to parse
	 * @param config - Configuration options
	 */
	static parseWithConfig(html: string, config: SoupConfig): Soup;

	/** Finds the first element matching the selector. */
	find(selector: string): Tag | undefined;

	/** Finds all elements matching the selector. */
	findAll(selector: string): Tag[];

	/** Selects elements using a CSS selector. */
	select(selector: string): Tag[];
}

/**
 * Parse multiple HTML documents.
 * Note: WASM does not support threads, so this processes documents sequentially.
 * @param documents - Array of HTML strings to parse
 * @returns Array of Soup documents
 */
export declare function parseBatch(documents: string[]): Soup[];

/**
 * Check if WASM SIMD is supported in the current environment.
 * @returns true if SIMD is supported
 */
export declare function hasSimdSupport(): boolean;

/**
 * Initialize the WASM module.
 * Must be called before using any other functions.
 */
export default function init(): Promise<void>;
