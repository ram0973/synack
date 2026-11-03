<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Plus } from '@lucide/svelte';
	import { auth } from '$lib/auth.svelte';

	onMount(async () => {
		await auth.refresh();
		if (!auth.value) await goto('/login');
	});
</script>

<div class="flex flex-wrap items-start justify-between gap-3">
	<div>
		<h1 class="text-3xl font-semibold tracking-tight">Блог</h1>
		<p class="mt-1 text-sm text-muted-foreground">Посты и заметки по датам.</p>
	</div>
	<Button disabled title="скоро">
		<Plus class="size-4" />
		Новый пост
	</Button>
</div>

<Card.Root class="mt-6 border-dashed">
	<Card.Content class="flex flex-col items-center gap-3 py-16 text-center">
		<Badge variant="secondary">в разработке</Badge>
		<p class="max-w-sm text-sm text-muted-foreground">
			Раздел уже есть в меню, но постов пока не бывает. Здесь появится список статей блога.
		</p>
	</Card.Content>
</Card.Root>
