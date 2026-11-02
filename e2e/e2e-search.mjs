import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));
page.on('response', (response) => {
	if (response.status() >= 400) console.log('HTTP', response.status(), response.url());
});

await page.goto(base + '/search?q=markdown', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);

console.log('hits:', await page.locator('a[href^="/rust"]').count());
console.log('mark count:', await page.locator('mark').count());
await page.screenshot({ path: '/tmp/shot-search.png' });

await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.waitForTimeout(600);
await page.locator('input[placeholder="Поиск"]').fill('таблицы');
await page.keyboard.press('Enter');
await page.waitForTimeout(1200);
console.log('url after sidebar search:', page.url());
console.log('results on page:', await page.locator('mark').count());

await browser.close();
