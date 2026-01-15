const { describe, it } = require("node:test");
const assert = require("node:assert");

describe("scrape-rs", () => {
	it("should load the module", () => {
		const scrapeRs = require("../index.js");
		assert.ok(scrapeRs);
	});
});
