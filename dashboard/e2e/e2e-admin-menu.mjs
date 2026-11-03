import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4187';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1500, height: 950 } });

await page.goto(base + '/wiki', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);
console.log('подпись на /wiki:', (await page.locator('header span').first().innerText()).trim());

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').first().fill('admin');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1500);

await page.goto(base + '/admin/wiki', { waitUntil: 'networkidle' });
await page.waitForTimeout(1000);
console.log('меню:', await page.getByRole('link', { name: 'Вики' }).count(), await page.getByRole('link', { name: 'Категории' }).count(), await page.getByRole('link', { name: 'Страницы' }).count());
console.log('заголовок:', (await page.locator('h1').first().innerText()).trim());
await page.screenshot({ path: '/tmp/shot-admin-categories.png' });

await page.getByRole('link', { name: 'Страницы' }).click();
await page.waitForTimeout(1000);
console.log('url:', new URL(page.url()).pathname);
console.log('h1:', (await page.locator('h1').first().innerText()).trim());
console.log('страниц в списке:', await page.getByRole('button', { name: 'Открыть' }).count());
await page.screenshot({ path: '/tmp/shot-admin-pages.png' });

await browser.close();
