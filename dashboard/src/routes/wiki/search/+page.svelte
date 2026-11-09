<script lang="ts">
import { wikiHref } from '$lib/paths';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Search } from '@lucide/svelte';
	import { api, type SearchHit } from '$lib/api';

	let query = $state('');
	let hits = $state<SearchHit[]>([]);
	let loading = $state(false);
	let error = $state('');

	$effect(() => {
		const value = page.url.searchParams.get('q') ?? '';
		query = value;

		if (!value.trim()) {
			hits = [];
			return;
		}

		void run(value);
	});

	async function run(value: string) {
		loading = true;
		try {
			hits = await api.search(value);
			error = '';
		} catch (e) {
			error = e instanceof Error ? e.message : 'ошибка поиска';
			hits = [];
		} finally {
			loading = false;
		}
	}

	function submit(event: SubmitEvent) {
		event.preventDefault();
		const value = query.trim();
		if (value) {
			goto('/wiki/search?q=' + encodeURIComponent(value));
		}
	}

	function segments(snippet: string) {
		const parts: { text: string; mark: boolean }[] = [];
		const regex = /\[\[(.*?)\]\]/g;
		let last = 0;
		let match: RegExpExecArray | null;

		while ((match = regex.exec(snippet))) {
			if (match.index > last) {
				parts.push({ text: snippet.slice(last, match.index), mark: false });
			}
			parts.push({ text: match[1], mark: true });
			last = match.index + match[0].length;
		}

		if (last < snippet.length) {
			parts.push({ text: snippet.slice(last), mark: false });
		}

		return parts;
	}
</script>

<div>
	<h1 class="text-2xl font-semibold tracking-tight">Поиск</h1>
	<p class="mt-1 text-sm text-muted-foreground">
		Ищет по заголовкам и тексту статей. Работает и по началу слова: «табл» найдёт «таблицы».
	</p>

	<form onsubmit={submit} class="mt-6">
		<div class="relative">
			<Search class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
			<Input class="h-10 pl-9" placeholder="Что искать?" bind:value={query} autofocus />
		</div>
	</form>

	{#if error}
		<div class="mt-6 rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-sm text-destructive">
			{error}
		</div>
	{/if}

	{#if loading}
		<p class="mt-6 text-sm text-muted-foreground">Ищу…</p>
	{:else if hits.length}
		<p class="mt-6 text-xs text-muted-foreground">Найдено: {hits.length}</p>
		<div class="mt-3 space-y-2">
			{#each hits as hit (hit.id)}
				<a href={wikiHref(hit.path)} class="group block no-underline">
					<Card.Root class="transition-colors hover:bg-accent/50">
						<Card.Content class="space-y-1.5 py-4">
							<div class="flex items-center gap-2">
								<span class="link-title text-sm font-medium">
									{hit.title}
								</span>
							</div>
							<div class="text-xs text-muted-foreground">{hit.path}</div>
							<p class="text-[13px] leading-relaxed text-muted-foreground">
								{#each segments(hit.snippet) as part}
									{#if part.mark}<mark>{part.text}</mark>{:else}{part.text}{/if}
								{/each}
							</p>
						</Card.Content>
					</Card.Root>
				</a>
			{/each}
		</div>
	{:else if query.trim()}
		<p class="mt-6 text-sm text-muted-foreground">Ничего не найдено.</p>
	{/if}
</div>
