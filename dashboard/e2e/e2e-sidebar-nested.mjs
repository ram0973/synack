import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4195';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1500, height: 950 } });

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').first().fill('admin');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1500);

await page.goto(base + '/wiki', { waitUntil: 'networkidle' });
await page.waitForTimeout(1000);

const cisco = page.getByRole('button', { name: 'Cisco' }).first();
console.log('Cisco стрелка:', await cisco.locator('svg').count());
await cisco.click();
await page.waitForTimeout(600);

const ccna = page.getByRole('button', { name: 'CCNA' }).first();
console.log('CCNA стрелка:', await ccna.locator('svg').count());
console.log('CCNA aria-expanded:', await ccna.getAttribute('aria-expanded'));
await ccna.click();
await page.waitForTimeout(600);
const secRow = page.getByRole('button', { name: 'Security' }).first();
await secRow.click();
await page.waitForTimeout(600);
console.log('ACL в дереве:', await page.getByText('ACL', { exact: true }).count());

const sec = page.getByRole('button', { name: 'Security' }).first();
console.log('Security стрелка:', await sec.locator('svg').count());
console.log('Security в дереве:', await sec.count());
console.log('страница CCNA Basics:', await page.getByText('CCNA Basics', { exact: true }).count());

await page.screenshot({ path: '/tmp/shot-sidebar-nested.png', clip: { x: 0, y: 0, width: 320, height: 560 } });
await browser.close();
