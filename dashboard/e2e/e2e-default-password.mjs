import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));

const banner = page.getByText('Вы вошли со стандартным паролем', { exact: false });

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').first().fill('admin');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1500);

await page.goto(base + '/admin', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);
console.log('баннер со стандартным паролем:', await banner.count());
await page.screenshot({ path: '/tmp/shot-default-banner.png' });

await page.goto(base + '/admin/password', { waitUntil: 'networkidle' });
await page.waitForTimeout(700);
await page.locator('#pw-current').fill('admin');
await page.locator('#pw-next').fill('newpass123');
await page.locator('#pw-repeat').fill('newpass123');
await page.getByRole('button', { name: 'Сменить пароль' }).click();
await page.waitForTimeout(1500);

await page.goto(base + '/admin', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);
console.log('баннер после смены:', await banner.count());

await browser.close();
