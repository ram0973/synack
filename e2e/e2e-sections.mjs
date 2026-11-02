import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4181';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1600, height: 1000 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').first().fill('admin');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1500);

await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);
console.log('главная — раздел Вики:', await page.getByText('Вики', { exact: true }).count());
await page.screenshot({ path: '/tmp/shot-sections-home.png' });

await page.goto(base + '/wiki', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);
console.log('wiki страниц:', await page.locator('a[href^="/wiki/rust"]').count());
await page.screenshot({ path: '/tmp/shot-wiki-index.png' });

await page.goto(base + '/wiki/rust/osnovy', { waitUntil: 'networkidle' });
await page.waitForTimeout(1000);
console.log('статья:', (await page.locator('h1').first().innerText()).trim());

await page.goto(base + '/wiki/search?q=основы', { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);
console.log('поиск, ссылка на статью:', await page.locator('a[href="/wiki/rust/osnovy"]').count());

await page.goto(base + '/admin', { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);
console.log('редирект админки:', new URL(page.url()).pathname);
console.log('вкладки:', await page.getByRole('link', { name: 'Вики' }).count());
await page.screenshot({ path: '/tmp/shot-admin-tabs.png' });

await page.goto(base + '/admin/wiki/page/new?category=1', { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);
console.log('редактор:', await page.locator('.cm-editor').count());

await browser.close();
