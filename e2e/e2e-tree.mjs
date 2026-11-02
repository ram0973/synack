import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));

const asyncButton = () => page.getByRole('button', { name: 'Async', exact: true });
const expand = () => page.getByRole('button', { name: 'Развернуть' }).first();
const collapse = () => page.getByRole('button', { name: 'Свернуть' }).first();

await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.waitForTimeout(800);

console.log('подкатегория видна сразу:', await asyncButton().count());
await page.screenshot({ path: '/tmp/shot-tree-collapsed.png' });

await expand().click();
await page.waitForTimeout(500);
console.log('после клика по шеврону:', await asyncButton().count());
await page.screenshot({ path: '/tmp/shot-tree-expanded.png' });

await collapse().click();
await page.waitForTimeout(500);
console.log('после повторного клика:', await asyncButton().count());

await page.goto(base + '/rust/async/vvedenie', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);
console.log('на странице подкатегории ветка раскрыта:', await asyncButton().count());

await browser.close();
