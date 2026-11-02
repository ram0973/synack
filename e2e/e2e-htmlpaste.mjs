import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').first().fill('admin');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1500);

await page.goto(base + '/admin/page/new?category=1', { waitUntil: 'networkidle' });
await page.waitForTimeout(1500);

console.log('cm-editor:', await page.locator('.cm-editor').count());

await page.locator('.cm-content').click();

await page.evaluate(() => {
	const data = new DataTransfer();
	data.setData(
		'text/html',
		'<h2>Заголовок</h2><p>Текст <strong>жирный</strong> и <a href="https://example.com">ссылка</a></p><ul><li>раз</li><li>два</li></ul><p><img src="https://example.com/pic.png" alt="pic"></p>'
	);
	data.setData('text/plain', 'Заголовок Текст жирный и ссылка');

	const target = document.querySelector('.cm-content');
	target?.dispatchEvent(
		new ClipboardEvent('paste', { clipboardData: data, bubbles: true, cancelable: true })
	);
});

await page.waitForTimeout(900);

const text = await page.locator('.cm-content').innerText();
console.log('markdown:', JSON.stringify(text));

for (const expected of [
	'## Заголовок',
	'**жирный**',
	'[ссылка](https://example.com)',
	'![pic](https://example.com/pic.png)'
]) {
	console.log(expected, '→', text.includes(expected));
}

await page.getByRole('button', { name: 'Превью' }).click();
await page.waitForTimeout(1000);
console.log('пунктов списка в превью:', await page.locator('.md ul li').count());
console.log('картинок в превью:', await page.locator('.md img').count());

await page.screenshot({ path: '/tmp/shot-htmlpaste.png' });
await browser.close();
