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

const content = page.locator('.cm-content');
await content.click();
await page.keyboard.type('черновик текста');
await page.waitForTimeout(400);

console.log('до отмены:', JSON.stringify(await content.innerText()));

await page.getByTitle('Отменить (Ctrl+Z)').click();
await page.waitForTimeout(500);
console.log('после отмены:', JSON.stringify(await content.innerText()));

await page.getByTitle('Повторить (Ctrl+Shift+Z)').click();
await page.waitForTimeout(500);
console.log('после повтора:', JSON.stringify(await content.innerText()));

await page.waitForTimeout(1200);
await page.reload({ waitUntil: 'networkidle' });
await page.waitForTimeout(1500);

console.log('после перезагрузки:', JSON.stringify(await page.locator('.cm-content').innerText()));
console.log('тост о черновике:', await page.getByText('Восстановлен несохранённый черновик').count());

await page.screenshot({ path: '/tmp/shot-draft.png' });
await browser.close();
