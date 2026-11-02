<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import Breadcrumbs from '$lib/components/breadcrumbs.svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { Pencil } from '@lucide/svelte';
	import { api, ApiError, type CategoryView, type PageOut } from '$lib/api';
	import { addHeadingAnchors, highlightWithin } from '$lib/highlight';

	let mode = $state<'loading' | 'page' | 'category' | 'missing'>('loading');
	let doc = $state<PageOut | null>(null);
	let category = $state<CategoryView | null>(null);
	let message = $state('');
	let contentEl = $state<HTMLDivElement | null>(null);

	$effect(() => {
		doc;
		highlightWithin(contentEl);
		addHeadingAnchors(contentEl);
	});

	async function load(path: string) {
		mode = 'loading';
		doc = null;
		category = null;
		message = '';

		try {
			const resolved = await api.resolve(path);
			if (resolved.kind === 'page') {
				doc = resolved.page;
				mode = 'page';
			} else {
				category = resolved.category;
				mode = 'category';
			}
		} catch (e) {
			message =
				e instanceof ApiError && e.status === 404
					? 'Страница не найдена'
					: e instanceof Error
						? e.message
						: 'ошибка';
			mode = 'missing';
		}
	}

	$effect(() => {
		load(page.params.path ?? '');
	});

	function plural(count: number, one: string, few: string, many: string) {
		const mod10 = count % 10;
		const mod100 = count % 100;
		if (mod10 === 1 && mod100 !== 11) return one;
		if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) return few;
		return many;
	}
</script>

{#if mode === 'loading'}
	<div class="space-y-4">
		<Skeleton class="h-4 w-64" />
		<Skeleton class="h-8 w-80" />
		<div class="space-y-2 pt-6">
			<Skeleton class="h-4 w-full" />
			<Skeleton class="h-4 w-11/12" />
			<Skeleton class="h-4 w-9/12" />
		</div>
	</div>
{:else if mode === 'page' && doc}
	<article class="max-w-3xl">
		<Breadcrumbs path={doc.path} current={doc.title} />

		<div class="mt-6 flex items-start justify-between gap-4">
			<div>
				<h1 class="text-2xl font-semibold tracking-tight">{doc.title}</h1>
				<div class="mt-2 flex items-center gap-2">
					<Badge variant={doc.status === 'published' ? 'default' : 'outline'}>
						{doc.status === 'published' ? 'опубликовано' : 'черновик'}
					</Badge>
					<span class="text-xs text-muted-foreground">{doc.path}</span>
				</div>
			</div>
			<Button variant="outline" onclick={() => goto('/admin/page/' + doc!.id)}>
				<Pencil class="size-4" />
				Редактировать
			</Button>
		</div>

		<div class="md mt-8" bind:this={contentEl}>{@html doc.html}</div>
	</article>
{:else if mode === 'category' && category}
	<div>
		<Breadcrumbs path={category.path} current={category.name} />

		<h1 class="mt-6 text-2xl font-semibold tracking-tight">{category.name}</h1>
		<p class="mt-1 text-xs text-muted-foreground">
			{category.children.length}
			{plural(category.children.length, 'подкатегория', 'подкатегории', 'подкатегорий')} ·
			{category.pages.length}
			{plural(category.pages.length, 'страница', 'страницы', 'страниц')}
		</p>

		{#if category.children.length}
			<div class="mt-6 grid gap-3 sm:grid-cols-2">
				{#each category.children as child (child.id)}
					<a href={category.path + '/' + child.slug} class="block no-underline">
						<Card.Root class="transition-colors hover:bg-accent/50">
							<Card.Header>
								<Card.Title class="text-base">{child.name}</Card.Title>
							</Card.Header>
						</Card.Root>
					</a>
				{/each}
			</div>
		{/if}

		{#if category.pages.length}
			<div class="mt-4 space-y-2">
				{#each category.pages as item (item.id)}
					<a href={category.path + '/' + item.slug} class="block no-underline">
						<Card.Root class="transition-colors hover:bg-accent/50">
							<Card.Content class="flex items-center justify-between py-3">
								<span class="text-sm font-medium">{item.title}</span>
								<Badge variant={item.status === 'published' ? 'default' : 'outline'}>
									{item.status === 'published' ? 'опубликовано' : 'черновик'}
								</Badge>
							</Card.Content>
						</Card.Root>
					</a>
				{/each}
			</div>
		{/if}

		{#if !category.children.length && !category.pages.length}
			<p class="mt-6 text-sm text-muted-foreground">Пусто.</p>
		{/if}
	</div>
{:else}
	<div>
		<h1 class="text-2xl font-semibold">404</h1>
		<p class="mt-2 text-sm text-muted-foreground">{message || 'Страница не найдена'}</p>
		<Button class="mt-6" variant="outline" onclick={() => goto('/')}>На главную</Button>
	</div>
{/if}
