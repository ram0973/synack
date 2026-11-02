import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4181';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1600, height: 1000 } });

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').first().fill('admin');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1500);

await page.goto(base + '/admin/wiki', { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);

console.log('вкладок Блог/Страницы:', await page.getByText('Блог', { exact: true }).count(), await page.getByText('Страницы', { exact: true }).count());
console.log('крупный заголовок:', await page.locator('h1').first().innerText());
console.log('размер заголовка:', await page.locator('h1').first().evaluate((el) => getComputedStyle(el).fontSize));

await page.screenshot({ path: '/tmp/shot-admin-clean.png' });
await browser.close();
