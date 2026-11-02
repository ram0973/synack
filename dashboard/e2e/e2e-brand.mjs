import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4181';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1600, height: 1000 } });

await page.goto(base + '/wiki', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);

const brand = page.locator('a[href="/"]').first();
console.log('ник:', (await brand.innerText()).trim());
console.log('href:', await brand.getAttribute('href'));
console.log('title:', await page.title());

await brand.click();
await page.waitForTimeout(700);
console.log('после клика:', new URL(page.url()).pathname);

await page.screenshot({ path: '/tmp/shot-brand.png', clip: { x: 0, y: 0, width: 420, height: 220 } });

await browser.close();
