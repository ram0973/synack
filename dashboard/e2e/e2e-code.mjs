import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));

const block = page.locator('.md pre').first();

await page.goto(base + '/rust/kod', { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);

console.log('блоков кода:', await page.locator('.md pre').count());
console.log('токенов hljs:', await page.locator('.md pre .hljs-keyword').count());
const lightBg = await block.evaluate((el) => getComputedStyle(el).backgroundColor);
const lightFg = await block.evaluate((el) => getComputedStyle(el).color);
await page.screenshot({ path: '/tmp/shot-code-light.png' });

await page.evaluate(() => document.documentElement.classList.add('dark'));
await page.waitForTimeout(500);
const darkBg = await block.evaluate((el) => getComputedStyle(el).backgroundColor);
const darkFg = await block.evaluate((el) => getComputedStyle(el).color);
await page.screenshot({ path: '/tmp/shot-code-dark.png' });

console.log('light:', lightBg, lightFg);
console.log('dark: ', darkBg, darkFg);
console.log('темы различаются:', lightBg !== darkBg);

await browser.close();
