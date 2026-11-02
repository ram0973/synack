import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('dialog', (dialog) => dialog.accept());
page.on('pageerror', (error) => console.log('PAGEERROR', error.message));
page.on('response', (response) => {
	if (response.status() >= 400) console.log('HTTP', response.status(), response.url());
});

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').fill('secret');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1200);

await page.goto(base + '/admin/page/1', { waitUntil: 'networkidle' });
await page.waitForTimeout(1000);

await page.getByRole('button', { name: 'История' }).click();
await page.waitForTimeout(900);
console.log('dialog visible:', await page.getByRole('dialog').isVisible());
console.log('versions:', await page.getByRole('dialog').getByRole('button', { name: 'Вернуть' }).count());
await page.screenshot({ path: '/tmp/shot-history.png' });

await page.getByRole('dialog').getByRole('button', { name: 'Вернуть' }).last().click();
await page.waitForTimeout(1500);

const value = await page.locator('textarea').inputValue();
console.log('body after restore:', JSON.stringify(value));

await browser.close();
