import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));

const trigger = page.locator('[data-slot="sidebar-trigger"]').first();
const sidebar = page.locator('[data-slot="sidebar"]').first();

await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.waitForTimeout(700);
console.log('open, width:', Math.round((await sidebar.boundingBox())?.width ?? 0));

await trigger.click();
await page.waitForTimeout(700);
console.log('after toggle, width:', Math.round((await sidebar.boundingBox())?.width ?? 0));
await page.screenshot({ path: '/tmp/shot-sidebar-closed.png' });

await trigger.click();
await page.waitForTimeout(700);
console.log('after second toggle, width:', Math.round((await sidebar.boundingBox())?.width ?? 0));

await page.setViewportSize({ width: 390, height: 844 });
await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.waitForTimeout(700);

const mobileTrigger = page.locator('[data-slot="sidebar-trigger"]').first();
await mobileTrigger.click();
await page.waitForTimeout(800);
console.log('mobile sheet visible:', await page.getByRole('dialog').isVisible());
await page.screenshot({ path: '/tmp/shot-sidebar-mobile.png' });

await page.keyboard.press('Escape');
await page.waitForTimeout(600);
console.log('mobile sheet closed:', !(await page.getByRole('dialog').isVisible()));

await page.setViewportSize({ width: 1440, height: 900 });
await page.waitForTimeout(400);
await page.keyboard.press('Control+b');
await page.waitForTimeout(600);
console.log('after Ctrl+B, width:', Math.round((await sidebar.boundingBox())?.width ?? 0));

await browser.close();
