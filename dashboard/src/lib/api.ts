export type Category = {
	id: number;
	parent_id: number | null;
	position: number;
	name: string;
	slug: string;
};

export type CategoryNode = Category & { children: CategoryNode[] };

export type PageSummary = {
	id: number;
	category_id: number;
	slug: string;
	title: string;
	status: string;
	updated_at: string;
};

export type PageOut = PageSummary & {
	body_md: string;
	html: string;
	path: string;
	created_at: string;
};

export type CategoryView = Category & {
	path: string;
	children: Category[];
	pages: PageSummary[];
};

export type Resolved =
	| { kind: 'page'; page: PageOut }
	| { kind: 'category'; category: CategoryView };

export type SearchHit = {
	id: number;
	title: string;
	path: string;
	status: string;
	snippet: string;
};

export type SiteSettings = {
	author_name: string;
	author_avatar: string;
};

export type RevisionSummary = {
	id: number;
	title: string;
	created_at: string;
	size: number;
	preview: string;
};

export class ApiError extends Error {
	status: number;

	constructor(status: number, message: string) {
		super(message);
		this.status = status;
	}
}

async function request<T>(url: string, init?: RequestInit): Promise<T> {
	const res = await fetch(url, {
		...init,
		headers: { 'Content-Type': 'application/json', ...(init?.headers ?? {}) }
	});

	if (res.status === 204) return undefined as T;

	if (!res.ok) {
		let message = res.statusText;
		try {
			const body = await res.json();
			message = body.error ?? message;
		} catch {
			/* ignore */
		}
		throw new ApiError(res.status, message);
	}

	return (await res.json()) as T;
}

export const api = {
	tree: () => request<CategoryNode[]>('/api/wiki/tree'),
	pages: (category?: number) =>
		request<PageSummary[]>('/api/wiki/pages' + (category ? '?category=' + category : '')),
	page: (id: number) => request<PageOut>('/api/wiki/pages/' + id),
	pageByPath: (path: string) =>
		request<PageOut>('/api/wiki/pages/by-path?path=' + encodeURIComponent(path)),
	resolve: (path: string) =>
		request<Resolved>('/api/wiki/resolve?path=' + encodeURIComponent(path)),
	categoryByPath: (path: string) =>
		request<CategoryView>('/api/wiki/categories/by-path?path=' + encodeURIComponent(path)),
	createPage: (data: unknown) =>
		request<PageOut>('/api/wiki/pages', { method: 'POST', body: JSON.stringify(data) }),
	updatePage: (id: number, data: unknown) =>
		request<PageOut>('/api/wiki/pages/' + id, { method: 'PATCH', body: JSON.stringify(data) }),
	deletePage: (id: number) => request<void>('/api/wiki/pages/' + id, { method: 'DELETE' }),
	createCategory: (data: unknown) =>
		request<Category>('/api/wiki/categories', { method: 'POST', body: JSON.stringify(data) }),
	updateCategory: (id: number, data: unknown) =>
		request<Category>('/api/wiki/categories/' + id, { method: 'PATCH', body: JSON.stringify(data) }),
	moveCategory: (id: number, data: unknown) =>
		request<Category>('/api/wiki/categories/' + id + '/move', {
			method: 'POST',
			body: JSON.stringify(data)
		}),
	deleteCategory: (id: number) => request<void>('/api/wiki/categories/' + id, { method: 'DELETE' }),
	preview: (body_md: string) =>
		request<{ html: string }>('/api/wiki/preview', {
			method: 'POST',
			body: JSON.stringify({ body_md })
		}),
	me: () => request<{ authenticated: boolean; default_password: boolean }>('/api/auth/me'),
	settings: () => request<SiteSettings>('/api/settings'),
	updateSettings: (body: Partial<SiteSettings>) =>
		request<SiteSettings>('/api/settings', { method: 'PUT', body: JSON.stringify(body) }),
	search: (q: string) => request<SearchHit[]>('/api/wiki/search?q=' + encodeURIComponent(q)),
	revisions: (pageId: number) =>
		request<RevisionSummary[]>('/api/wiki/pages/' + pageId + '/revisions'),
	restoreRevision: (revisionId: number) =>
		request<PageOut>('/api/wiki/revisions/' + revisionId + '/restore', { method: 'POST' }),
	uploadMedia: async (file: File) => {
		const form = new FormData();
		form.append('file', file);

		const response = await fetch('/api/media', { method: 'POST', body: form });

		if (!response.ok) {
			const body = await response.json().catch(() => ({ error: response.statusText }));
			throw new ApiError(response.status, body.error ?? response.statusText);
		}

		return (await response.json()) as { url: string; name: string; size: number };
	},
	login: (password: string) =>
		request<{ ok: boolean }>('/api/auth/login', {
			method: 'POST',
			body: JSON.stringify({ password })
		}),
	logout: () => request<{ ok: boolean }>('/api/auth/logout', { method: 'POST' }),
	changePassword: (current: string, next: string) =>
		request<{ ok: boolean }>('/api/auth/password', {
			method: 'POST',
			body: JSON.stringify({ current, next })
		})
};

export type TreeRow = { node: CategoryNode; depth: number; path: string };

export function treeRows(
	nodes: CategoryNode[],
	prefix = '',
	depth = 0,
	out: TreeRow[] = []
): TreeRow[] {
	for (const node of nodes) {
		const path = prefix ? prefix + '/' + node.slug : node.slug;
		out.push({ node, depth, path });
		if (node.children && node.children.length) {
			treeRows(node.children, path, depth + 1, out);
		}
	}
	return out;
}
