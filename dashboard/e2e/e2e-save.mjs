import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4181';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1600, height: 1000 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').first().fill('admin');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1500);

await page.goto(base + '/admin/wiki/page/new?category=1', { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);

await page.locator('#page-title').fill('Новая статья');
await page.locator('.cm-content').click();
await page.keyboard.type('Текст статьи.');
await page.waitForTimeout(600);
await page.getByRole('button', { name: 'Сохранить' }).click();
await page.waitForTimeout(2000);
console.log('url после сохранения:', new URL(page.url()).pathname);

await page.goto(base + '/wiki', { waitUntil: 'networkidle' });
await page.waitForTimeout(1000);
console.log('новая статья в списке:', await page.getByText('Новая статья').count());

await page.goto(base + '/admin/password', { waitUntil: 'networkidle' });
await page.waitForTimeout(1000);
console.log('страница пароля:', await page.getByText('Текущий пароль').count());

await browser.close();
