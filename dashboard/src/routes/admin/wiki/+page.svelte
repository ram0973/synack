<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import * as Card from '$lib/components/ui/card/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { ArrowDown, ArrowUp, Ellipsis, FilePlus, Folder, Pencil, Plus, Trash2 } from '@lucide/svelte';
	import { api, treeRows, type CategoryNode, type PageSummary } from '$lib/api';
	import { auth } from '$lib/auth.svelte';

	let tree = $state<CategoryNode[]>([]);
	let pages = $state<PageSummary[]>([]);
	let error = $state('');
	let loading = $state(true);

	let dialogOpen = $state(false);
	let dialogMode = $state<'create' | 'rename'>('create');
	let dialogParent = $state<number | null>(null);
	let dialogTarget = $state<CategoryNode | null>(null);
	let draftName = $state('');

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

	async function run(action: () => Promise<unknown>, success: string) {
		try {
			await action();
			await reload();
			toast.success(success);
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'ошибка');
		}
	}

	function openCreate(parentId: number | null) {
		dialogMode = 'create';
		dialogParent = parentId;
		dialogTarget = null;
		draftName = '';
		dialogOpen = true;
	}

	function openRename(node: CategoryNode) {
		dialogMode = 'rename';
		dialogTarget = node;
		draftName = node.name;
		dialogOpen = true;
	}

	function confirmDialog() {
		const name = draftName.trim();
		if (!name) return;

		if (dialogMode === 'create') {
			run(() => api.createCategory({ parent_id: dialogParent, name }), 'Категория создана');
		} else if (dialogTarget) {
			const target = dialogTarget;
			run(() => api.updateCategory(target.id, { name }), 'Категория переименована');
		}

		dialogOpen = false;
	}

	function remove(node: CategoryNode) {
		if (!window.confirm('Удалить категорию «' + node.name + '»?')) return;
		run(() => api.deleteCategory(node.id), 'Категория удалена');
	}

	function move(node: CategoryNode, direction: -1 | 1) {
		const siblings = rows
			.filter((row) => row.node.parent_id === node.parent_id)
			.map((row) => row.node);
		const index = siblings.findIndex((item) => item.id === node.id);
		const target = index + direction;
		if (index < 0 || target < 0 || target >= siblings.length) return;

		const a = siblings[index];
		const b = siblings[target];

		run(async () => {
			await api.moveCategory(a.id, { parent_id: a.parent_id, position: target });
			await api.moveCategory(b.id, { parent_id: b.parent_id, position: index });
		}, 'Порядок обновлён');
	}

	function newPage() {
		const first = rows[0]?.node.id;
		goto(first ? '/admin/wiki/page/new?category=' + first : '/admin/wiki/page/new');
	}
</script>

<div class="flex items-start justify-between gap-4">
	<div>
		<h1 class="text-3xl font-semibold tracking-tight">Категории</h1>
	</div>
	<div class="flex items-center gap-2">
		<Button variant="outline" onclick={newPage}>
			<Plus class="size-4" />
			Новая страница
		</Button>
		<Button onclick={() => openCreate(null)}>
			<Plus class="size-4" />
			Новая категория
		</Button>
	</div>
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
				<Skeleton class="h-6 w-full" />
				<Skeleton class="h-6 w-5/6" />
				<Skeleton class="h-6 w-4/6" />
			</div>
		{:else}
			{#each rows as row (row.node.id)}
				<div
					class="flex items-center gap-2 border-b px-3 py-1.5 last:border-b-0"
					style={'padding-left: ' + (12 + row.depth * 18) + 'px'}
				>
					<Folder class="size-4 shrink-0 text-muted-foreground" />
					<span class="flex-1 truncate text-sm">{row.node.name}</span>
					<code class="hidden text-[11px] text-muted-foreground sm:inline">/{row.path}</code>

					<DropdownMenu.Root>
						<DropdownMenu.Trigger>
							{#snippet child({ props })}
								<Button variant="ghost" size="icon" class="size-7" {...props}>
									<Ellipsis class="size-4" />
								</Button>
							{/snippet}
						</DropdownMenu.Trigger>
						<DropdownMenu.Content align="end" class="w-48">
							<DropdownMenu.Item onclick={() => openRename(row.node)}>
								<Pencil class="size-4" />
								Переименовать
							</DropdownMenu.Item>
							<DropdownMenu.Item onclick={() => openCreate(row.node.id)}>
								<Plus class="size-4" />
								Подкатегория
							</DropdownMenu.Item>
							<DropdownMenu.Item
								onclick={() => goto('/admin/wiki/page/new?category=' + row.node.id)}
							>
								<FilePlus class="size-4" />
								Новая страница
							</DropdownMenu.Item>
							<DropdownMenu.Separator />
							<DropdownMenu.Item onclick={() => move(row.node, -1)}>
								<ArrowUp class="size-4" />
								Выше
							</DropdownMenu.Item>
							<DropdownMenu.Item onclick={() => move(row.node, 1)}>
								<ArrowDown class="size-4" />
								Ниже
							</DropdownMenu.Item>
							<DropdownMenu.Separator />
							<DropdownMenu.Item variant="destructive" onclick={() => remove(row.node)}>
								<Trash2 class="size-4" />
								Удалить
							</DropdownMenu.Item>
						</DropdownMenu.Content>
					</DropdownMenu.Root>
				</div>
			{/each}
			{#if !rows.length}
				<p class="px-4 py-8 text-center text-sm text-muted-foreground">Категорий пока нет.</p>
			{/if}
		{/if}
	</Card.Content>
</Card.Root>

<div class="mt-8 flex items-center justify-between">
	<h2 class="text-lg font-semibold tracking-tight">Страницы</h2>
	<span class="text-xs text-muted-foreground">{pages.length}</span>
</div>

<Card.Root class="mt-3 overflow-hidden py-0">
	<Card.Content class="p-0">
		{#each pages as item (item.id)}
			<div class="flex items-center gap-3 border-b px-4 py-2 last:border-b-0">
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
			<p class="px-4 py-8 text-center text-sm text-muted-foreground">Страниц пока нет.</p>
		{/if}
	</Card.Content>
</Card.Root>

<Dialog.Root bind:open={dialogOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>{dialogMode === 'create' ? 'Новая категория' : 'Переименовать'}</Dialog.Title>
			<Dialog.Description>
				Название видно в дереве. Slug и URL сгенерируются автоматически.
			</Dialog.Description>
		</Dialog.Header>

		<Input
			bind:value={draftName}
			placeholder="Название"
			onkeydown={(event: KeyboardEvent) => {
				if (event.key === 'Enter') confirmDialog();
			}}
		/>

		<Dialog.Footer>
			<Button variant="outline" onclick={() => (dialogOpen = false)}>Отмена</Button>
			<Button onclick={confirmDialog}>{dialogMode === 'create' ? 'Создать' : 'Сохранить'}</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
