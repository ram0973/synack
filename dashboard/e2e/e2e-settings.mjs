import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4187';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1500, height: 950 } });

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').first().fill('admin');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1500);

await page.goto(base + '/admin/settings', { waitUntil: 'networkidle' });
await page.waitForTimeout(1000);
console.log('h1:', (await page.locator('h1').first().innerText()).trim());

await page.locator('#author-name').fill('Роман');
await page.locator('input[type="file"]').setInputFiles('/tmp/avatar.png');
await page.waitForTimeout(1500);
await page.getByRole('button', { name: 'Сохранить' }).click();
await page.waitForTimeout(1200);

await page.reload({ waitUntil: 'networkidle' });
await page.waitForTimeout(1200);
console.log('имя после перезагрузки:', await page.locator('#author-name').inputValue());
console.log('аватар в настройках:', await page.locator('img[alt="Аватар"]').count());
console.log('автор в сайдбаре:', await page.getByText('Роман', { exact: true }).count());
console.log('аватар в сайдбаре:', await page.locator('aside img, [data-sidebar="footer"] img').count());

await page.screenshot({ path: '/tmp/shot-settings.png' });

await page.goto(base + '/admin/password', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);
console.log('редирект /admin/password →', new URL(page.url()).pathname);

await browser.close();
