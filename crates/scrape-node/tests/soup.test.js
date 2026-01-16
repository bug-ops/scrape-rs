const { describe, it, before } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const os = require("node:os");

let Soup, version;

describe("Soup", () => {
	before(async () => {
		const mod = await import("../index.js");
		Soup = mod.Soup;
		version = mod.version;
	});

	describe("parsing", () => {
		it("should parse simple HTML", () => {
			const soup = new Soup("<html><body><div>Hello</div></body></html>");
			assert.ok(soup.root !== null);
		});

		it("should parse with config", () => {
			const config = { maxDepth: 100, preserveWhitespace: true };
			const soup = new Soup("<div>Hello</div>", config);
			assert.ok(soup.root !== null);
		});

		it("should handle empty HTML", () => {
			const soup = new Soup("");
			assert.strictEqual(soup.length, 0);
		});

		it("should handle malformed HTML", () => {
			const soup = new Soup("<div><span>unclosed");
			assert.ok(soup.root !== null);
		});
	});

	describe("properties", () => {
		it("should extract title", () => {
			const soup = new Soup("<html><head><title>Test Page</title></head></html>");
			assert.strictEqual(soup.title, "Test Page");
		});

		it("should return null for missing title", () => {
			const soup = new Soup("<html><body>no title</body></html>");
			assert.strictEqual(soup.title, null);
		});

		it("should strip tags from text", () => {
			const soup = new Soup("<div>Hello <b>World</b></div>");
			const text = soup.text;
			assert.ok(text.includes("Hello"));
			assert.ok(text.includes("World"));
			assert.ok(!text.includes("<b>"));
		});

		it("should get root element", () => {
			const soup = new Soup("<html><body></body></html>");
			assert.strictEqual(soup.root.name, "html");
		});

		it("should convert to HTML", () => {
			const soup = new Soup("<div><span>text</span></div>");
			const html = soup.toHtml();
			assert.ok(html.includes("<div>"));
			assert.ok(html.includes("<span>"));
		});

		it("should report correct length", () => {
			const soup = new Soup("<div><span>A</span><span>B</span></div>");
			assert.ok(soup.length > 0);
		});
	});

	describe("find", () => {
		it("should find by tag name", () => {
			const soup = new Soup("<div><span>Hello</span></div>");
			const span = soup.find("span");
			assert.ok(span !== null);
			assert.strictEqual(span.name, "span");
		});

		it("should find by class", () => {
			const soup = new Soup('<div class="item">First</div>');
			const item = soup.find(".item");
			assert.ok(item !== null);
			assert.ok(item.text.includes("First"));
		});

		it("should find by id", () => {
			const soup = new Soup('<div id="main">Content</div>');
			const main = soup.find("#main");
			assert.ok(main !== null);
			assert.strictEqual(main.get("id"), "main");
		});

		it("should find with compound selector", () => {
			const soup = new Soup('<li class="item active">First</li>');
			const active = soup.find("li.item.active");
			assert.ok(active !== null);
		});

		it("should find with descendant combinator", () => {
			const soup = new Soup("<div><ul><li>Item</li></ul></div>");
			const li = soup.find("div li");
			assert.ok(li !== null);
		});

		it("should find with child combinator", () => {
			const soup = new Soup("<ul><li>A</li><li>B</li><li>C</li></ul>");
			const items = soup.findAll("ul > li");
			assert.strictEqual(items.length, 3);
		});

		it("should find with attribute selector", () => {
			const soup = new Soup('<a href="/link">Click me</a>');
			const link = soup.find('a[href="/link"]');
			assert.ok(link !== null);
			assert.strictEqual(link.text, "Click me");
		});

		it("should return null when not found", () => {
			const soup = new Soup("<div>Content</div>");
			const result = soup.find(".nonexistent");
			assert.strictEqual(result, null);
		});

		it("should return array from findAll", () => {
			const soup = new Soup('<ul><li class="item">A</li><li class="item">B</li></ul>');
			const items = soup.findAll(".item");
			assert.ok(Array.isArray(items));
			assert.strictEqual(items.length, 2);
		});

		it("should have select as alias for findAll", () => {
			const soup = new Soup('<ul><li class="item">A</li><li class="item">B</li></ul>');
			const items1 = soup.findAll(".item");
			const items2 = soup.select(".item");
			assert.strictEqual(items1.length, items2.length);
		});

		it("should return empty array when findAll matches nothing", () => {
			const soup = new Soup("<div>Content</div>");
			const result = soup.findAll(".nonexistent");
			assert.ok(Array.isArray(result));
			assert.strictEqual(result.length, 0);
		});
	});

	describe("errors", () => {
		it("should throw on invalid selector", () => {
			const soup = new Soup("<div>Content</div>");
			assert.throws(() => soup.find("div[[["), /selector/i);
		});
	});

	describe("fromFile", () => {
		it("should parse from file", () => {
			const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "scrape-test-"));
			const tmpFile = path.join(tmpDir, "test.html");
			fs.writeFileSync(tmpFile, "<div>Test</div>");

			try {
				const soup = Soup.fromFile(tmpFile);
				assert.strictEqual(soup.find("div").text, "Test");
			} finally {
				fs.rmSync(tmpDir, { recursive: true });
			}
		});

		it("should throw on file not found", () => {
			assert.throws(() => Soup.fromFile("/nonexistent/path.html"), /error/i);
		});
	});

	describe("version", () => {
		it("should return version string", () => {
			const v = version();
			assert.ok(typeof v === "string");
			assert.ok(v.length > 0);
		});
	});
});
