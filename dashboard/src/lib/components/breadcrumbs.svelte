<script lang="ts">
	let { path, current }: { path: string; current: string } = $props();

	const segments = $derived(path.split('/').filter(Boolean));
	const parents = $derived(segments.slice(0, -1));
</script>

<nav class="flex flex-wrap items-center gap-1.5 text-xs text-muted-foreground">
	<a href="/wiki" class="transition-colors hover:text-foreground">wiki</a>
	{#each parents as item, index (item)}
		<span class="opacity-40">/</span>
		<a
			href={'/wiki/' + parents.slice(0, index + 1).join('/')}
			class="transition-colors hover:text-foreground">{item}</a
		>
	{/each}
	<span class="opacity-40">/</span>
	<span class="text-foreground">{current}</span>
</nav>
