/// Ссылка на страницу вики: принимает путь с ведущим слэшем или без.
export function wikiHref(path: string): string {
	const clean = path.replace(/^\/+/, '');
	return clean ? '/wiki/' + clean : '/wiki';
}
