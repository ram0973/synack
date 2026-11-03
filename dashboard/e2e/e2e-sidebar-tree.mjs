import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4191';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1500, height: 950 } });

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').first().fill('admin');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1500);

await page.goto(base + '/wiki', { waitUntil: 'networkidle' });
await page.waitForTimeout(1000);

const wikiHeader = page.getByRole('link', { name: 'Вики' }).first();
const cisco = page.getByRole('link', { name: 'Cisco' }).first();
const blogLabel = page.getByText('Блог', { exact: true }).first();
const blogHint = page.getByText('появится позже', { exact: true });
const admin = page.getByRole('button', { name: 'Админка' }).first();

console.log('заголовок раздела Вики:', await wikiHeader.count());
console.log('Cisco под Вики, y:', Math.round((await cisco.boundingBox())?.y ?? -1), 'заголовок y:', Math.round((await wikiHeader.boundingBox())?.y ?? -1));
console.log('Блог:', await blogLabel.count(), 'подсказка:', await blogHint.count());
console.log('Админка внизу:', ((await admin.boundingBox())?.y ?? 0) > ((await cisco.boundingBox())?.y ?? 0) + 40);

await page.screenshot({ path: '/tmp/shot-sidebar-tree.png', clip: { x: 0, y: 0, width: 300, height: 950 } });
await browser.close();
