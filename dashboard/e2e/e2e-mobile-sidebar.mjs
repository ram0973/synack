import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4181';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 390, height: 844 } });

await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);

async function openSidebar() {
	await page.locator('[data-sidebar="trigger"]').click();
	await page.waitForTimeout(600);
}

await openSidebar();
console.log('панель после открытия:', await page.getByRole('dialog').count());
await page.screenshot({ path: '/tmp/shot-mobile-open.png' });

await page.getByRole('button', { name: 'Вики' }).click();
await page.waitForTimeout(900);
console.log('url после клика «Вики»:', new URL(page.url()).pathname);
console.log('панель после перехода:', await page.getByRole('dialog').count());

await openSidebar();
const login = page.locator('button[title="Войти"]');
console.log('кнопка входа в футере:', await login.count());
await login.first().click();
await page.waitForTimeout(1000);
console.log('url после входа:', new URL(page.url()).pathname);
console.log('панель после входа:', await page.getByRole('dialog').count());

await page.screenshot({ path: '/tmp/shot-mobile-closed.png' });
await browser.close();
