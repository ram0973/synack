import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4181';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1400, height: 900 } });

await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.waitForTimeout(800);
console.log('без входа, пунктов «Админка»:', await page.getByRole('button', { name: 'Админка' }).count());

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').first().fill('admin');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1500);

await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.waitForTimeout(800);
console.log('после входа, пунктов «Админка»:', await page.getByRole('button', { name: 'Админка' }).count());

await page.screenshot({ path: '/tmp/shot-sidebar-admin.png', clip: { x: 0, y: 0, width: 300, height: 340 } });
await browser.close();
