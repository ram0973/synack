import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').fill('secret');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1200);

await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.waitForTimeout(700);
await page.screenshot({ path: '/tmp/shot-home-newpage.png' });

await page.getByRole('button', { name: 'Новая страница' }).click();
await page.waitForTimeout(900);
console.log('url после кнопки на главной:', page.url());
console.log('редактор открыт:', await page.locator('input[placeholder="Заголовок"]').isVisible());

await page.goto(base + '/admin', { waitUntil: 'networkidle' });
await page.waitForTimeout(800);
console.log('кнопок «Новая страница» в админке:', await page.getByRole('button', { name: 'Новая страница' }).count());

await page.locator('[data-slot="dropdown-menu-trigger"]').first().click();
await page.waitForTimeout(500);
console.log('пунктов в меню категории:', await page.locator('[role="menuitem"]').count());
await page.screenshot({ path: '/tmp/shot-admin-menu.png' });

await browser.close();
