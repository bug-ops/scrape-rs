const { describe, it, before } = require("node:test");
const assert = require("node:assert");

let parseBatch;

describe("parseBatch", () => {
	before(async () => {
		const mod = await import("../index.js");
		parseBatch = mod.parseBatch;
	});

	it("should parse multiple documents", () => {
		const htmls = ["<div>A</div>", "<div>B</div>", "<div>C</div>"];
		const soups = parseBatch(htmls);
		assert.strictEqual(soups.length, 3);
		const texts = soups.map((s) => s.find("div").text);
		assert.deepStrictEqual(texts, ["A", "B", "C"]);
	});

	it("should preserve order", () => {
		const htmls = Array.from({ length: 100 }, (_, i) => `<div>${i}</div>`);
		const soups = parseBatch(htmls);
		const texts = soups.map((s) => s.find("div").text);
		const expected = Array.from({ length: 100 }, (_, i) => String(i));
		assert.deepStrictEqual(texts, expected);
	});

	it("should handle empty array", () => {
		const soups = parseBatch([]);
		assert.deepStrictEqual(soups, []);
	});

	it("should handle single document", () => {
		const soups = parseBatch(["<div>Single</div>"]);
		assert.strictEqual(soups.length, 1);
		assert.strictEqual(soups[0].find("div").text, "Single");
	});

	it("should accept config", () => {
		const htmls = ["<div>Test</div>"];
		const soups = parseBatch(htmls, { maxDepth: 100 });
		assert.strictEqual(soups.length, 1);
	});

	it("should handle large batch efficiently", () => {
		const htmls = Array.from(
			{ length: 1000 },
			(_, i) => `<html><body><div id="d${i}">Document ${i}</div></body></html>`,
		);
		const start = Date.now();
		const soups = parseBatch(htmls);
		const elapsed = Date.now() - start;

		assert.strictEqual(soups.length, 1000);
		assert.ok(elapsed < 10000, `Batch parsing took too long: ${elapsed}ms`);
	});

	it("should handle malformed HTML in batch", () => {
		const htmls = ["<div>Valid</div>", "<div><span>Unclosed", "<div>Also valid</div>"];
		const soups = parseBatch(htmls);
		assert.strictEqual(soups.length, 3);
		assert.strictEqual(soups[0].find("div").text, "Valid");
		assert.ok(soups[1].find("span") !== null);
		assert.strictEqual(soups[2].find("div").text, "Also valid");
	});

	it("should handle complex HTML in batch", () => {
		const htmls = [
			`<html><head><title>Page 1</title></head><body><div class="content">Content 1</div></body></html>`,
			`<html><head><title>Page 2</title></head><body><div class="content">Content 2</div></body></html>`,
		];
		const soups = parseBatch(htmls);
		assert.strictEqual(soups[0].title, "Page 1");
		assert.strictEqual(soups[1].title, "Page 2");
		assert.strictEqual(soups[0].find(".content").text, "Content 1");
		assert.strictEqual(soups[1].find(".content").text, "Content 2");
	});
});
