import { chromium } from 'playwright';

const base = 'https://82.24.174.152.nip.io';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 900, height: 700 } });

await page.goto(base + '/login', { waitUntil: 'networkidle' });
await page.waitForTimeout(1200);

const text = await page.locator('body').innerText();
console.log('есть WIKI_PASSWORD:', text.includes('WIKI_PASSWORD'));
console.log('есть подсказка:', text.includes('Пароль задаётся'));
console.log('заголовок:', text.split('\n').slice(0, 4).join(' | '));

await page.screenshot({ path: '/tmp/shot-live-login.png' });
await browser.close();
