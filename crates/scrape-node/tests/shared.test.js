/**
 * Cross-platform integration tests for scrape-rs.
 *
 * This module loads shared test cases from test_cases.json and executes them
 * against the Node.js binding. The same test cases are run by Rust, Python,
 * and WASM test runners to ensure API consistency across all platforms.
 */

const { describe, it, before } = require("node:test");
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");

const testCasesPath = path.join(__dirname, "..", "..", "..", "tests", "shared", "test_cases.json");
const testData = JSON.parse(fs.readFileSync(testCasesPath, "utf-8"));

let Soup;

function runFindAssertion(soup, selector, expected, testId) {
	let result;
	try {
		result = soup.find(selector);
	} catch (e) {
		assert.fail(`[${testId}] Selector '${selector}' failed with error: ${e.message}`);
	}

	if (expected.exists === false) {
		assert.strictEqual(result, null, `[${testId}] Expected selector '${selector}' to find nothing`);
		return;
	}

	if (result === null && expected.exists !== false) {
		assert.fail(`[${testId}] Expected selector '${selector}' to find element`);
	}

	if (expected.text !== undefined) {
		assert.strictEqual(
			result.text,
			expected.text,
			`[${testId}] Text mismatch for selector '${selector}'`,
		);
	}

	if (expected.name !== undefined) {
		assert.strictEqual(
			result.name,
			expected.name,
			`[${testId}] Tag name mismatch for selector '${selector}'`,
		);
	}

	if (expected.attr !== undefined) {
		for (const [key, value] of Object.entries(expected.attr)) {
			assert.strictEqual(result.get(key), value, `[${testId}] Attribute '${key}' mismatch`);
		}
	}

	if (expected.attr_missing !== undefined) {
		assert.strictEqual(
			result.get(expected.attr_missing),
			null,
			`[${testId}] Expected attribute to be missing`,
		);
	}

	if (expected.inner_html !== undefined) {
		assert.strictEqual(result.innerHTML, expected.inner_html, `[${testId}] inner_html mismatch`);
	}

	if (expected.has_class !== undefined) {
		for (const cls of expected.has_class) {
			assert.ok(result.hasClass(cls), `[${testId}] Expected element to have class '${cls}'`);
		}
	}

	if (expected.not_has_class !== undefined) {
		for (const cls of expected.not_has_class) {
			assert.ok(!result.hasClass(cls), `[${testId}] Expected element to NOT have class '${cls}'`);
		}
	}
}

function runFindAllAssertion(soup, selector, expected, testId) {
	let results;
	try {
		results = soup.findAll(selector);
	} catch (e) {
		assert.fail(`[${testId}] find_all('${selector}') failed with error: ${e.message}`);
	}

	if (expected.count !== undefined) {
		assert.strictEqual(
			results.length,
			expected.count,
			`[${testId}] Count mismatch for find_all('${selector}')`,
		);
	}
}

function runFindThenAssertion(soup, selector, chain, expected, testId) {
	const tag = soup.find(selector);
	assert.ok(tag !== null, `[${testId}] find('${selector}') returned null`);

	if (chain === "parent") {
		const result = tag.parent;
		if (expected.exists === false) {
			assert.strictEqual(result, null, `[${testId}] Expected parent to be null`);
		} else {
			assert.ok(result !== null, `[${testId}] Expected parent to exist`);
			if (expected.attr !== undefined) {
				for (const [key, value] of Object.entries(expected.attr)) {
					assert.strictEqual(
						result.get(key),
						value,
						`[${testId}] Parent attribute '${key}' mismatch`,
					);
				}
			}
		}
	} else if (chain === "children") {
		const children = tag.children;
		if (expected.count !== undefined) {
			assert.strictEqual(children.length, expected.count, `[${testId}] Children count mismatch`);
		}
	} else if (chain === "next_sibling") {
		const sibling = tag.nextSibling;
		if (expected.exists === false) {
			assert.strictEqual(sibling, null, `[${testId}] Expected nextSibling to be null`);
		} else {
			assert.ok(sibling !== null, `[${testId}] Expected nextSibling to exist`);
			if (expected.attr !== undefined) {
				for (const [key, value] of Object.entries(expected.attr)) {
					assert.strictEqual(
						sibling.get(key),
						value,
						`[${testId}] nextSibling attribute '${key}' mismatch`,
					);
				}
			}
		}
	} else if (chain === "prev_sibling") {
		const sibling = tag.prevSibling;
		if (expected.exists === false) {
			assert.strictEqual(sibling, null, `[${testId}] Expected prevSibling to be null`);
		} else {
			assert.ok(sibling !== null, `[${testId}] Expected prevSibling to exist`);
			if (expected.attr !== undefined) {
				for (const [key, value] of Object.entries(expected.attr)) {
					assert.strictEqual(
						sibling.get(key),
						value,
						`[${testId}] prevSibling attribute '${key}' mismatch`,
					);
				}
			}
		}
	} else if (chain === "descendants") {
		const descendants = tag.descendants;
		if (expected.count !== undefined) {
			assert.strictEqual(
				descendants.length,
				expected.count,
				`[${testId}] Descendants count mismatch`,
			);
		}
		if (expected.min_count !== undefined) {
			assert.ok(
				descendants.length >= expected.min_count,
				`[${testId}] Expected at least ${expected.min_count} descendants, got ${descendants.length}`,
			);
		}
	} else {
		assert.fail(`[${testId}] Unknown chain method: ${chain}`);
	}
}

function runTextAssertion(soup, expected, testId) {
	const text = soup.text;
	if (expected.contains !== undefined) {
		assert.ok(
			text.includes(expected.contains),
			`[${testId}] Expected text to contain '${expected.contains}'`,
		);
	}
}

function runTitleAssertion(soup, expected, testId) {
	const title = soup.title;
	if (expected.equals !== undefined) {
		assert.strictEqual(title, expected.equals, `[${testId}] Title mismatch`);
	}
	if (expected.is_null === true) {
		assert.strictEqual(title, null, `[${testId}] Expected title to be null`);
	}
}

function runScopedFindAssertion(soup, scope, selector, expected, testId) {
	const scopeTag = soup.find(scope);
	assert.ok(scopeTag !== null, `[${testId}] Scope selector '${scope}' returned null`);

	const result = scopeTag.find(selector);

	if (expected.exists === false) {
		assert.strictEqual(result, null, `[${testId}] Expected scoped selector to find nothing`);
		return;
	}

	if (result === null && expected.exists !== false) {
		assert.fail(
			`[${testId}] Expected scoped selector '${selector}' within '${scope}' to find element`,
		);
	}

	if (expected.text !== undefined) {
		assert.strictEqual(result.text, expected.text, `[${testId}] Scoped text mismatch`);
	}
}

function runScopedFindAllAssertion(soup, scope, selector, expected, testId) {
	const scopeTag = soup.find(scope);
	assert.ok(scopeTag !== null, `[${testId}] Scope selector '${scope}' returned null`);

	const results = scopeTag.findAll(selector);

	if (expected.count !== undefined) {
		assert.strictEqual(
			results.length,
			expected.count,
			`[${testId}] Scoped find_all count mismatch`,
		);
	}
}

function runAssertion(soup, assertion, testId) {
	const method = assertion.method;

	if (method === "find") {
		runFindAssertion(soup, assertion.selector, assertion.expected, testId);
	} else if (method === "find_all") {
		runFindAllAssertion(soup, assertion.selector, assertion.expected, testId);
	} else if (method === "find_then") {
		runFindThenAssertion(soup, assertion.selector, assertion.chain, assertion.expected, testId);
	} else if (method === "text") {
		runTextAssertion(soup, assertion.expected, testId);
	} else if (method === "title") {
		runTitleAssertion(soup, assertion.expected, testId);
	} else if (method === "scoped_find") {
		runScopedFindAssertion(soup, assertion.scope, assertion.selector, assertion.expected, testId);
	} else if (method === "scoped_find_all") {
		runScopedFindAllAssertion(
			soup,
			assertion.scope,
			assertion.selector,
			assertion.expected,
			testId,
		);
	} else {
		assert.fail(`[${testId}] Unknown assertion method: ${method}`);
	}
}

describe("Shared Test Cases", () => {
	before(async () => {
		const mod = await import("../index.js");
		Soup = mod.Soup;
	});

	it("should have correct version", () => {
		assert.strictEqual(testData.version, "1.0");
	});

	for (const suite of testData.test_suites) {
		describe(suite.name, () => {
			for (const testCase of suite.cases) {
				it(`${testCase.id}: ${testCase.description}`, () => {
					const soup = new Soup(testCase.input);

					for (const assertion of testCase.assertions) {
						runAssertion(soup, assertion, testCase.id);
					}
				});
			}
		});
	}
});
