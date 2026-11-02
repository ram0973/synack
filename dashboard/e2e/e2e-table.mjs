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

await page.getByRole('button', { name: 'Таблица' }).click();
await page.waitForTimeout(600);
console.log('диалог таблицы:', await page.getByRole('dialog').isVisible());

await page.locator('#table-cols').fill('4');
await page.locator('#table-rows').fill('2');
await page.getByRole('button', { name: 'Вставить' }).click();
await page.waitForTimeout(900);

const text = await page.locator('.cm-content').innerText();
console.log('заголовок таблицы:', text.includes('| Столбец 1 | Столбец 2 | Столбец 3 | Столбец 4 |'));
console.log('разделитель:', text.includes('| --- | --- | --- | --- |'));

await page.getByRole('button', { name: 'Превью' }).click();
await page.waitForTimeout(1000);
console.log('строк в таблице превью:', await page.locator('.md table tr').count());
console.log('колонок:', await page.locator('.md table tr').first().locator('th').count());

await page.screenshot({ path: '/tmp/shot-table.png' });
await browser.close();
