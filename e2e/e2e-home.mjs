import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));

await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);

const list = page.locator('h2:text("Категории") + div a');
console.log('строк категорий:', await list.count());
console.log('тексты:', JSON.stringify(await list.allInnerTexts()));

await page.screenshot({ path: '/tmp/shot-home-categories.png' });
await browser.close();
