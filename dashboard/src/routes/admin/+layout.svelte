<script lang="ts">
	import { page } from '$app/state';

	let { children } = $props();

	// Сюда добавляются новые приложения: блог, страницы и т.д.
	const apps = [{ href: '/admin/wiki', label: 'Вики' }];

	const current = $derived(page.url.pathname);

	const sections = $derived(
		current.startsWith('/admin/wiki')
			? [
					{ href: '/admin/wiki', label: 'Категории', match: (path: string) => path === '/admin/wiki' },
					{
						href: '/admin/wiki/pages',
						label: 'Страницы',
						match: (path: string) => path.startsWith('/admin/wiki/pages') || path.startsWith('/admin/wiki/page/')
					}
				]
			: []
	);

	function tabClass(active: boolean) {
		return active
			? 'rounded-md bg-background px-3 py-1 text-sm font-medium text-foreground shadow-sm'
			: 'rounded-md px-3 py-1 text-sm text-muted-foreground transition-colors hover:text-foreground';
	}
</script>

<div class="mb-7 flex flex-wrap items-center gap-3">
	<div class="inline-flex items-center rounded-lg border border-border bg-muted/50 p-0.5">
		{#each apps as app (app.href)}
			<a href={app.href} class={tabClass(current.startsWith(app.href))}>{app.label}</a>
		{/each}
	</div>

	<a
		href="/admin/settings"
		class="ml-auto {tabClass(current.startsWith('/admin/settings'))}"
	>
		Настройки
	</a>

	{#if sections.length}
		<nav class="flex items-center gap-1" aria-label="Разделы приложения">
			{#each sections as section (section.href)}
				<a href={section.href} class={tabClass(section.match(current))}>{section.label}</a>
			{/each}
		</nav>
	{/if}
</div>

{@render children()}
