import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4197';
const browser = await chromium.launch();

// --- аноним: аватара нет
const anon = await browser.newContext({ viewport: { width: 1500, height: 950 } });
const anonPage = await anon.newPage();
await anonPage.goto(base + '/wiki', { waitUntil: 'networkidle' });
await anonPage.waitForTimeout(900);
const anonFooter = await anonPage.locator('[data-sidebar="footer"]').innerText();
console.log('аноним, автор в футере:', anonFooter.includes('Роман'));
console.log('аноним, версия в футере:', anonFooter.includes('v0.1.0'));
await anon.close();

// --- админ
const ctx = await browser.newContext({ viewport: { width: 1500, height: 950 } });
const page = await ctx.newPage();

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').first().fill('admin');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1500);

await page.goto(base + '/wiki', { waitUntil: 'networkidle' });
await page.waitForTimeout(1000);

const cisco = page.getByRole('button', { name: 'Cisco' }).first();
await cisco.click();
await page.waitForTimeout(700);

const leaf = page.getByRole('button', { name: 'Настройка' }).first();
console.log('листовая подкатегория, svg:', await leaf.locator('svg').count());
console.log('title подкатегории:', await leaf.getAttribute('title'));

await leaf.click();
await page.waitForTimeout(900);
console.log('клик по листу →', new URL(page.url()).pathname);

await page.goto(base + '/wiki', { waitUntil: 'networkidle' });
await page.waitForTimeout(800);
await page.getByRole('button', { name: 'Cisco' }).first().click();
await page.waitForTimeout(700);

const pageRow = page.getByRole('link', { name: 'Первоначальная настройка' }).first();
console.log('страница title-атрибут:', await pageRow.getAttribute('title'));
const pagePad = await pageRow.evaluate((el) => getComputedStyle(el).paddingLeft);
const catPad = await page.getByRole('button', { name: 'Cisco' }).first().evaluate((el) => getComputedStyle(el).paddingLeft);
console.log('отступ страницы/категории:', pagePad, '/', catPad);

const adminFooter = await page.locator('[data-sidebar="footer"]').innerText();
console.log('админ, автор в футере:', adminFooter.includes('Роман'));
console.log('админ, версия в футере:', adminFooter.includes('v0.1.0'));

await page.screenshot({ path: '/tmp/shot-sidebar-menu.png', clip: { x: 0, y: 0, width: 320, height: 560 } });
await browser.close();
