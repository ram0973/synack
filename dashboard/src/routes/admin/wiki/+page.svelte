<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import * as Card from '$lib/components/ui/card/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import {
		ArrowDown,
		ArrowUp,
		Ellipsis,
		FilePlus,
		Folder,
		GripVertical,
		Pencil,
		Plus,
		Trash2
	} from '@lucide/svelte';
	import { api, treeRows, type CategoryNode } from '$lib/api';
	import { auth } from '$lib/auth.svelte';

	let tree = $state<CategoryNode[]>([]);
	let error = $state('');
	let loading = $state(true);

	let dialogOpen = $state(false);
	let dialogMode = $state<'create' | 'rename'>('create');
	let dialogParent = $state<number | null>(null);
	let dialogTarget = $state<CategoryNode | null>(null);
	let draftName = $state('');

	async function reload() {
		try {
			tree = await api.tree();
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

	type DropMode = 'before' | 'inside' | 'after';

	let dragId = $state<number | null>(null);
	let dropTarget = $state<{ id: number; mode: DropMode } | null>(null);

	function startDrag(event: PointerEvent, node: CategoryNode) {
		if (event.button !== 0) return;

		event.preventDefault();
		dragId = node.id;
		dropTarget = null;

		function cleanup() {
			window.removeEventListener('pointermove', onMove);
			window.removeEventListener('pointerup', onUp);
			window.removeEventListener('pointercancel', onUp);
		}

		function onMove(moveEvent: PointerEvent) {
			if (dragId === null) return;

			const element = document.elementFromPoint(moveEvent.clientX, moveEvent.clientY);
			const rowElement = element?.closest('[data-row-id]') as HTMLElement | null;

			if (!rowElement) {
				dropTarget = null;
				return;
			}

			const id = Number(rowElement.dataset.rowId);
			if (id === dragId) {
				dropTarget = null;
				return;
			}

			const rect = rowElement.getBoundingClientRect();
			const ratio = (moveEvent.clientY - rect.top) / rect.height;
			const mode: DropMode = ratio < 0.3 ? 'before' : ratio > 0.7 ? 'after' : 'inside';

			dropTarget = { id, mode };
		}

		async function onUp() {
			cleanup();

			const id = dragId;
			const target = dropTarget;

			dragId = null;
			dropTarget = null;

			if (id === null || !target) return;
			await applyMove(id, target);
		}

		window.addEventListener('pointermove', onMove);
		window.addEventListener('pointerup', onUp);
		window.addEventListener('pointercancel', onUp);
	}

	async function applyMove(id: number, target: { id: number; mode: DropMode }) {
		const dragged = rows.find((row) => row.node.id === id)?.node;
		const targetNode = rows.find((row) => row.node.id === target.id)?.node;

		if (!dragged || !targetNode) return;

		let parentId: number | null;
		let position: number;

		if (target.mode === 'inside') {
			parentId = targetNode.id;
			position = rows.filter((row) => row.node.parent_id === targetNode.id).length;
		} else {
			parentId = targetNode.parent_id;
			const siblings = rows
				.filter((row) => row.node.parent_id === parentId && row.node.id !== id)
				.sort((a, b) => a.node.position - b.node.position);
			const index = siblings.findIndex((row) => row.node.id === target.id);
			position = target.mode === 'before' ? index : index + 1;
		}

		const currentSiblings = rows
			.filter((row) => row.node.parent_id === parentId)
			.sort((a, b) => a.node.position - b.node.position);
		const currentIndex = currentSiblings.findIndex((row) => row.node.id === id);

		if (dragged.parent_id === parentId && currentIndex === position) return;

		try {
			await api.moveCategory(id, { parent_id: parentId, position });
			await reload();
			toast.success('Перемещено');
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'не удалось переместить');
			await reload();
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

</script>

<div class="flex items-start justify-between gap-4">
	<div>
		<h1 class="text-3xl font-semibold tracking-tight">Категории</h1>
	</div>
	<div class="flex items-center gap-2">
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
					data-row-id={row.node.id}
					class="flex items-center gap-2 border-b px-3 py-1.5 last:border-b-0 {dragId === row.node.id
						? 'opacity-40'
						: ''} {dropTarget?.id === row.node.id && dropTarget.mode === 'before'
						? 'border-t-2 border-t-primary'
						: ''} {dropTarget?.id === row.node.id && dropTarget.mode === 'after'
						? 'border-b-2 border-b-primary'
						: ''} {dropTarget?.id === row.node.id && dropTarget.mode === 'inside'
						? 'bg-accent ring-1 ring-inset ring-primary/40'
						: ''}"
					style={'padding-left: ' + (12 + row.depth * 18) + 'px'}
				>
					<button
						type="button"
						class="cursor-grab touch-none rounded p-0.5 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground active:cursor-grabbing"
						title="Перетащить"
						aria-label="Перетащить категорию"
						onpointerdown={(event) => startDrag(event, row.node)}
					>
						<GripVertical class="size-4" />
					</button>
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
