import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('response', (response) => {
	if (response.status() >= 400) {
		console.log('HTTP', response.status(), response.request().method(), response.url());
	}
});

await page.goto(base + '/admin/page/1', { waitUntil: 'networkidle' });
await page.waitForTimeout(600);

const out = await page.evaluate(() => {
	const attrs = (el) => (el ? [...el.attributes].map((a) => a.name + '=' + a.value) : null);
	const rect = (el) => (el ? el.getBoundingClientRect().toJSON() : null);

	const root = document.querySelector('[data-slot="tabs"]');
	const list = document.querySelector('[data-slot="tabs-list"]');
	const contents = [...document.querySelectorAll('[data-slot="tabs-content"]')];
	const textarea = document.querySelector('textarea');

	return {
		rootAttrs: attrs(root),
		rootStyle: root
			? { display: getComputedStyle(root).display, direction: getComputedStyle(root).flexDirection }
			: null,
		listAttrs: attrs(list),
		contents: contents.map((el) => ({
			attrs: attrs(el),
			rect: rect(el),
			display: getComputedStyle(el).display
		})),
		textarea: rect(textarea)
	};
});

console.log(JSON.stringify(out, null, 1));

await page.screenshot({ path: '/tmp/shot-editor2.png', fullPage: true });
await browser.close();
