import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').first().fill('secret');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1200);

await page.goto(base + '/admin/password', { waitUntil: 'networkidle' });
await page.waitForTimeout(800);

console.log('страница пароля:', page.url());
await page.locator('#pw-current').fill('secret');
await page.locator('#pw-next').fill('newpass123');
await page.locator('#pw-repeat').fill('newpass123');
await page.getByRole('button', { name: 'Сменить пароль' }).click();
await page.waitForTimeout(1200);
console.log('тост:', await page.getByText('Пароль изменён, остальные сессии сброшены').count());

await page.locator('button[title="Выйти"]').click();
await page.waitForTimeout(1000);
await page.locator('input[type="password"]').first().fill('newpass123');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1500);
console.log('после входа новым:', page.url());

await page.goto(base + '/admin/password', { waitUntil: 'networkidle' });
await page.waitForTimeout(700);
await page.screenshot({ path: '/tmp/shot-password.png' });

await browser.close();
