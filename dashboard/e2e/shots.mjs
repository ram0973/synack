import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';

const shots = [
	['home', '/'],
	['admin', '/admin'],
	['page', '/rust/async/vvedenie'],
	['editor', '/admin/page/1'],
	['category', '/rust']
];

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

const errors = [];
page.on('console', (message) => {
	if (message.type() === 'error') errors.push('console: ' + message.text());
});
page.on('pageerror', (error) => errors.push('pageerror: ' + error.message));

for (const [name, path] of shots) {
	await page.goto(base + path, { waitUntil: 'networkidle' });
	await page.waitForTimeout(600);
	await page.screenshot({ path: '/tmp/shot-' + name + '.png', fullPage: true });
	console.log('shot:', name);
}

await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.evaluate(() => document.documentElement.classList.add('dark'));
await page.waitForTimeout(400);
await page.screenshot({ path: '/tmp/shot-dark.png', fullPage: true });
console.log('shot: dark');

if (errors.length) {
	console.log('--- browser errors:');
	for (const line of errors) console.log(line);
} else {
	console.log('--- no browser errors');
}

await browser.close();
