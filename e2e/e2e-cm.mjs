import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.locator('input[type="password"]').fill('secret');
await page.locator('form button[type="submit"]').click();
await page.waitForTimeout(1200);

await page.goto(base + '/admin/page/new?category=1', { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);

console.log('CodeMirror на странице:', await page.locator('.cm-editor').count());

await page.locator('.cm-content').click();
await page.keyboard.type('# Заголовок\n\nТекст с **жирным** и кодом:\n\n```rust\nfn main() {}\n```\n');
await page.waitForTimeout(600);

const spans = page.locator('.cm-content .cm-line span');
console.log('спанов подсветки:', await spans.count());
const colors = await spans.evaluateAll((nodes) =>
	[...new Set(nodes.map((node) => getComputedStyle(node).color))].slice(0, 6)
);
console.log('цвета токенов:', JSON.stringify(colors));

await page.screenshot({ path: '/tmp/shot-cm-edit.png' });

await page.getByRole('tab', { name: 'Превью' }).click();
await page.waitForTimeout(1000);
console.log('pre в превью:', await page.locator('.md pre').count());

await browser.close();
