<script lang="ts">
import { wikiHref } from '$lib/paths';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { Folder, Plus } from '@lucide/svelte';
	import { api, treeRows, type CategoryNode, type PageSummary } from '$lib/api';

	let tree = $state<CategoryNode[]>([]);
	let pages = $state<PageSummary[]>([]);
	let error = $state('');
	let loading = $state(true);

	onMount(async () => {
		try {
			const [t, p] = await Promise.all([api.tree(), api.pages()]);
			tree = t;
			pages = p;
		} catch (e) {
			error = e instanceof Error ? e.message : 'ошибка';
		} finally {
			loading = false;
		}
	});

	const rows = $derived(treeRows(tree));
	const catPaths = $derived(new Map(rows.map((row) => [row.node.id, row.path])));

	function href(item: PageSummary) {
		const cat = catPaths.get(item.category_id);
		return cat ? wikiHref(cat + '/' + item.slug) : '/wiki';
	}

	function newPage() {
		const first = rows[0]?.node.id;
		goto(first ? '/admin/wiki/page/new?category=' + first : '/admin/wiki/page/new');
	}
</script>

<div class="flex items-start justify-between gap-4">
	<div>
		<h1 class="text-2xl font-semibold tracking-tight">Главная</h1>
		<p class="mt-1 text-sm text-muted-foreground">Все страницы вики.</p>
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

{#if loading}
	<div class="mt-8 space-y-2">
		{#each [1, 2, 3] as item (item)}
			<Skeleton class="h-9 w-full rounded-lg" />
		{/each}
	</div>
{:else}
	{#if rows.length}
		<h2 class="mt-10 text-lg font-semibold tracking-tight">Категории</h2>

		<div class="mt-3 overflow-hidden rounded-xl border border-border bg-white">
			{#each rows as row (row.node.id)}
				<a
					href={wikiHref(row.path)}
					class="flex items-center border-b px-4 py-2 text-inherit no-underline transition-colors last:border-b-0 hover:bg-accent/50"
				>
					<span
						class="flex min-w-0 items-center gap-2 text-sm"
						style={'padding-left: ' + row.depth * 16 + 'px'}
					>
						<Folder class="size-4 shrink-0 text-muted-foreground" />
						<span class="truncate">{row.node.name}</span>
					</span>
				</a>
			{/each}
		</div>
	{/if}

	<h2 class="mt-10 text-lg font-semibold tracking-tight">Страницы</h2>

	<div class="mt-3 grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
		{#each pages as item (item.id)}
			<a href={href(item)} class="block no-underline">
				<Card.Root class="h-full transition-colors hover:bg-accent/50">
					<Card.Header class="gap-1.5">
						<Card.Title class="text-base">{item.title}</Card.Title>
						<Card.Description class="text-xs">{href(item)}</Card.Description>
					</Card.Header>
					<Card.Content class="pt-0">
						<Badge variant={item.status === 'published' ? 'default' : 'outline'}>
							{item.status === 'published' ? 'опубликовано' : 'черновик'}
						</Badge>
					</Card.Content>
				</Card.Root>
			</a>
		{/each}
	</div>

	{#if !pages.length && !error}
		<Card.Root class="mt-3 border-dashed">
			<Card.Content class="flex flex-col items-center gap-3 py-12 text-center">
				<p class="text-sm text-muted-foreground">Страниц пока нет.</p>
				<Button variant="outline" onclick={() => goto('/admin/wiki')}>Создать в админке</Button>
			</Card.Content>
		</Card.Root>
	{/if}
{/if}
