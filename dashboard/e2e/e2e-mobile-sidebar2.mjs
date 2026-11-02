import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4181';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 390, height: 844 } });

await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);

async function open() {
	await page.locator('[data-sidebar="trigger"]').click();
	await page.waitForTimeout(600);
}

async function dialogs() {
	return page.getByRole('dialog').count();
}

await open();
console.log('открыта:', await dialogs());

await page.getByRole('button', { name: 'Главная' }).click();
await page.waitForTimeout(700);
console.log('после «Главная» (тот же путь):', await dialogs());

await open();
await page.getByRole('button', { name: 'Cisco' }).first().click();
await page.waitForTimeout(700);
console.log('после клика по категории:', await dialogs());

await open();
await page.getByRole('button', { name: 'Вики' }).click();
await page.waitForTimeout(800);
console.log('после «Вики»:', await dialogs(), new URL(page.url()).pathname);

await browser.close();
