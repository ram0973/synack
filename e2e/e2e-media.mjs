import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));
page.on('response', (response) => {
	if (response.status() >= 400) console.log('HTTP', response.status(), response.url());
});

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').fill('secret');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1200);

await page.goto(base + '/admin/page/1', { waitUntil: 'networkidle' });
await page.waitForTimeout(1000);

await page.locator('input[type="file"]').setInputFiles('/tmp/test2.png');
await page.waitForTimeout(2000);

const value = await page.locator('textarea').inputValue();
console.log('markdown contains /media/:', value.includes('/media/'));

await page.getByRole('tab', { name: 'Превью' }).click();
await page.waitForTimeout(1200);

const image = page.locator('.md img').first();
console.log('image count:', await page.locator('.md img').count());
console.log('image visible:', await image.isVisible());
console.log('natural size:', await image.evaluate((node) => node.naturalWidth + 'x' + node.naturalHeight));

await page.screenshot({ path: '/tmp/shot-editor-image.png' });

await browser.close();
