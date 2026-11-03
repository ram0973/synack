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

const cisco = page.getByRole('button', { name: 'Cisco' }).first();

console.log('CCNA до клика:', await page.getByText('CCNA', { exact: true }).count());
console.log('страница до клика:', await page.getByText('Маршрутизация', { exact: true }).count());
console.log('иконка повёрнута:', await cisco.locator('svg').last().evaluate((el) => el.className.baseVal?.includes('rotate-90') ?? el.getAttribute('class')?.includes('rotate-90')));

await cisco.click();
await page.waitForTimeout(700);

console.log('CCNA после клика:', await page.getByText('CCNA', { exact: true }).count());
console.log('страница после клика:', await page.getByText('Маршрутизация', { exact: true }).count());
console.log('иконка повёрнута:', await cisco.locator('svg').last().evaluate((el) => el.getAttribute('class')?.includes('rotate-90')));

await page.screenshot({ path: '/tmp/shot-sidebar-expanded.png', clip: { x: 0, y: 0, width: 300, height: 500 } });

await cisco.click();
await page.waitForTimeout(700);
console.log('CCNA после повторного клика:', await page.getByText('CCNA', { exact: true }).count());

await browser.close();
