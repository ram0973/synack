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

await page.goto(base + '/admin/page/new?category=1', { waitUntil: 'networkidle' });
await page.waitForTimeout(800);

console.log('save button:', await page.getByRole('button', { name: 'Сохранить', exact: true }).count());
console.log('publish button:', await page.getByRole('button', { name: 'Опубликовать' }).count());
console.log('status select:', await page.locator('select').count());

await page.locator('input[placeholder="Заголовок"]').fill('Заметка');
await page.locator('textarea').fill('# Заметка\n\nтекст');
await page.getByRole('button', { name: 'Опубликовать' }).click();
await page.waitForTimeout(1500);
console.log('url after publish:', page.url());

await page.goto(base + '/admin/page/1', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);
console.log('badge published:', await page.getByText('опубликовано').count());
console.log('publish disabled:', await page.getByRole('button', { name: 'Опубликовать' }).isDisabled());

await page.screenshot({ path: '/tmp/shot-publish.png' });
await browser.close();
