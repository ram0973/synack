import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('console', (message) => console.log('CONSOLE', message.type(), message.text()));
page.on('pageerror', (error) => console.log('PAGEERROR', error.message));

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').first().fill('secret');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1200);

await page.goto(base + '/admin/page/new?category=1', { waitUntil: 'networkidle' });
await page.waitForTimeout(1500);

console.log('cm-editor:', await page.locator('.cm-editor').count());
console.log('textarea:', await page.locator('textarea').count());
console.log('body text:', (await page.locator('body').innerText()).slice(0, 300));

await browser.close();
