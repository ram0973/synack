<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { Plus } from '@lucide/svelte';
	import { api, treeRows, type CategoryNode, type PageSummary } from '$lib/api';
	import { auth } from '$lib/auth.svelte';

	let tree = $state<CategoryNode[]>([]);
	let pages = $state<PageSummary[]>([]);
	let error = $state('');
	let loading = $state(true);

	async function reload() {
		try {
			const [t, p] = await Promise.all([api.tree(), api.pages()]);
			tree = t;
			pages = p;
			error = '';
		} catch (e) {
			error = e instanceof Error ? e.message : 'ошибка';
		} finally {
			loading = false;
		}
	}

	onMount(async () => {
		await auth.refresh();
		if (!auth.value) {
			await goto('/login');
			return;
		}
		await reload();
	});

	const rows = $derived(treeRows(tree));

	function newPage() {
		const first = rows[0]?.node.id;
		goto(first ? '/admin/wiki/page/new?category=' + first : '/admin/wiki/page/new');
	}
</script>

<div class="flex items-start justify-between gap-4">
	<div>
		<h1 class="text-3xl font-semibold tracking-tight">Страницы</h1>
		<p class="mt-1 text-sm text-muted-foreground">Все статьи вики, включая черновики.</p>
	</div>
	<Button onclick={newPage}>
		<Plus class="size-4" />
		Новая страница
	</Button>
</div>

{#if error}
	<div class="mt-6 rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-sm text-destructive">
		{error}
	</div>
{/if}

<Card.Root class="mt-6 overflow-hidden py-0">
	<Card.Content class="p-0">
		{#if loading}
			<div class="space-y-2 p-4">
				<Skeleton class="h-8 w-full" />
				<Skeleton class="h-8 w-5/6" />
				<Skeleton class="h-8 w-4/6" />
			</div>
		{:else}
			{#each pages as item (item.id)}
				<div class="flex items-center gap-3 border-b px-4 py-2.5 last:border-b-0">
					<span class="flex-1 truncate text-sm">{item.title}</span>
					<Badge variant={item.status === 'published' ? 'default' : 'outline'}>
						{item.status === 'published' ? 'опубликовано' : 'черновик'}
					</Badge>
					<Button size="sm" variant="outline" onclick={() => goto('/admin/wiki/page/' + item.id)}>
						Открыть
					</Button>
				</div>
			{/each}
			{#if !pages.length}
				<p class="px-4 py-10 text-center text-sm text-muted-foreground">Страниц пока нет.</p>
			{/if}
		{/if}
	</Card.Content>
</Card.Root>
