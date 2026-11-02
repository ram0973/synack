import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('response', (response) => {
	if (response.status() >= 400) {
		console.log('HTTP', response.status(), response.request().method(), response.url());
	}
});

await page.goto(base + '/admin', { waitUntil: 'networkidle' });
await page.waitForTimeout(500);

const items = await page.$$eval('[data-sidebar="menu-button"]', (nodes) =>
	nodes.map((node) => ({
		text: node.textContent.trim(),
		active: node.getAttribute('data-active'),
		bg: getComputedStyle(node).backgroundColor
	}))
);
console.log(JSON.stringify(items, null, 1));

const sidebarBg = await page.$eval('[data-slot="sidebar"]', (node) => getComputedStyle(node).backgroundColor);
console.log('sidebar bg:', sidebarBg);

await browser.close();
