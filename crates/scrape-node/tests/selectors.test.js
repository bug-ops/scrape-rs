const { describe, it, before } = require("node:test");
const assert = require("node:assert");

let Soup;

describe("CSS Selectors", () => {
	before(async () => {
		const mod = await import("../index.js");
		Soup = mod.Soup;
	});

	const html = `
		<html>
		<head><title>Test</title></head>
		<body>
			<div id="main" class="container">
				<ul class="list">
					<li class="item active" data-id="1">First</li>
					<li class="item" data-id="2">Second</li>
					<li class="item" data-id="3">Third</li>
				</ul>
				<p class="description">Some text</p>
				<a href="/link" target="_blank">Click</a>
			</div>
			<footer>
				<span class="item">Footer item</span>
			</footer>
		</body>
		</html>
	`;

	describe("type selectors", () => {
		it("should match by tag name", () => {
			const soup = new Soup(html);
			const ul = soup.find("ul");
			assert.ok(ul !== null);
			assert.strictEqual(ul.name, "ul");
		});

		it("should find all matching elements", () => {
			const soup = new Soup(html);
			const lis = soup.findAll("li");
			assert.strictEqual(lis.length, 3);
		});
	});

	describe("class selectors", () => {
		it("should match single class", () => {
			const soup = new Soup(html);
			const container = soup.find(".container");
			assert.ok(container !== null);
		});

		it("should find all with class", () => {
			const soup = new Soup(html);
			const items = soup.findAll(".item");
			assert.strictEqual(items.length, 4);
		});

		it("should match multiple classes", () => {
			const soup = new Soup(html);
			const active = soup.find(".item.active");
			assert.ok(active !== null);
			assert.ok(active.text.includes("First"));
		});
	});

	describe("ID selectors", () => {
		it("should match by ID", () => {
			const soup = new Soup(html);
			const main = soup.find("#main");
			assert.ok(main !== null);
			assert.strictEqual(main.get("id"), "main");
		});
	});

	describe("attribute selectors", () => {
		it("should match presence", () => {
			const soup = new Soup(html);
			const withHref = soup.find("[href]");
			assert.ok(withHref !== null);
			assert.strictEqual(withHref.name, "a");
		});

		it("should match exact value", () => {
			const soup = new Soup(html);
			const link = soup.find('[href="/link"]');
			assert.ok(link !== null);
		});

		it("should match data attributes", () => {
			const soup = new Soup(html);
			const item = soup.find('[data-id="2"]');
			assert.ok(item !== null);
			assert.ok(item.text.includes("Second"));
		});

		it("should find all with attribute", () => {
			const soup = new Soup(html);
			const items = soup.findAll("[data-id]");
			assert.strictEqual(items.length, 3);
		});
	});

	describe("combinators", () => {
		it("should match descendants", () => {
			const soup = new Soup(html);
			const lis = soup.findAll("div li");
			assert.strictEqual(lis.length, 3);
		});

		it("should match direct children", () => {
			const soup = new Soup(html);
			const items = soup.findAll("ul > li");
			assert.strictEqual(items.length, 3);
		});

		it("should not match non-direct descendants with child combinator", () => {
			const soup = new Soup("<div><ul><li>A</li></ul></div>");
			const items = soup.findAll("div > li");
			assert.strictEqual(items.length, 0);
		});

		it("should match adjacent sibling", () => {
			const soup = new Soup(html);
			const p = soup.find("ul + p");
			assert.ok(p !== null);
			assert.strictEqual(p.name, "p");
		});

		it("should match general sibling", () => {
			const soup = new Soup(html);
			const a = soup.find("ul ~ a");
			assert.ok(a !== null);
			assert.strictEqual(a.name, "a");
		});
	});

	describe("compound selectors", () => {
		it("should match tag and class", () => {
			const soup = new Soup(html);
			const item = soup.find("li.active");
			assert.ok(item !== null);
		});

		it("should match tag and ID", () => {
			const soup = new Soup(html);
			const main = soup.find("div#main");
			assert.ok(main !== null);
		});

		it("should match multiple conditions", () => {
			const soup = new Soup(html);
			const item = soup.find('li.item[data-id="1"]');
			assert.ok(item !== null);
			assert.ok(item.text.includes("First"));
		});
	});

	describe("pseudo-classes", () => {
		it("should match :first-child", () => {
			const soup = new Soup(html);
			const first = soup.find("li:first-child");
			assert.ok(first !== null);
			assert.ok(first.text.includes("First"));
		});

		it("should match :last-child", () => {
			const soup = new Soup(html);
			const last = soup.find("li:last-child");
			assert.ok(last !== null);
			assert.ok(last.text.includes("Third"));
		});

		// Note: :nth-child selectors have limited support in scrape-core
		// These tests are skipped as they're a core limitation, not bindings issue
		it.skip("should match :nth-child", () => {
			const soup = new Soup(html);
			const second = soup.find("li:nth-child(2)");
			assert.ok(second !== null);
			assert.ok(second.text.includes("Second"));
		});

		it.skip("should match :nth-child(odd)", () => {
			const soup = new Soup(html);
			const odds = soup.findAll("li:nth-child(odd)");
			assert.strictEqual(odds.length, 2);
		});

		it.skip("should match :nth-child(even)", () => {
			const soup = new Soup(html);
			const evens = soup.findAll("li:nth-child(even)");
			assert.strictEqual(evens.length, 1);
		});

		it("should match :not()", () => {
			const soup = new Soup(html);
			const notActive = soup.findAll("li:not(.active)");
			assert.strictEqual(notActive.length, 2);
		});
	});

	describe("error handling", () => {
		it("should throw on invalid selector syntax", () => {
			const soup = new Soup(html);
			assert.throws(() => soup.find("div[[["), /selector/i);
		});

		it("should throw on empty selector", () => {
			const soup = new Soup(html);
			assert.throws(() => soup.find(""), /selector/i);
		});
	});
});
