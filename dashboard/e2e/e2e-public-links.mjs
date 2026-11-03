import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4185';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1400, height: 900 } });

await page.goto(base + '/wiki', { waitUntil: 'networkidle' });
await page.waitForTimeout(1000);

console.log('бейджей «опубликовано»:', await page.getByText('опубликовано').count());
console.log('текстовых путей в карточках:', await page.locator('a[href^="/wiki/notes"] .text-xs').count());

const card = page.getByRole('link', { name: 'Первая' }).first();
const before = await page.getByText('Первая', { exact: true }).first().evaluate((el) => getComputedStyle(el).color);
await card.hover();
await page.waitForTimeout(400);
const after = await page.getByText('Первая', { exact: true }).first().evaluate((el) => getComputedStyle(el).color);
console.log('цвет заголовка до/после наведения:', before, '→', after);

await page.screenshot({ path: '/tmp/shot-wiki-cards.png' });

await page.goto(base + '/wiki/notes/pervaya', { waitUntil: 'networkidle' });
await page.waitForTimeout(800);
const linkColor = await page.locator('.md a').first().evaluate((el) => getComputedStyle(el).color);
console.log('цвет ссылки в тексте:', linkColor);
console.log('бейдж в статье:', await page.getByText('опубликовано').count());
await page.locator('.md a').first().hover();
await page.waitForTimeout(300);
console.log('подчёркивание при наведении:', await page.locator('.md a').first().evaluate((el) => getComputedStyle(el).textDecorationLine));

await page.screenshot({ path: '/tmp/shot-article-links.png' });
await browser.close();
