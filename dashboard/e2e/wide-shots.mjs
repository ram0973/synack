import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4181';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1600, height: 1000 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').first().fill('admin');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1200);

await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);
console.log('ширина контента:', await page.locator('main.max-w-6xl, div.max-w-6xl').first().evaluate((el) => el.getBoundingClientRect().width));
await page.screenshot({ path: '/tmp/shot-wide-home.png' });

await page.goto(base + '/admin', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);
await page.screenshot({ path: '/tmp/shot-wide-admin.png' });

await browser.close();
