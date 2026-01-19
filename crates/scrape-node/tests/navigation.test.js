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

	describe("parents and ancestors (Phase 12)", () => {
		it("parents getter should return ancestor array", () => {
			const html = "<html><body><div><span><a>link</a></span></div></body></html>";
			const soup = new Soup(html);
			const link = soup.find("a");

			const parents = link.parents;
			assert.strictEqual(parents.length, 4); // span, div, body, html
			assert.strictEqual(parents[0].name, "span");
			assert.strictEqual(parents[1].name, "div");
			assert.strictEqual(parents[2].name, "body");
			assert.strictEqual(parents[3].name, "html");
		});

		it("ancestors getter should be alias for parents", () => {
			const html = "<html><body><div><span>text</span></div></body></html>";
			const soup = new Soup(html);
			const span = soup.find("span");

			const parents = span.parents;
			const ancestors = span.ancestors;

			assert.strictEqual(parents.length, ancestors.length);
			for (let i = 0; i < parents.length; i++) {
				assert.strictEqual(parents[i].name, ancestors[i].name);
			}
		});

		it("parents should return empty array for root", () => {
			const html = "<html><body><div>text</div></body></html>";
			const soup = new Soup(html);
			const root = soup.root;

			assert.strictEqual(root.parents.length, 0);
		});

		it("parents should return partial chain", () => {
			const html = '<div id="outer"><div id="middle"><div id="inner">text</div></div></div>';
			const soup = new Soup(html);
			const inner = soup.find("#inner");

			const parents = inner.parents;
			assert.strictEqual(parents.length, 4); // middle, outer, body, html
			assert.strictEqual(parents[0].get("id"), "middle");
			assert.strictEqual(parents[1].get("id"), "outer");
			assert.strictEqual(parents[2].name, "body");
			assert.strictEqual(parents[3].name, "html");
		});
	});

	describe("closest (Phase 12)", () => {
		it("closest should find matching ancestor", () => {
			const html = '<div class="outer"><div class="middle"><span>text</span></div></div>';
			const soup = new Soup(html);
			const span = soup.find("span");

			const result = span.closest(".outer");
			assert.notStrictEqual(result, null);
			assert.strictEqual(result.get("class"), "outer");
		});

		it("closest should find nearest match", () => {
			const html = '<div class="target"><div class="target"><span>text</span></div></div>';
			const soup = new Soup(html);
			const span = soup.find("span");

			const result = span.closest(".target");
			assert.notStrictEqual(result, null);
			// Should be the inner div (nearest)
			const parent = span.parent;
			assert.strictEqual(result.outerHtml, parent.outerHtml);
		});

		it("closest should return null when not found", () => {
			const html = "<div><span>text</span></div>";
			const soup = new Soup(html);
			const span = soup.find("span");

			const result = span.closest(".nonexistent");
			assert.strictEqual(result, null);
		});

		it("closest should throw on invalid selector", () => {
			const html = "<div><span>text</span></div>";
			const soup = new Soup(html);
			const span = soup.find("span");

			assert.throws(() => {
				span.closest("[[[invalid");
			});
		});

		it("closest should exclude self", () => {
			const html = '<div class="target"><span class="target">text</span></div>';
			const soup = new Soup(html);
			const span = soup.find("span");

			const result = span.closest(".target");
			assert.notStrictEqual(result, null);
			assert.strictEqual(result.name, "div"); // Parent, not self
		});
	});

	describe("nextSiblings (Phase 12)", () => {
		it("nextSiblings getter should return following elements", () => {
			const html = '<div><span id="a">A</span><span id="b">B</span><span id="c">C</span></div>';
			const soup = new Soup(html);
			const first = soup.find("#a");

			const siblings = first.nextSiblings;
			assert.strictEqual(siblings.length, 2);
			assert.strictEqual(siblings[0].get("id"), "b");
			assert.strictEqual(siblings[1].get("id"), "c");
		});

		it("nextSiblings should return empty array for last", () => {
			const html = '<div><span id="a">A</span><span id="b">B</span></div>';
			const soup = new Soup(html);
			const last = soup.find("#b");

			assert.strictEqual(last.nextSiblings.length, 0);
		});

		it("nextSiblings should skip text nodes", () => {
			const html = '<div><span id="a">A</span>text<span id="b">B</span></div>';
			const soup = new Soup(html);
			const first = soup.find("#a");

			const siblings = first.nextSiblings;
			assert.strictEqual(siblings.length, 1);
			assert.strictEqual(siblings[0].get("id"), "b");
		});
	});

	describe("prevSiblings (Phase 12)", () => {
		it("prevSiblings getter should return preceding elements", () => {
			const html = '<div><span id="a">A</span><span id="b">B</span><span id="c">C</span></div>';
			const soup = new Soup(html);
			const last = soup.find("#c");

			const siblings = last.prevSiblings;
			assert.strictEqual(siblings.length, 2);
			// Note: prevSiblings returns in reverse order
			assert.strictEqual(siblings[0].get("id"), "b");
			assert.strictEqual(siblings[1].get("id"), "a");
		});

		it("prevSiblings should return empty array for first", () => {
			const html = '<div><span id="a">A</span><span id="b">B</span></div>';
			const soup = new Soup(html);
			const first = soup.find("#a");

			assert.strictEqual(first.prevSiblings.length, 0);
		});
	});

	describe("siblings (Phase 12)", () => {
		it("siblings getter should return all except self", () => {
			const html = '<div><span id="a">A</span><span id="b">B</span><span id="c">C</span></div>';
			const soup = new Soup(html);
			const middle = soup.find("#b");

			const siblings = middle.siblings;
			assert.strictEqual(siblings.length, 2);
			assert.strictEqual(siblings[0].get("id"), "a");
			assert.strictEqual(siblings[1].get("id"), "c");
		});

		it("siblings should return empty array for only child", () => {
			const html = '<div><span id="only">text</span></div>';
			const soup = new Soup(html);
			const only = soup.find("#only");

			assert.strictEqual(only.siblings.length, 0);
		});

		it("siblings should skip text nodes", () => {
			const html = '<div>text1<span id="a">A</span>text2<span id="b">B</span>text3</div>';
			const soup = new Soup(html);
			const first = soup.find("#a");

			const siblings = first.siblings;
			assert.strictEqual(siblings.length, 1);
			assert.strictEqual(siblings[0].get("id"), "b");
		});
	});
});
