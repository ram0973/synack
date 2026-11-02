import { chromium } from 'playwright';

const base = 'https://82.24.174.152.nip.io';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1400, height: 900 } });

await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);
console.log('live без входа, «Админка»:', await page.getByRole('button', { name: 'Админка' }).count());

await page.goto(base + '/admin/wiki', { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);
console.log('live /admin/wiki →', new URL(page.url()).pathname);

await browser.close();
