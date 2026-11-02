import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));

await page.goto(base + '/rust/anchors', { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);

const anchors = page.locator('.md .heading-anchor');
console.log('якорей:', await anchors.count());
console.log('hrefs:', JSON.stringify(await anchors.evaluateAll((nodes) => nodes.map((n) => n.getAttribute('href')))));

const opacityBefore = await anchors.first().evaluate((node) => getComputedStyle(node).opacity);
await page.locator('.md h2').first().hover();
await page.waitForTimeout(400);
const opacityAfter = await anchors.first().evaluate((node) => getComputedStyle(node).opacity);
console.log('opacity до/после наведения:', opacityBefore, '→', opacityAfter);

await anchors.nth(1).click();
await page.waitForTimeout(500);
console.log('hash после клика:', new URL(page.url()).hash);

await page.locator('.md h2').first().hover();
await page.screenshot({ path: '/tmp/shot-anchors.png' });

await browser.close();
