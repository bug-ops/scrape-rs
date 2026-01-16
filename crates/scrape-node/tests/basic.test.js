const { describe, it } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");

describe("scrape-rs", () => {
	it("should have built native binary", () => {
		const bindings = fs.readdirSync(path.join(__dirname, "..")).filter((f) => f.endsWith(".node"));
		assert.ok(bindings.length > 0, "Native binary should be built");
	});
});
