#!/usr/bin/env node
/**
 * Compare @fast-scrape/node performance against Cheerio.
 *
 * Run from project root:
 *     node benches/compare_node.mjs
 */

import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const FIXTURES_DIR = join(__dirname, 'fixtures');

/**
 * Run function multiple times and return mean/stdev in milliseconds.
 */
function timed(fn, iterations = 100, warmup = 10) {
    // Warmup iterations
    for (let i = 0; i < warmup; i++) {
        fn();
    }

    // Actual measurement
    const times = [];
    for (let i = 0; i < iterations; i++) {
        const start = process.hrtime.bigint();
        fn();
        const end = process.hrtime.bigint();
        const elapsed = Number(end - start) / 1_000_000; // Convert to ms
        times.push(elapsed);
    }

    const mean = times.reduce((a, b) => a + b, 0) / times.length;
    const variance = times.reduce((sum, t) => sum + Math.pow(t - mean, 2), 0) / times.length;
    const stdev = Math.sqrt(variance);

    return [mean, stdev];
}

/**
 * Load HTML fixture file.
 */
function loadFixture(name) {
    return readFileSync(join(FIXTURES_DIR, name), 'utf-8');
}

async function runBenchmarks() {
    console.log('='.repeat(60));
    console.log('@fast-scrape/node vs Cheerio Benchmark');
    console.log('='.repeat(60));
    console.log();

    // Import libraries
    let Soup, cheerio;
    try {
        // Try to import from local build first
        const localPath = join(__dirname, '..', 'crates', 'scrape-node', 'index.mjs');
        const scrapeModule = await import(localPath);
        Soup = scrapeModule.Soup;
    } catch (err) {
        console.error('Error: @fast-scrape/node not found. Build it first:');
        console.error('  cd crates/scrape-node && pnpm run build');
        console.error('Error details:', err.message);
        process.exit(1);
    }

    try {
        const cheerioModule = await import('cheerio');
        cheerio = cheerioModule.default || cheerioModule;
    } catch (err) {
        console.error('Error: cheerio not found. Install it first:');
        console.error('  pnpm add -D cheerio');
        process.exit(1);
    }

    // Load fixtures
    const smallHtml = loadFixture('small.html');
    const mediumHtml = loadFixture('medium.html');
    const largeHtml = loadFixture('large.html');

    const results = [];

    // Parse benchmarks
    console.log('=== Parse Benchmarks ===');
    console.log();

    for (const [name, html] of [['small', smallHtml], ['medium', mediumHtml], ['large', largeHtml]]) {
        const sizeKb = html.length / 1024;

        // @fast-scrape/node
        const [scrapeMean, scrapeStd] = timed(() => new Soup(html), 100);

        // Cheerio
        const [cheerioMean, cheerioStd] = timed(() => cheerio.load(html), 100);

        const speedup = cheerioMean / scrapeMean;

        console.log(`${name} (${sizeKb.toFixed(1)} KB):`);
        console.log(`  @fast-scrape/node: ${scrapeMean.toFixed(3)}ms +/- ${scrapeStd.toFixed(3)}ms`);
        console.log(`  Cheerio:           ${cheerioMean.toFixed(3)}ms +/- ${cheerioStd.toFixed(3)}ms`);
        console.log(`  Speedup:           ${speedup.toFixed(1)}x`);
        console.log();

        results.push(['parse', name, speedup]);
    }

    // Query benchmarks
    console.log('=== Query Benchmarks (on medium.html) ===');
    console.log();

    const soupScrape = new Soup(mediumHtml);
    const $cheerio = cheerio.load(mediumHtml);

    const queryTests = [
        ['find div', (soup) => soup.find('div'), ($) => $('div').first()],
        ['find .product-card', (soup) => soup.find('.product-card'), ($) => $('.product-card').first()],
        ['find #product-100', (soup) => soup.find('#product-100'), ($) => $('#product-100')],
        ['findAll div', (soup) => soup.findAll('div'), ($) => $('div')],
        ['select .product-card', (soup) => soup.select('.product-card'), ($) => $('.product-card')],
    ];

    for (const [name, scrapeFn, cheerioFn] of queryTests) {
        // @fast-scrape/node
        const [scrapeMean, scrapeStd] = timed(() => scrapeFn(soupScrape), 100);

        // Cheerio
        const [cheerioMean, cheerioStd] = timed(() => cheerioFn($cheerio), 100);

        const speedup = cheerioMean / scrapeMean;

        console.log(`${name}:`);
        console.log(`  @fast-scrape/node: ${scrapeMean.toFixed(3)}ms +/- ${scrapeStd.toFixed(3)}ms`);
        console.log(`  Cheerio:           ${cheerioMean.toFixed(3)}ms +/- ${cheerioStd.toFixed(3)}ms`);
        console.log(`  Speedup:           ${speedup.toFixed(1)}x`);
        console.log();

        results.push(['query', name, speedup]);
    }

    // Summary
    console.log('='.repeat(60));
    console.log('Summary');
    console.log('='.repeat(60));
    console.log();

    const parseSpeedups = results.filter(r => r[0] === 'parse').map(r => r[2]);
    const querySpeedups = results.filter(r => r[0] === 'query').map(r => r[2]);

    const avgParse = parseSpeedups.reduce((a, b) => a + b, 0) / parseSpeedups.length;
    const avgQuery = querySpeedups.reduce((a, b) => a + b, 0) / querySpeedups.length;

    console.log(`Average Parse Speedup:  ${avgParse.toFixed(1)}x faster than Cheerio`);
    console.log(`Average Query Speedup:  ${avgQuery.toFixed(1)}x faster than Cheerio`);
    console.log();

    if (avgParse >= 10) {
        console.log('Parse target (10x+): PASSED');
    } else {
        console.log(`Parse target (10x+): FAILED (${avgParse.toFixed(1)}x)`);
    }

    if (avgQuery >= 10) {
        console.log('Query target (10x+): PASSED');
    } else {
        console.log(`Query target (10x+): MISSED (${avgQuery.toFixed(1)}x)`);
    }
}

runBenchmarks().catch(err => {
    console.error('Error:', err);
    process.exit(1);
});
