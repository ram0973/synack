import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4193';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1500, height: 950 } });

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').first().fill('admin');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1500);

// первая страница
await page.goto(base + '/admin/wiki/page/new?category=1', { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);
await page.locator('#page-title').fill('Введение');
await page.locator('.cm-content').click();
await page.keyboard.type('текст');
await page.waitForTimeout(500);
await page.getByRole('button', { name: 'Сохранить' }).click();
await page.waitForTimeout(1800);
console.log('первая сохранена:', new URL(page.url()).pathname);

// вторая с тем же заголовком
await page.goto(base + '/admin/wiki/page/new?category=1', { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);
await page.locator('#page-title').fill('введение');
await page.locator('.cm-content').click();
await page.keyboard.type('текст');
await page.waitForTimeout(500);
await page.getByRole('button', { name: 'Сохранить' }).click();
await page.waitForTimeout(1500);

const body = await page.locator('body').innerText();
console.log('ошибка про заголовок:', body.includes('Страница с таким заголовком уже есть'));
console.log('url остался:', new URL(page.url()).pathname);

await page.screenshot({ path: '/tmp/shot-duplicate-title.png' });
await browser.close();
