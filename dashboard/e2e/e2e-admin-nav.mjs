import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4189';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1500, height: 950 } });

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').first().fill('admin');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1500);

await page.goto(base + '/admin/wiki', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);
console.log('вкладки:', await page.getByRole('link', { name: 'Вики' }).count(), await page.getByRole('link', { name: 'Блог' }).count(), await page.getByRole('link', { name: 'Настройки' }).count());
console.log('подразделы вики:', await page.getByRole('link', { name: 'Категории' }).count(), await page.getByRole('link', { name: 'Страницы' }).count());
await page.screenshot({ path: '/tmp/shot-admin-nav-wiki.png' });

await page.getByRole('link', { name: 'Блог' }).click();
await page.waitForTimeout(900);
console.log('url:', new URL(page.url()).pathname);
console.log('h1:', (await page.locator('h1').first().innerText()).trim());
console.log('подразделы на блоге:', await page.getByRole('link', { name: 'Категории' }).count());
await page.screenshot({ path: '/tmp/shot-admin-nav-blog.png' });

await page.getByRole('link', { name: 'Настройки' }).click();
await page.waitForTimeout(900);
console.log('url:', new URL(page.url()).pathname);
console.log('h1:', (await page.locator('h1').first().innerText()).trim());

await browser.close();
