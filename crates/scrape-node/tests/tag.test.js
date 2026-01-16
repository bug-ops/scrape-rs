const { describe, it, before } = require("node:test");
const assert = require("node:assert");

let Soup;

describe("Tag", () => {
	before(async () => {
		const mod = await import("../index.js");
		Soup = mod.Soup;
	});

	describe("content", () => {
		it("should get tag name", () => {
			const soup = new Soup("<div>text</div>");
			const tag = soup.find("div");
			assert.strictEqual(tag.name, "div");
		});

		it("should get text content", () => {
			const soup = new Soup("<div>Hello <span>World</span></div>");
			const tag = soup.find("div");
			assert.ok(tag.text.includes("Hello"));
			assert.ok(tag.text.includes("World"));
		});

		it("should get innerHTML", () => {
			const soup = new Soup("<div><span>Hello</span></div>");
			const tag = soup.find("div");
			const html = tag.innerHTML;
			assert.ok(html.includes("<span>"));
			assert.ok(!html.includes("<div>"));
		});

		it("should get outerHTML", () => {
			const soup = new Soup("<div><span>Hello</span></div>");
			const tag = soup.find("div");
			const html = tag.outerHTML;
			assert.ok(html.includes("<div>"));
			assert.ok(html.includes("</div>"));
		});

		it("should escape HTML entities in text", () => {
			const soup = new Soup("<div>&lt;script&gt;</div>");
			const tag = soup.find("div");
			assert.ok(tag.text.includes("<script>"));
		});
	});

	describe("attributes", () => {
		it("should get attribute with get()", () => {
			const soup = new Soup('<a href="/page">Link</a>');
			const tag = soup.find("a");
			assert.strictEqual(tag.get("href"), "/page");
		});

		it("should get attribute with attr()", () => {
			const soup = new Soup('<a href="/page">Link</a>');
			const tag = soup.find("a");
			assert.strictEqual(tag.attr("href"), "/page");
		});

		it("should return null for missing attribute", () => {
			const soup = new Soup('<a href="/page">Link</a>');
			const tag = soup.find("a");
			assert.strictEqual(tag.get("nonexistent"), null);
		});

		it("should check hasAttr", () => {
			const soup = new Soup('<input disabled type="text">');
			const tag = soup.find("input");
			assert.strictEqual(tag.hasAttr("disabled"), true);
			assert.strictEqual(tag.hasAttr("type"), true);
			assert.strictEqual(tag.hasAttr("value"), false);
		});

		it("should get all attrs", () => {
			const soup = new Soup('<div id="main" class="container">text</div>');
			const tag = soup.find("div");
			const attrs = tag.attrs;
			assert.strictEqual(attrs.id, "main");
			assert.strictEqual(attrs.class, "container");
		});

		it("should check hasClass", () => {
			const soup = new Soup('<div class="foo bar">text</div>');
			const tag = soup.find("div");
			assert.strictEqual(tag.hasClass("foo"), true);
			assert.strictEqual(tag.hasClass("bar"), true);
			assert.strictEqual(tag.hasClass("baz"), false);
		});

		it("should get classes array", () => {
			const soup = new Soup('<div class="foo bar baz">text</div>');
			const tag = soup.find("div");
			const classes = tag.classes;
			assert.ok(classes.includes("foo"));
			assert.ok(classes.includes("bar"));
			assert.ok(classes.includes("baz"));
		});

		it("should return empty classes for elements without class", () => {
			const soup = new Soup("<div>text</div>");
			const tag = soup.find("div");
			assert.deepStrictEqual(tag.classes, []);
		});

		it("should return empty attrs for elements without attributes", () => {
			const soup = new Soup("<div>text</div>");
			const tag = soup.find("div");
			assert.deepStrictEqual(tag.attrs, {});
		});
	});

	describe("length", () => {
		it("should return child element count", () => {
			const soup = new Soup("<ul><li>A</li><li>B</li><li>C</li></ul>");
			const ul = soup.find("ul");
			assert.strictEqual(ul.length, 3);
		});

		it("should return 0 for leaf elements", () => {
			const soup = new Soup("<span>text</span>");
			const span = soup.find("span");
			assert.strictEqual(span.length, 0);
		});
	});
});
