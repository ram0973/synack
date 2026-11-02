import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));
page.on('response', (response) => {
	if (response.status() >= 400) console.log('HTTP', response.status(), response.url());
});

const submit = page.locator('form button[type="submit"]');

await page.goto(base + '/admin', { waitUntil: 'networkidle' });
await page.waitForTimeout(800);
console.log('url after /admin:', page.url());
await page.screenshot({ path: '/tmp/shot-login.png' });

await page.locator('input[type="password"]').fill('wrong');
await submit.click();
await page.waitForTimeout(800);
console.log('wrong password error:', await page.locator('p.text-destructive').count());

await page.locator('input[type="password"]').fill('secret');
await submit.click();
await page.waitForTimeout(1500);
console.log('url after login:', page.url());
await page.screenshot({ path: '/tmp/shot-admin-auth.png' });

await page.locator('button[title="Выйти"]').click();
await page.waitForTimeout(1200);
console.log('url after logout:', page.url());

await browser.close();
