import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('response', (response) => {
	if (response.status() >= 400) {
		console.log('HTTP', response.status(), response.request().method(), response.url());
	}
});
page.on('pageerror', (error) => console.log('PAGEERROR', error.message));

const routes = ['/', '/admin', '/rust', '/rust/async/vvedenie', '/admin/page/1'];

for (const route of routes) {
	console.log('--- route', route);
	await page.goto(base + route, { waitUntil: 'networkidle' });
	await page.waitForTimeout(400);
}

console.log('--- open dialog');
await page.goto(base + '/admin', { waitUntil: 'networkidle' });
await page.getByRole('button', { name: 'Новая категория' }).click();
await page.waitForTimeout(500);
await page.screenshot({ path: '/tmp/shot-dialog.png' });
console.log('dialog visible:', await page.getByRole('dialog').isVisible());

await page.keyboard.press('Escape');
await page.waitForTimeout(300);

console.log('--- open dropdown');
const trigger = page.locator('[data-slot="dropdown-menu-trigger"]').first();
await trigger.click();
await page.waitForTimeout(400);
await page.screenshot({ path: '/tmp/shot-menu.png' });
console.log('menu items:', await page.locator('[role="menuitem"]').count());

await browser.close();
