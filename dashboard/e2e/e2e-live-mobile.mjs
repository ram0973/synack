import { chromium } from 'playwright';

const base = 'https://82.24.174.152.nip.io';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 390, height: 844 } });

await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);

await page.locator('[data-sidebar="trigger"]').click();
await page.waitForTimeout(700);
console.log('live: панель открыта:', await page.getByRole('dialog').count());

await page.locator('button[title="Войти"]').first().click();
await page.waitForTimeout(1200);
console.log('live: url:', new URL(page.url()).pathname);
console.log('live: панель после входа:', await page.getByRole('dialog').count());

await browser.close();
