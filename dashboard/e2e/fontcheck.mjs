import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.waitForTimeout(800);

const info = await page.evaluate(() => ({
	fontFamily: getComputedStyle(document.body).fontFamily,
	checkInter: document.fonts.check('16px "Inter Variable"'),
	faces: [...document.fonts].map((f) => f.family + ':' + f.status).slice(0, 8)
}));
console.log(JSON.stringify(info, null, 1));

const tree = await page.evaluate(async () => {
	const res = await fetch('/api/tree');
	return res.json();
});
console.log('tree order:', tree.map((n) => n.name + '#' + n.position + '(' + n.id + ')').join(', '));

await browser.close();
