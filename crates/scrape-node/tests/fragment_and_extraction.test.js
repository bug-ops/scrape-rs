/**
 * Integration tests for Phase 13b features.
 *
 * Tests:
 * - Fragment parsing with various contexts
 * - CompiledSelector reuse and performance
 * - Extraction methods (selectText, selectAttr)
 * - Text nodes iterator
 * - Filtered iterators (childrenByName, childrenByClass)
 */

import assert from "node:assert";
import { describe, it } from "node:test";
import { CompiledSelector, Soup } from "../index.js";

// ==================== Fragment Parsing Tests ====================

describe("Fragment Parsing", () => {
	it("should parse empty fragment", () => {
		const soup = Soup.parseFragment("");
		assert.strictEqual(soup.length, 0);
	});

	it("should parse text-only fragment", () => {
		const soup = Soup.parseFragment("Hello World");
		assert.ok(soup.text.includes("Hello World"));
	});

	it("should parse simple fragment", () => {
		const soup = Soup.parseFragment("<div>Test</div>");
		const div = soup.find("div");
		assert.ok(div !== null);
		assert.strictEqual(div?.text, "Test");
	});

	it("should parse nested fragment", () => {
		const html = "<div><span>A</span><span>B</span></div>";
		const soup = Soup.parseFragment(html);
		const spans = soup.findAll("span");
		assert.strictEqual(spans.length, 2);
		assert.strictEqual(spans[0].text, "A");
		assert.strictEqual(spans[1].text, "B");
	});

	it("should not add wrapper tags", () => {
		const soup = Soup.parseFragment("<div>Content</div>");
		const html = soup.toHtml();
		assert.ok(html.includes("<div>"));
		assert.ok(soup.root !== null);
	});

	it("should parse with body context", () => {
		const soup = Soup.parseFragment("<div>Test</div>", "body");
		const div = soup.find("div");
		assert.ok(div !== null);
		assert.strictEqual(div?.text, "Test");
	});

	it("should parse with table context", () => {
		const html = "<tr><td>Cell 1</td><td>Cell 2</td></tr>";
		const soup = Soup.parseFragment(html, "table");
		const tds = soup.findAll("td");
		assert.strictEqual(tds.length, 2);
		assert.strictEqual(tds[0].text, "Cell 1");
		assert.strictEqual(tds[1].text, "Cell 2");
	});

	it("should parse with tbody context", () => {
		const html = "<tr><td>A</td></tr><tr><td>B</td></tr>";
		const soup = Soup.parseFragment(html, "tbody");
		const trs = soup.findAll("tr");
		assert.strictEqual(trs.length, 2);
	});

	it("should handle multiple root elements", () => {
		const html = "<div>First</div><div>Second</div><div>Third</div>";
		const soup = Soup.parseFragment(html);
		const divs = soup.findAll("div");
		assert.strictEqual(divs.length, 3);
	});

	it("should preserve attributes in fragments", () => {
		const html = '<span class="highlight" id="item">Text</span>';
		const soup = Soup.parseFragment(html);
		const span = soup.find("span");
		assert.ok(span !== null);
		assert.strictEqual(span?.get("class"), "highlight");
		assert.strictEqual(span?.get("id"), "item");
	});
});

// ==================== CompiledSelector Tests ====================

describe("CompiledSelector", () => {
	it("should compile simple selector", () => {
		const selector = CompiledSelector.compile("div.item");
		assert.ok(selector !== null);
		assert.strictEqual(selector.source, "div.item");
	});

	it("should compile complex selector", () => {
		const selector = CompiledSelector.compile("ul > li.active:first-child");
		assert.strictEqual(selector.source, "ul > li.active:first-child");
	});

	it("should throw on invalid selector", () => {
		assert.throws(() => {
			CompiledSelector.compile("[[[invalid");
		}, /selector/i);
	});

	it("should work with findCompiled", () => {
		const html = "<div class='item'>First</div><div class='item'>Second</div>";
		const soup = new Soup(html);
		const selector = CompiledSelector.compile(".item");

		const result = soup.findCompiled(selector);
		assert.ok(result !== null);
		assert.strictEqual(result?.text, "First");
	});

	it("should work with selectCompiled", () => {
		const html = "<ul><li class='active'>A</li><li>B</li><li class='active'>C</li></ul>";
		const soup = new Soup(html);
		const selector = CompiledSelector.compile("li.active");

		const results = soup.selectCompiled(selector);
		assert.strictEqual(results.length, 2);
		assert.strictEqual(results[0].text, "A");
		assert.strictEqual(results[1].text, "C");
	});

	it("should allow selector reuse", () => {
		const selector = CompiledSelector.compile("span.highlight");

		const soup1 = new Soup("<div><span class='highlight'>Doc 1</span></div>");
		const soup2 = new Soup("<div><span class='highlight'>Doc 2</span></div>");

		const result1 = soup1.findCompiled(selector);
		const result2 = soup2.findCompiled(selector);

		assert.ok(result1 !== null);
		assert.ok(result2 !== null);
		assert.strictEqual(result1?.text, "Doc 1");
		assert.strictEqual(result2?.text, "Doc 2");
	});

	it("should return null when not found", () => {
		const soup = new Soup("<div>No match</div>");
		const selector = CompiledSelector.compile(".nonexistent");

		const result = soup.findCompiled(selector);
		assert.strictEqual(result, null);
	});

	it("should return empty array when no matches", () => {
		const soup = new Soup("<div>No match</div>");
		const selector = CompiledSelector.compile("li");

		const results = soup.selectCompiled(selector);
		assert.strictEqual(results.length, 0);
	});
});

// ==================== Extraction Methods Tests ====================

describe("Extraction Methods", () => {
	it("should extract single text", () => {
		const html = "<div><span class='item'>Hello</span></div>";
		const soup = new Soup(html);

		const texts = soup.selectText(".item");
		assert.strictEqual(texts.length, 1);
		assert.strictEqual(texts[0], "Hello");
	});

	it("should extract multiple texts", () => {
		const html = `
			<ul>
				<li class='item'>First</li>
				<li class='item'>Second</li>
				<li class='item'>Third</li>
			</ul>
		`;
		const soup = new Soup(html);

		const texts = soup.selectText(".item");
		assert.strictEqual(texts.length, 3);
		assert.strictEqual(texts[0], "First");
		assert.strictEqual(texts[1], "Second");
		assert.strictEqual(texts[2], "Third");
	});

	it("should return empty array when no text matches", () => {
		const soup = new Soup("<div>No matches</div>");
		const texts = soup.selectText(".nonexistent");
		assert.strictEqual(texts.length, 0);
	});

	it("should extract nested content", () => {
		const html = "<div class='item'>Hello <b>World</b>!</div>";
		const soup = new Soup(html);

		const texts = soup.selectText(".item");
		assert.strictEqual(texts.length, 1);
		assert.ok(texts[0].includes("Hello"));
		assert.ok(texts[0].includes("World"));
	});

	it("should throw on invalid selector in selectText", () => {
		const soup = new Soup("<div>Test</div>");
		assert.throws(() => {
			soup.selectText("[[[");
		}, /selector/i);
	});

	it("should extract single attribute", () => {
		const html = "<a href='/link' class='link'>Click</a>";
		const soup = new Soup(html);

		const hrefs = soup.selectAttr("a", "href");
		assert.strictEqual(hrefs.length, 1);
		assert.strictEqual(hrefs[0], "/link");
	});

	it("should extract multiple attributes", () => {
		const html = `
			<div>
				<a href='/page1'>Link 1</a>
				<a href='/page2'>Link 2</a>
				<a href='/page3'>Link 3</a>
			</div>
		`;
		const soup = new Soup(html);

		const hrefs = soup.selectAttr("a", "href");
		assert.strictEqual(hrefs.length, 3);
		assert.strictEqual(hrefs[0], "/page1");
		assert.strictEqual(hrefs[1], "/page2");
		assert.strictEqual(hrefs[2], "/page3");
	});

	it("should skip elements without attribute", () => {
		const html = '<div><a href="/link">Has</a><a>Missing</a></div>';
		const soup = new Soup(html);

		const hrefs = soup.selectAttr("a", "href");
		// Returns Option array - null for missing attributes
		assert.strictEqual(hrefs.length, 2);
		assert.strictEqual(hrefs[0], "/link");
		assert.strictEqual(hrefs[1], null);
	});

	it("should return empty array when no attributes", () => {
		const soup = new Soup("<div>No links</div>");
		const hrefs = soup.selectAttr("a", "href");
		assert.strictEqual(hrefs.length, 0);
	});

	it("should throw on invalid selector in selectAttr", () => {
		const soup = new Soup("<div>Test</div>");
		assert.throws(() => {
			soup.selectAttr("[[[", "id");
		}, /selector/i);
	});

	it("should extract class attributes", () => {
		const html = `
			<div>
				<span class='tag-a'>A</span>
				<span class='tag-b'>B</span>
				<span class='tag-c'>C</span>
			</div>
		`;
		const soup = new Soup(html);

		const classes = soup.selectAttr("span", "class");
		assert.strictEqual(classes.length, 3);
		assert.ok(classes.includes("tag-a"));
		assert.ok(classes.includes("tag-b"));
		assert.ok(classes.includes("tag-c"));
	});
});

// ==================== Text Nodes Iterator Tests ====================

describe("Text Nodes", () => {
	it("should get simple text node", () => {
		const html = "<div>Hello World</div>";
		const soup = new Soup(html);
		const div = soup.find("div");

		const textNodes = div?.textNodes || [];
		assert.strictEqual(textNodes.length, 1);
		assert.strictEqual(textNodes[0], "Hello World");
	});

	it("should get multiple text nodes", () => {
		const html = "<div>First<span>Middle</span>Last</div>";
		const soup = new Soup(html);
		const div = soup.find("div");

		const textNodes = div?.textNodes || [];
		const hasFirst = textNodes.some((t) => t.includes("First"));
		const hasLast = textNodes.some((t) => t.includes("Last"));
		const hasMiddle = textNodes.some((t) => t.includes("Middle"));

		assert.ok(hasFirst);
		assert.ok(hasLast);
		assert.ok(!hasMiddle); // Middle is inside span
	});

	it("should return empty for no direct text", () => {
		const html = "<div><span>No direct text</span></div>";
		const soup = new Soup(html);
		const div = soup.find("div");

		const textNodes = div?.textNodes || [];
		const nonEmptyNodes = textNodes.filter((t) => t.trim() !== "");
		assert.strictEqual(nonEmptyNodes.length, 0);
	});

	it("should preserve whitespace", () => {
		const html = "<div>  Text with spaces  </div>";
		const soup = new Soup(html);
		const div = soup.find("div");

		const textNodes = div?.textNodes || [];
		assert.ok(textNodes.length >= 1);
		assert.ok(textNodes.some((node) => node.includes("Text with spaces")));
	});

	it("should handle mixed content", () => {
		const html = "<p>Start <b>bold</b> middle <i>italic</i> end</p>";
		const soup = new Soup(html);
		const p = soup.find("p");

		const textNodes = p?.textNodes || [];
		assert.ok(textNodes.some((node) => node.includes("Start")));
		assert.ok(textNodes.some((node) => node.includes("middle")));
		assert.ok(textNodes.some((node) => node.includes("end")));
	});
});

// ==================== Filtered Iterators Tests ====================

describe("Filtered Iterators", () => {
	it("should filter children by name", () => {
		const html = "<div><span>A</span><p>B</p><span>C</span></div>";
		const soup = new Soup(html);
		const div = soup.find("div");

		const spans = div?.childrenByName("span") || [];
		assert.strictEqual(spans.length, 2);
		assert.strictEqual(spans[0].text, "A");
		assert.strictEqual(spans[1].text, "C");
	});

	it("should return all when all match", () => {
		const html = "<ul><li>A</li><li>B</li><li>C</li></ul>";
		const soup = new Soup(html);
		const ul = soup.find("ul");

		const items = ul?.childrenByName("li") || [];
		assert.strictEqual(items.length, 3);
	});

	it("should return empty when none match by name", () => {
		const html = "<div><span>A</span><span>B</span></div>";
		const soup = new Soup(html);
		const div = soup.find("div");

		const items = div?.childrenByName("li") || [];
		assert.strictEqual(items.length, 0);
	});

	it("should return empty for no children", () => {
		const html = "<div></div>";
		const soup = new Soup(html);
		const div = soup.find("div");

		const items = div?.childrenByName("span") || [];
		assert.strictEqual(items.length, 0);
	});

	it("should be case sensitive for tag names", () => {
		const html = "<div><SPAN>A</SPAN><span>B</span></div>";
		const soup = new Soup(html);
		const div = soup.find("div");

		const items = div?.childrenByName("span") || [];
		assert.strictEqual(items.length, 2);
	});

	it("should filter children by class", () => {
		const html = "<div><span class='item'>A</span><span>B</span></div>";
		const soup = new Soup(html);
		const div = soup.find("div");

		const items = div?.childrenByClass("item") || [];
		assert.strictEqual(items.length, 1);
		assert.strictEqual(items[0].text, "A");
	});

	it("should filter multiple elements by class", () => {
		const html = `
			<div>
				<span class='tag'>A</span>
				<p class='tag'>B</p>
				<div class='tag'>C</div>
			</div>
		`;
		const soup = new Soup(html);
		const div = soup.find("div");

		const items = div?.childrenByClass("tag") || [];
		assert.strictEqual(items.length, 3);
	});

	it("should match elements with multiple classes", () => {
		const html = "<div><span class='item active'>A</span><span class='item'>B</span></div>";
		const soup = new Soup(html);
		const div = soup.find("div");

		const items = div?.childrenByClass("item") || [];
		assert.strictEqual(items.length, 2);

		const activeItems = div?.childrenByClass("active") || [];
		assert.strictEqual(activeItems.length, 1);
		assert.strictEqual(activeItems[0].text, "A");
	});

	it("should return empty when no class matches", () => {
		const html = "<div><span class='a'>A</span><span class='b'>B</span></div>";
		const soup = new Soup(html);
		const div = soup.find("div");

		const items = div?.childrenByClass("c") || [];
		assert.strictEqual(items.length, 0);
	});

	it("should return empty for elements without class", () => {
		const html = "<div><span>A</span><span>B</span></div>";
		const soup = new Soup(html);
		const div = soup.find("div");

		const items = div?.childrenByClass("item") || [];
		assert.strictEqual(items.length, 0);
	});
});

// ==================== Scoped Extraction Methods Tests ====================

describe("Scoped Extraction Methods", () => {
	it("should extract text within scope", () => {
		const html = `
			<div class='container'>
				<span class='item'>Inside</span>
			</div>
			<span class='item'>Outside</span>
		`;
		const soup = new Soup(html);
		const container = soup.find(".container");

		const texts = container?.selectText(".item") || [];
		assert.strictEqual(texts.length, 1);
		assert.strictEqual(texts[0], "Inside");
	});

	it("should extract attributes within scope", () => {
		const html = `
			<div class='container'>
				<a href='/inside'>Inside</a>
			</div>
			<a href='/outside'>Outside</a>
		`;
		const soup = new Soup(html);
		const container = soup.find(".container");

		const hrefs = container?.selectAttr("a", "href") || [];
		assert.strictEqual(hrefs.length, 1);
		assert.strictEqual(hrefs[0], "/inside");
	});

	it("should use compiled selector in tag scope", () => {
		const html = "<div><span class='target'>Found</span></div>";
		const soup = new Soup(html);
		const div = soup.find("div");

		const selector = CompiledSelector.compile(".target");
		const result = div?.findCompiled(selector);

		assert.ok(result !== null);
		assert.strictEqual(result?.text, "Found");
	});

	it("should use selectCompiled in tag scope", () => {
		const html = "<div><span class='item'>A</span><span class='item'>B</span></div>";
		const soup = new Soup(html);
		const div = soup.find("div");

		const selector = CompiledSelector.compile(".item");
		const results = div?.selectCompiled(selector) || [];

		assert.strictEqual(results.length, 2);
		assert.strictEqual(results[0].text, "A");
		assert.strictEqual(results[1].text, "B");
	});
});

// ==================== Edge Cases and Integration Tests ====================

describe("Edge Cases", () => {
	it("should handle fragments with scripts", () => {
		const html = "<div>Before</div><script>alert('test');</script><div>After</div>";
		const soup = Soup.parseFragment(html);
		const divs = soup.findAll("div");
		assert.strictEqual(divs.length, 2);
	});

	it("should work with pseudo-classes", () => {
		const html = "<ul><li>First</li><li>Second</li><li>Third</li></ul>";
		const soup = new Soup(html);

		const selector = CompiledSelector.compile("li:first-child");
		const result = soup.findCompiled(selector);

		assert.ok(result !== null);
		assert.strictEqual(result?.text, "First");
	});

	it("should handle empty elements in selectText", () => {
		const html = "<div class='item'></div><div class='item'>Text</div>";
		const soup = new Soup(html);

		const texts = soup.selectText(".item");
		assert.strictEqual(texts.length, 2);
		assert.strictEqual(texts[0], "");
		assert.strictEqual(texts[1], "Text");
	});

	it("should extract data attributes", () => {
		const html = `
			<div>
				<button data-id='1'>A</button>
				<button data-id='2'>B</button>
				<button data-id='3'>C</button>
			</div>
		`;
		const soup = new Soup(html);

		const ids = soup.selectAttr("button", "data-id");
		assert.strictEqual(ids.length, 3);
		assert.deepStrictEqual(ids, ["1", "2", "3"]);
	});

	it("should handle entities in text nodes", () => {
		const html = "<div>Hello &amp; goodbye &lt;test&gt;</div>";
		const soup = new Soup(html);
		const div = soup.find("div");

		const textNodes = div?.textNodes || [];
		assert.ok(textNodes.length >= 1);
		const text = textNodes.join("");
		assert.ok(text.includes("&") || text.includes("&amp;"));
	});
});
