const { describe, it, before } = require("node:test");
const assert = require("node:assert");

let Soup;

describe("Navigation", () => {
	before(async () => {
		const mod = await import("../index.js");
		Soup = mod.Soup;
	});

	const navHtml = `
		<html>
		<body>
			<div id="container">
				<span id="first">A</span>
				<span id="second">B</span>
				<span id="third">C</span>
			</div>
			<div id="footer">Footer</div>
		</body>
		</html>
	`;

	describe("parent", () => {
		it("should get parent element", () => {
			const soup = new Soup(navHtml);
			const span = soup.find("#first");
			const parent = span.parent;
			assert.strictEqual(parent.name, "div");
			assert.strictEqual(parent.get("id"), "container");
		});

		it("should traverse parent chain", () => {
			const soup = new Soup(navHtml);
			const span = soup.find("#first");
			const div = span.parent;
			const body = div.parent;
			const html = body.parent;
			assert.strictEqual(html.name, "html");
		});

		it("should return null for root parent", () => {
			const soup = new Soup(navHtml);
			const root = soup.root;
			assert.strictEqual(root.parent, null);
		});
	});

	describe("children", () => {
		it("should get child elements", () => {
			const soup = new Soup(navHtml);
			const container = soup.find("#container");
			const children = container.children;
			assert.strictEqual(children.length, 3);
			assert.ok(children.every((c) => c.name === "span"));
		});

		it("should return empty array for leaf elements", () => {
			const soup = new Soup(navHtml);
			const span = soup.find("#first");
			assert.strictEqual(span.children.length, 0);
		});

		it("should filter out text nodes", () => {
			const soup = new Soup("<div>text<span>A</span>more text<span>B</span></div>");
			const div = soup.find("div");
			const children = div.children;
			assert.strictEqual(children.length, 2);
			assert.ok(children.every((c) => c.name === "span"));
		});
	});

	describe("siblings", () => {
		it("should get next sibling", () => {
			const soup = new Soup(navHtml);
			const first = soup.find("#first");
			const second = first.nextSibling;
			assert.strictEqual(second.get("id"), "second");
		});

		it("should get previous sibling", () => {
			const soup = new Soup(navHtml);
			const second = soup.find("#second");
			const first = second.prevSibling;
			assert.strictEqual(first.get("id"), "first");
		});

		it("should return null for last element next sibling", () => {
			const soup = new Soup(navHtml);
			const third = soup.find("#third");
			assert.strictEqual(third.nextSibling, null);
		});

		it("should return null for first element prev sibling", () => {
			const soup = new Soup(navHtml);
			const first = soup.find("#first");
			assert.strictEqual(first.prevSibling, null);
		});

		it("should skip text nodes when getting siblings", () => {
			const soup = new Soup("<div><span>A</span> text <span>B</span></div>");
			const firstSpan = soup.find("span");
			const secondSpan = firstSpan.nextSibling;
			assert.strictEqual(secondSpan.name, "span");
			assert.strictEqual(secondSpan.text, "B");
		});
	});

	describe("descendants", () => {
		it("should get all descendants", () => {
			const soup = new Soup(navHtml);
			const container = soup.find("#container");
			const descendants = container.descendants;
			assert.strictEqual(descendants.length, 3);
		});

		it("should include deep descendants", () => {
			const soup = new Soup(navHtml);
			const body = soup.find("body");
			const descendants = body.descendants;
			assert.ok(descendants.length >= 5);
		});

		it("should return empty array for leaf elements", () => {
			const soup = new Soup("<span>text</span>");
			const span = soup.find("span");
			assert.strictEqual(span.descendants.length, 0);
		});

		it("should include nested descendants", () => {
			const soup = new Soup("<div><ul><li><a>link</a></li></ul></div>");
			const div = soup.find("div");
			const descendants = div.descendants;
			const names = descendants.map((d) => d.name);
			assert.ok(names.includes("ul"));
			assert.ok(names.includes("li"));
			assert.ok(names.includes("a"));
		});
	});

	describe("scoped queries", () => {
		it("should find within element", () => {
			const soup = new Soup(navHtml);
			const container = soup.find("#container");
			const span = container.find("span");
			assert.strictEqual(span.get("id"), "first");
		});

		it("should findAll within element", () => {
			const soup = new Soup(navHtml);
			const container = soup.find("#container");
			const spans = container.findAll("span");
			assert.strictEqual(spans.length, 3);
		});

		it("should not find elements outside scope", () => {
			const soup = new Soup(navHtml);
			const container = soup.find("#container");
			const footer = container.find("#footer");
			assert.strictEqual(footer, null);
		});

		it("should select within element (alias)", () => {
			const soup = new Soup(navHtml);
			const container = soup.find("#container");
			const spans = container.select("span");
			assert.strictEqual(spans.length, 3);
		});

		it("should find with complex selector within scope", () => {
			const soup = new Soup(
				'<div id="scope"><ul><li class="a">A</li><li class="b">B</li></ul></div>',
			);
			const scope = soup.find("#scope");
			const item = scope.find("li.b");
			assert.strictEqual(item.text, "B");
		});
	});
});
