import { chromium } from 'playwright';

const base = 'http://127.0.0.1:4173';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

page.on('pageerror', (error) => console.log('PAGEERROR', error.message));

const btn = (name) => page.getByRole('button', { name, exact: true });

await page.goto(base + '/', { waitUntil: 'networkidle' });
await page.waitForTimeout(900);

console.log('страницы скрыты:', await btn('Введение').count());
console.log('подкатегории скрыты:', await btn('Async').count());

await btn('Rust').click();
await page.waitForTimeout(600);
console.log('после клика по Rust: Async', await btn('Async').count(), '| Введение', await btn('Введение').count(), '| Основы', await btn('Основы').count());
await page.screenshot({ path: '/tmp/shot-nav-open.png' });

await btn('Заметки').click();
await page.waitForTimeout(600);
console.log('после клика по Заметки: Async', await btn('Async').count(), '| Идеи', await btn('Идеи').count());

await btn('Идеи').click();
await page.waitForTimeout(900);
console.log('страница открылась:', page.url());

await page.screenshot({ path: '/tmp/shot-nav-page.png' });
await browser.close();
