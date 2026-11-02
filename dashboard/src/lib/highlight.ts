import hljs from 'highlight.js/lib/common';

/// Подсвечивает все ещё не обработанные блоки кода внутри контейнера.
export function highlightWithin(root: HTMLElement | null) {
	if (!root) return;

	root.querySelectorAll<HTMLElement>('pre code').forEach((block) => {
		if (!block.dataset.highlighted) {
			hljs.highlightElement(block);
		}
	});
}

const LINK_ICON =
	'<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>';

/// Добавляет к заголовкам с id ссылку-якорь.
export function addHeadingAnchors(root: HTMLElement | null) {
	if (!root) return;

	root.querySelectorAll<HTMLElement>('h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]').forEach(
		(heading) => {
			if (heading.querySelector('.heading-anchor')) return;

			const link = document.createElement('a');
			link.href = '#' + heading.id;
			link.className = 'heading-anchor';
			link.title = 'Ссылка на раздел';
			link.setAttribute('aria-label', 'Ссылка на раздел');
			link.innerHTML = LINK_ICON;

			heading.append(link);
		}
	);
}
