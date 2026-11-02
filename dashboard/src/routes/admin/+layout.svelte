<script lang="ts">
	import { page } from '$app/state';

	let { children } = $props();

	const tabs = [
		{ href: '/admin/wiki', label: 'Вики', ready: true },
		{ href: '/admin/blog', label: 'Блог', ready: false },
		{ href: '/admin/pages', label: 'Страницы', ready: false }
	];

	const current = $derived(page.url.pathname);

	function tabClass(href: string, ready: boolean) {
		if (!ready) {
			return 'rounded-md px-3 py-1 text-sm text-muted-foreground/50';
		}

		return current.startsWith(href)
			? 'rounded-md bg-background px-3 py-1 text-sm font-medium text-foreground shadow-sm'
			: 'rounded-md px-3 py-1 text-sm text-muted-foreground transition-colors hover:text-foreground';
	}
</script>

<div class="inline-flex items-center rounded-lg border border-border bg-muted/50 p-0.5">
	{#each tabs as tab (tab.href)}
		{#if tab.ready}
			<a href={tab.href} class={tabClass(tab.href, true)}>{tab.label}</a>
		{:else}
			<span class={tabClass(tab.href, false)} title="скоро">{tab.label}</span>
		{/if}
	{/each}
</div>

{@render children()}
