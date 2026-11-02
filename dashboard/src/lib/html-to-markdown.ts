import TurndownService from 'turndown';
import { gfm } from 'turndown-plugin-gfm';

const turndown = new TurndownService({
	headingStyle: 'atx',
	codeBlockStyle: 'fenced',
	bulletListMarker: '-',
	emDelimiter: '*'
});

turndown.use(gfm);

/// HTML из буфера обмена → Markdown.
export function htmlToMarkdown(html: string): string {
	return turndown.turndown(html).trim();
}

/// Стоит ли вообще конвертировать: обычный текст браузер тоже отдаёт как HTML.
export function looksLikeMarkup(html: string): boolean {
	return /<(h[1-6]|p|ul|ol|table|img|pre|blockquote|a)\b/i.test(html);
}
