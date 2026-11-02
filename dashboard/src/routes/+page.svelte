<script lang="ts">
	import { goto } from '$app/navigation';
	import { BookOpen, FileText, NotebookPen } from '@lucide/svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
</script>

<div class="flex flex-wrap items-center justify-between gap-3">
	<div>
		<h1 class="text-2xl font-semibold tracking-tight">Главная</h1>
		<p class="mt-1 text-sm text-muted-foreground">Разделы сайта.</p>
	</div>
</div>

<div class="mt-8 grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
	<Card.Root
		class="cursor-pointer transition-shadow hover:shadow-md"
		role="link"
		tabindex={0}
		onclick={() => goto('/wiki')}
		onkeydown={(event: KeyboardEvent) => {
			if (event.key === 'Enter') goto('/wiki');
		}}
	>
		<Card.Content class="flex h-full flex-col gap-3 py-5">
			<div class="flex items-center gap-2">
				<BookOpen class="size-4 text-muted-foreground" />
				<span class="font-medium">Вики</span>
			</div>
			<p class="text-sm text-muted-foreground">
				Статьи, категории и поиск по ним. Markdown, картинки, видео, код.
			</p>
		</Card.Content>
	</Card.Root>

	<Card.Root class="opacity-60">
		<Card.Content class="flex h-full flex-col gap-3 py-5">
			<div class="flex items-center gap-2">
				<NotebookPen class="size-4 text-muted-foreground" />
				<span class="font-medium">Блог</span>
				<Badge variant="secondary" class="ml-auto">скоро</Badge>
			</div>
			<p class="text-sm text-muted-foreground">Заметки и статьи по датам.</p>
		</Card.Content>
	</Card.Root>

	<Card.Root class="opacity-60">
		<Card.Content class="flex h-full flex-col gap-3 py-5">
			<div class="flex items-center gap-2">
				<FileText class="size-4 text-muted-foreground" />
				<span class="font-medium">Страницы</span>
				<Badge variant="secondary" class="ml-auto">скоро</Badge>
			</div>
			<p class="text-sm text-muted-foreground">Отдельные страницы: о проекте, контакты.</p>
		</Card.Content>
	</Card.Root>
</div>
