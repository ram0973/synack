import { chromium } from 'playwright';

const base = 'https://82.24.174.152.nip.io';
const password = process.env.LIVE_PASSWORD || '';

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1600, height: 1000 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));
page.on('console', (message) => {
	if (message.type() === 'error') console.log('CONSOLE', message.text());
});

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').first().fill(password);
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(2000);
console.log('после входа:', new URL(page.url()).pathname);

await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);
console.log('разделы главной:', await page.getByText('Вики', { exact: true }).count());
await page.screenshot({ path: '/tmp/shot-live-home.png' });

await page.goto(base + '/admin/wiki', { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);
console.log('баннер стандартного пароля:', await page.getByText('Вы вошли со стандартным паролем', { exact: false }).count());
console.log('вкладки:', await page.getByRole('link', { name: 'Вики' }).count());
await page.screenshot({ path: '/tmp/shot-live-admin.png' });

await page.goto(base + '/admin/wiki/page/new?category=1', { waitUntil: 'networkidle' });
await page.waitForTimeout(1500);
console.log('редактор:', await page.locator('.cm-editor').count());

await browser.close();
