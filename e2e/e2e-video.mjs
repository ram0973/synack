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

console.log('сегменты:', await page.getByRole('button', { name: 'Текст' }).count(), await page.getByRole('button', { name: 'Превью' }).count());

await page.getByRole('button', { name: 'Видео' }).click();
await page.waitForTimeout(600);
console.log('диалог видео:', await page.getByRole('dialog').isVisible());
await page.screenshot({ path: '/tmp/shot-video-dialog.png' });

await page.locator('input[placeholder^="https://youtube"]').fill('https://youtu.be/dQw4w9WgXcQ');
await page.getByRole('button', { name: 'Вставить с YouTube' }).click();
await page.waitForTimeout(1000);

const text = await page.locator('.cm-content').innerText();
console.log('директива в тексте:', text.includes('::youtube{id="dQw4w9WgXcQ"}'));

await page.getByRole('button', { name: 'Превью' }).click();
await page.waitForTimeout(1200);
console.log('iframe в превью:', await page.locator('.md iframe').count());
await page.screenshot({ path: '/tmp/shot-video-preview.png' });

await browser.close();
