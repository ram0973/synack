import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').fill('secret');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1200);

await page.goto(base + '/admin/page/new?category=1', { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);

await page.getByRole('button', { name: 'Ссылка' }).click();
await page.waitForTimeout(500);
await page.locator('#link-text').fill('документация');
await page.locator('#link-url').fill('https://example.com/docs');
await page.getByRole('button', { name: 'Вставить' }).click();
await page.waitForTimeout(700);

await page.getByRole('button', { name: 'Врезка' }).click();
await page.waitForTimeout(500);
await page.locator('#callout-type').selectOption('warning');
await page.locator('#callout-text').fill('Не делайте так.');
await page.getByRole('button', { name: 'Вставить' }).click();
await page.waitForTimeout(700);

const text = await page.locator('.cm-content').innerText();
console.log('ссылка в тексте:', text.includes('[документация](https://example.com/docs)'));
console.log('врезка в тексте:', text.includes(':::warning'));

await page.getByRole('button', { name: 'Превью' }).click();
await page.waitForTimeout(1000);
console.log('ссылка в превью:', await page.locator('.md a').count());
console.log('врезка в превью:', await page.locator('.md .callout-warning').count());

await page.screenshot({ path: '/tmp/shot-callout.png' });
await browser.close();
