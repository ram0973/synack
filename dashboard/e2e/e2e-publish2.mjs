import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('dialog', (dialog) => dialog.accept());
page.on('pageerror', (error) => console.log('PAGEERROR', error.message));
page.on('response', (response) => {
	if (response.status() >= 400) console.log('HTTP', response.status(), response.url());
});

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').fill('secret');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1200);

await page.goto(base + '/admin/page/new?category=1', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);

for (const label of ['Заголовок', 'Категория', 'Ссылка (slug)']) {
	console.log('label', label + ':', await page.getByText(label, { exact: true }).count());
}

await page.locator('#page-title').fill('Заметка');
await page.locator('textarea').fill('# Заметка\n\nтекст');
await page.screenshot({ path: '/tmp/shot-editor-labels.png' });

await page.getByRole('button', { name: 'Опубликовать' }).click();
await page.waitForTimeout(1600);
console.log('url после публикации:', page.url());
console.log('кнопка «Отменить публикацию»:', await page.getByRole('button', { name: 'Отменить публикацию' }).count());
await page.screenshot({ path: '/tmp/shot-editor-published.png' });

await page.getByRole('button', { name: 'Отменить публикацию' }).click();
await page.waitForTimeout(1400);
console.log('после отмены, кнопка «Опубликовать»:', await page.getByRole('button', { name: 'Опубликовать' }).count());
console.log('бейдж черновик:', await page.getByText('черновик').count());

await browser.close();
