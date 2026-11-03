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

const wiki = page.getByRole('button', { name: 'Вики' }).first();
const cisco = page.getByRole('link', { name: 'Cisco' }).first();
const blog = page.getByRole('button', { name: 'Блог' }).first();
const admin = page.getByRole('button', { name: 'Админка' }).first();

console.log('Блог в меню:', await blog.count());
console.log('Блог помечен «скоро»:', await page.getByText('скоро', { exact: true }).count());
console.log('Блог недоступен:', await blog.getAttribute('aria-disabled'));

const wikiBox = await wiki.boundingBox();
const ciscoBox = await cisco.boundingBox();
const adminBox = await admin.boundingBox();

console.log('y: Вики', Math.round(wikiBox?.y ?? -1), 'Cisco', Math.round(ciscoBox?.y ?? -1), 'Админка', Math.round(adminBox?.y ?? -1));
console.log('Cisco ниже Вики:', (ciscoBox?.y ?? 0) > (wikiBox?.y ?? 0));
console.log('Админка внизу:', (adminBox?.y ?? 0) > (ciscoBox?.y ?? 0) + 40);

await page.screenshot({ path: '/tmp/shot-sidebar-tree.png', clip: { x: 0, y: 0, width: 300, height: 950 } });
await browser.close();
