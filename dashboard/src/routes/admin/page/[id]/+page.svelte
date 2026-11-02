<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { toast } from 'svelte-sonner';
	import * as Card from '$lib/components/ui/card/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import CodeEditor from '$lib/components/code-editor.svelte';
	import { addHeadingAnchors, highlightWithin } from '$lib/highlight';
	import {
		ArrowLeft,
		History,
		ImagePlus,
		Link as LinkIcon,
		MessageSquare,
		Redo2,
		Table,
		Trash2,
		Undo2,
		Upload,
		Video
	} from '@lucide/svelte';
	import { api, treeRows, type CategoryNode, type RevisionSummary } from '$lib/api';
	import { auth } from '$lib/auth.svelte';

	const id = $derived(page.params.id ?? 'new');
	const isNew = $derived(id === 'new');

	let tree = $state<CategoryNode[]>([]);
	let title = $state('');
	let slug = $state('');
	let categoryId = $state<number | null>(null);
	let status = $state('draft');
	let body = $state('');
	let preview = $state('');
	let error = $state('');
	let saving = $state(false);
	let loading = $state(true);
	let tab = $state('edit');
	let timer: ReturnType<typeof setTimeout> | undefined;
	let editorEl = $state<{
		insert: (text: string) => void;
		undo: () => void;
		redo: () => void;
	} | null>(null);
	let fileInput = $state<HTMLInputElement | null>(null);
	let historyOpen = $state(false);
	let historyLoading = $state(false);
	let revisions = $state<RevisionSummary[]>([]);
	let videoOpen = $state(false);
	let videoUrl = $state('');
	let videoInput = $state<HTMLInputElement | null>(null);
	let tableOpen = $state(false);
	let tableCols = $state(3);
	let tableRows = $state(2);
	let linkOpen = $state(false);
	let linkText = $state('');
	let linkUrl = $state('');
	let calloutOpen = $state(false);
	let calloutType = $state('note');
	let calloutText = $state('');
	let previewEl = $state<HTMLDivElement | null>(null);
	let draftTimer: ReturnType<typeof setTimeout> | undefined;
	let serverSnapshot = {
		title: '',
		slug: '',
		categoryId: null as number | null,
		status: 'draft',
		body: ''
	};

	const draftKey = $derived('wiki-draft:' + id);

	function restoreDraft() {
		try {
			const raw = localStorage.getItem(draftKey);
			if (!raw) return;

			const draft = JSON.parse(raw) as {
				title?: string;
				slug?: string;
				categoryId?: number | null;
				status?: string;
				body?: string;
			};

			const changed =
				(draft.title ?? '') !== title ||
				(draft.slug ?? '') !== slug ||
				(draft.body ?? '') !== body ||
				(draft.categoryId ?? null) !== categoryId;

			if (!changed) return;

			title = draft.title ?? title;
			slug = draft.slug ?? slug;
			categoryId = draft.categoryId ?? categoryId;
			status = draft.status ?? status;
			body = draft.body ?? body;

			toast.info('Восстановлен несохранённый черновик', {
				action: {
					label: 'Сбросить',
					onClick: () => {
						localStorage.removeItem(draftKey);
						title = serverSnapshot.title;
						slug = serverSnapshot.slug;
						categoryId = serverSnapshot.categoryId;
						status = serverSnapshot.status;
						body = serverSnapshot.body;
					}
				}
			});
		} catch {
			/* битый черновик просто игнорируем */
		}
	}

	$effect(() => {
		const payload = { title, slug, categoryId, status, body };

		if (loading) return;

		clearTimeout(draftTimer);
		draftTimer = setTimeout(() => {
			try {
				localStorage.setItem(draftKey, JSON.stringify(payload));
			} catch {
				/* нет места — не критично */
			}
		}, 600);

		return () => clearTimeout(draftTimer);
	});

	$effect(() => {
		preview;
		highlightWithin(previewEl);
		addHeadingAnchors(previewEl);
	});

	const rows = $derived(treeRows(tree));
	const words = $derived(body.trim() ? body.trim().split(/\s+/).length : 0);

	const selectClass =
		'h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50';

	onMount(async () => {
		await auth.refresh();
		if (!auth.value) {
			await goto('/login');
			return;
		}

		try {
			tree = await api.tree();
			if (isNew) {
				const fromQuery = page.url.searchParams.get('category');
				categoryId = fromQuery ? Number(fromQuery) : (treeRows(tree)[0]?.node.id ?? null);
			} else {
				const doc = await api.page(Number(id));
				title = doc.title;
				slug = doc.slug;
				categoryId = doc.category_id;
				status = doc.status;
				body = doc.body_md;
				preview = doc.html;
			}

			serverSnapshot = { title, slug, categoryId, status, body };
			restoreDraft();
		} catch (e) {
			error = e instanceof Error ? e.message : 'ошибка';
		} finally {
			loading = false;
		}
	});

	$effect(() => {
		const markdown = body;
		if (loading) return;
		clearTimeout(timer);
		timer = setTimeout(async () => {
			try {
				preview = (await api.preview(markdown)).html;
			} catch {
				/* превью не критично */
			}
		}, 250);
		return () => clearTimeout(timer);
	});

	async function uploadImage(file: File) {
		if (!file.type.startsWith('image/')) return;

		try {
			const media = await api.uploadMedia(file);
			const alt = file.name.replace(/\.[^.]+$/, '') || 'image';
			const snippet = '\n![' + alt + '](' + media.url + ')\n';
			editorEl?.insert(snippet);
			toast.success('Картинка загружена');
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'не загрузилось');
		}
	}

	function onFilePicked(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (file) uploadImage(file);
		input.value = '';
	}

	function onDrop(event: DragEvent) {
		event.preventDefault();
		const file = event.dataTransfer?.files?.[0];
		if (file) uploadImage(file);
	}

	function onPasteImage(event: ClipboardEvent) {
		const file = event.clipboardData?.files?.[0];
		if (file) {
			event.preventDefault();
			uploadImage(file);
		}
	}

	async function uploadVideo(file: File) {
		try {
			const media = await api.uploadMedia(file);
			editorEl?.insert('\n::video{src="' + media.url + '"}\n');
			videoOpen = false;
			toast.success('Видео загружено');
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'не загрузилось');
		}
	}

	function onVideoPicked(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (file) uploadVideo(file);
		input.value = '';
	}

	function youtubeId(value: string): string | null {
		const input = value.trim();
		if (!input) return null;

		const patterns = [
			/(?:youtube\.com\/watch\?v=)([\w-]{6,24})/,
			/(?:youtu\.be\/)([\w-]{6,24})/,
			/(?:youtube\.com\/embed\/)([\w-]{6,24})/,
			/^([\w-]{6,24})$/
		];

		for (const pattern of patterns) {
			const match = input.match(pattern);
			if (match) return match[1];
		}

		return null;
	}

	function insertYouTube() {
		const id = youtubeId(videoUrl);

		if (!id) {
			toast.error('Не похоже на ссылку YouTube');
			return;
		}

		editorEl?.insert('\n::youtube{id="' + id + '"}\n');
		videoUrl = '';
		videoOpen = false;
		toast.success('Видео вставлено');
	}

	function insertTable() {
		const cols = Math.min(Math.max(Math.round(tableCols) || 1, 1), 10);
		const rows = Math.min(Math.max(Math.round(tableRows) || 1, 1), 50);

		const header =
			'| ' + Array.from({ length: cols }, (_, index) => 'Столбец ' + (index + 1)).join(' | ') + ' |';
		const divider = '| ' + Array.from({ length: cols }, () => '---').join(' | ') + ' |';
		const body = Array.from(
			{ length: rows },
			() => '| ' + Array.from({ length: cols }, () => '   ').join(' | ') + ' |'
		);

		editorEl?.insert('\n' + [header, divider, ...body].join('\n') + '\n');
		tableOpen = false;
		toast.success('Таблица вставлена');
	}

	function insertLink() {
		const url = linkUrl.trim();

		if (!url) {
			toast.error('Укажите адрес ссылки');
			return;
		}

		if (!/^(https?:\/\/|mailto:|\/|#)/.test(url)) {
			toast.error('Адрес должен начинаться с http(s)://, mailto:, / или #');
			return;
		}

		const text = linkText.trim() || url;
		editorEl?.insert('[' + text + '](' + url + ')');
		linkOpen = false;
		linkText = '';
		linkUrl = '';
		toast.success('Ссылка вставлена');
	}

	function insertCallout() {
		const text = calloutText.trim() || 'Текст';
		editorEl?.insert('\n:::' + calloutType + '\n' + text + '\n:::\n');
		calloutOpen = false;
		calloutText = '';
		toast.success('Блок вставлен');
	}

	function tabClass(active: boolean) {
		return active
			? 'rounded-md bg-background px-3 py-1 text-sm font-medium text-foreground shadow-sm'
			: 'rounded-md px-3 py-1 text-sm text-muted-foreground transition-colors hover:text-foreground';
	}

	async function openHistory() {
		historyOpen = true;
		historyLoading = true;

		try {
			revisions = await api.revisions(Number(id));
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'не удалось загрузить историю');
			revisions = [];
		} finally {
			historyLoading = false;
		}
	}

	async function restore(revisionId: number) {
		if (!window.confirm('Вернуть эту версию? Текущий текст останется в истории.')) return;

		try {
			const doc = await api.restoreRevision(revisionId);
			title = doc.title;
			body = doc.body_md;
			preview = doc.html;
			historyOpen = false;
			toast.success('Версия восстановлена');
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'не удалось восстановить');
		}
	}

	async function save(nextStatus?: 'draft' | 'published') {
		if (categoryId === null) {
			error = 'выберите категорию';
			return;
		}

		const wasPublished = status === 'published';
		const target = nextStatus ?? status;

		saving = true;
		error = '';

		try {
			const payload = {
				category_id: categoryId,
				title,
				slug: slug || undefined,
				status: target,
				body_md: body
			};

			const doc = isNew
				? await api.createPage(payload)
				: await api.updatePage(Number(id), payload);

			title = doc.title;
			slug = doc.slug;
			status = doc.status;

			localStorage.removeItem(draftKey);
			localStorage.removeItem('wiki-draft:new');

			if (isNew) {
				await goto('/admin/page/' + doc.id, { replaceState: true });
			}

			if (target === 'published' && !wasPublished) {
				toast.success('Опубликовано');
			} else if (target === 'draft' && wasPublished) {
				toast.success('Снято с публикации');
			} else {
				toast.success(isNew ? 'Страница создана' : 'Сохранено');
			}
		} catch (e) {
			const message = e instanceof Error ? e.message : 'не сохранилось';
			error = message;
			toast.error(message);
		} finally {
			saving = false;
		}
	}

	async function remove() {
		if (!window.confirm('Удалить страницу?')) return;
		try {
			await api.deletePage(Number(id));
			toast.success('Страница удалена');
			await goto('/admin');
		} catch (e) {
			const message = e instanceof Error ? e.message : 'не удалилось';
			error = message;
			toast.error(message);
		}
	}
</script>

<div class="mb-4 border-b pb-3">
	<div class="flex items-center justify-between gap-4">
		<Button variant="ghost" size="sm" onclick={() => goto('/admin')}>
			<ArrowLeft class="size-4" />
			Админка
		</Button>
		<div class="flex items-center gap-2">
			<Badge variant={status === 'published' ? 'default' : 'outline'}>
				{status === 'published' ? 'опубликовано' : 'черновик'}
			</Badge>
			{#if !isNew}
				<Button variant="outline" size="sm" onclick={openHistory}>
					<History class="size-4" />
					История
				</Button>
				<Button variant="ghost" size="icon" class="text-destructive" title="Удалить" onclick={remove}>
					<Trash2 class="size-4" />
				</Button>
			{/if}
			<Button variant="outline" onclick={() => save()} disabled={saving}>Сохранить</Button>
			{#if status === 'published'}
				<Button variant="outline" onclick={() => save('draft')} disabled={saving}>
					Отменить публикацию
				</Button>
			{:else}
				<Button onclick={() => save('published')} disabled={saving}>Опубликовать</Button>
			{/if}
		</div>
	</div>
</div>

{#if error}
	<div class="mt-4 rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-sm text-destructive">
		{error}
	</div>
{/if}

{#if loading}
	<p class="mt-8 text-sm text-muted-foreground">Загрузка…</p>
{:else}
	<Card.Root class="mt-4">
		<Card.Content class="space-y-4 py-5">
			<div>
				<label for="page-title" class="mb-1.5 block text-xs font-medium text-muted-foreground">
					Заголовок
				</label>
				<Input
					id="page-title"
					class="text-lg font-medium"
					placeholder="Например, «Введение в Rust»"
					bind:value={title}
				/>
			</div>

			<div class="grid gap-3 sm:grid-cols-2">
				<div>
					<label
						for="page-category"
						class="mb-1.5 block text-xs font-medium text-muted-foreground"
					>
						Категория
					</label>
					<select id="page-category" class={selectClass} bind:value={categoryId}>
						{#each rows as row (row.node.id)}
							<option value={row.node.id}>{'— '.repeat(row.depth)}{row.node.name}</option>
						{/each}
					</select>
				</div>

				<div>
					<label for="page-slug" class="mb-1.5 block text-xs font-medium text-muted-foreground">
						Ссылка (slug)
					</label>
					<Input id="page-slug" placeholder="сгенерируется из заголовка" bind:value={slug} />
				</div>
			</div>
		</Card.Content>
	</Card.Root>

	<div class="mt-6">
		<div class="flex items-center justify-between">
			<div class="inline-flex items-center rounded-lg border border-border bg-muted/50 p-0.5">
				<button type="button" class={tabClass(tab === 'edit')} onclick={() => (tab = 'edit')}>
					Текст
				</button>
				<button type="button" class={tabClass(tab === 'preview')} onclick={() => (tab = 'preview')}>
					Превью
				</button>
			</div>
			<span class="text-xs text-muted-foreground">{words} слов · {body.length} симв.</span>
		</div>

		{#if tab === 'edit'}
			<div class="mt-3">
				<div class="mb-2 flex flex-wrap items-center gap-2">
					<Button
						variant="ghost"
						size="icon"
						class="size-8"
						title="Отменить (Ctrl+Z)"
						disabled={!editorEl}
						onclick={() => editorEl?.undo()}
					>
						<Undo2 class="size-4" />
					</Button>
					<Button
						variant="ghost"
						size="icon"
						class="size-8"
						title="Повторить (Ctrl+Shift+Z)"
						disabled={!editorEl}
						onclick={() => editorEl?.redo()}
					>
						<Redo2 class="size-4" />
					</Button>
					<span class="mx-1 h-5 w-px bg-border"></span>
					<Button variant="outline" size="sm" onclick={() => fileInput?.click()}>
						<ImagePlus class="size-4" />
						Картинка
					</Button>
					<Button variant="outline" size="sm" onclick={() => (videoOpen = true)}>
						<Video class="size-4" />
						Видео
					</Button>
					<Button variant="outline" size="sm" onclick={() => (tableOpen = true)}>
						<Table class="size-4" />
						Таблица
					</Button>
					<Button variant="outline" size="sm" onclick={() => (linkOpen = true)}>
						<LinkIcon class="size-4" />
						Ссылка
					</Button>
					<Button variant="outline" size="sm" onclick={() => (calloutOpen = true)}>
						<MessageSquare class="size-4" />
						Врезка
					</Button>
					<span class="text-xs text-muted-foreground">или перетащи файл в текст</span>
					<input
						type="file"
						accept="image/*"
						class="hidden"
						bind:this={fileInput}
						onchange={onFilePicked}
					/>
				</div>

				<div
					role="presentation"
					ondrop={onDrop}
					ondragover={(event) => event.preventDefault()}
					onpaste={onPasteImage}
				>
					<CodeEditor bind:this={editorEl} bind:value={body} />
				</div>
			</div>
		{:else}
			<div class="md mt-3 h-[58vh] overflow-auto rounded-lg border p-6" bind:this={previewEl}>
				{@html preview}
			</div>
		{/if}
	</div>
{/if}

<Dialog.Root bind:open={historyOpen}>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>История версий</Dialog.Title>
			<Dialog.Description>Каждое сохранение создаёт новую версию.</Dialog.Description>
		</Dialog.Header>

		{#if historyLoading}
			<p class="text-sm text-muted-foreground">Загружаю…</p>
		{:else if !revisions.length}
			<p class="text-sm text-muted-foreground">Версий пока нет.</p>
		{:else}
			<div class="max-h-80 space-y-2 overflow-auto pr-1">
				{#each revisions as revision (revision.id)}
					<div class="flex items-center justify-between gap-3 rounded-lg border px-3 py-2">
						<div class="min-w-0">
							<div class="truncate text-sm">{revision.title}</div>
							<div class="text-xs text-muted-foreground">
								{revision.created_at} · {revision.size} симв.
							</div>
							{#if revision.preview}
								<div class="truncate text-xs text-muted-foreground/80">{revision.preview}</div>
							{/if}
						</div>
						<Button variant="outline" size="sm" onclick={() => restore(revision.id)}>
							Вернуть
						</Button>
					</div>
				{/each}
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={videoOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>Видео</Dialog.Title>
			<Dialog.Description>
				Загрузите файл (mp4 или webm) или вставьте ссылку YouTube.
			</Dialog.Description>
		</Dialog.Header>

		<div class="space-y-4">
			<Button variant="outline" class="w-full" onclick={() => videoInput?.click()}>
				<Upload class="size-4" />
				Загрузить файл
			</Button>
			<input
				type="file"
				accept="video/mp4,video/webm"
				class="hidden"
				bind:this={videoInput}
				onchange={onVideoPicked}
			/>

			<div class="flex items-center gap-2">
				<div class="h-px flex-1 bg-border"></div>
				<span class="text-xs text-muted-foreground">или</span>
				<div class="h-px flex-1 bg-border"></div>
			</div>

			<div class="space-y-2">
				<Input
					placeholder="https://youtube.com/watch?v=…"
					bind:value={videoUrl}
					onkeydown={(event: KeyboardEvent) => {
						if (event.key === 'Enter') insertYouTube();
					}}
				/>
				<Button class="w-full" onclick={insertYouTube} disabled={!videoUrl.trim()}>
					Вставить с YouTube
				</Button>
			</div>
		</div>
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={tableOpen}>
	<Dialog.Content class="sm:max-w-sm">
		<Dialog.Header>
			<Dialog.Title>Таблица</Dialog.Title>
			<Dialog.Description>Сколько столбцов и строк добавить?</Dialog.Description>
		</Dialog.Header>

		<div class="grid grid-cols-2 gap-3">
			<div>
				<label for="table-cols" class="mb-1.5 block text-xs font-medium text-muted-foreground">
					Столбцы
				</label>
				<Input id="table-cols" type="number" min="1" max="10" bind:value={tableCols} />
			</div>
			<div>
				<label for="table-rows" class="mb-1.5 block text-xs font-medium text-muted-foreground">
					Строки
				</label>
				<Input id="table-rows" type="number" min="1" max="50" bind:value={tableRows} />
			</div>
		</div>

		<Dialog.Footer>
			<Button variant="outline" onclick={() => (tableOpen = false)}>Отмена</Button>
			<Button onclick={insertTable}>Вставить</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={linkOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>Ссылка</Dialog.Title>
			<Dialog.Description>
				Текст необязателен — если пусто, будет показан сам адрес.
			</Dialog.Description>
		</Dialog.Header>

		<div class="space-y-3">
			<div>
				<label for="link-text" class="mb-1.5 block text-xs font-medium text-muted-foreground">
					Текст
				</label>
				<Input id="link-text" placeholder="Например, документация" bind:value={linkText} />
			</div>
			<div>
				<label for="link-url" class="mb-1.5 block text-xs font-medium text-muted-foreground">
					Адрес
				</label>
				<Input
					id="link-url"
					placeholder="https://…"
					bind:value={linkUrl}
					onkeydown={(event: KeyboardEvent) => {
						if (event.key === 'Enter') insertLink();
					}}
				/>
			</div>
		</div>

		<Dialog.Footer>
			<Button variant="outline" onclick={() => (linkOpen = false)}>Отмена</Button>
			<Button onclick={insertLink} disabled={!linkUrl.trim()}>Вставить</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={calloutOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>Врезка</Dialog.Title>
			<Dialog.Description>Цветной блок для заметки, совета или предупреждения.</Dialog.Description>
		</Dialog.Header>

		<div class="space-y-3">
			<div>
				<label for="callout-type" class="mb-1.5 block text-xs font-medium text-muted-foreground">
					Тип
				</label>
				<select id="callout-type" class={selectClass} bind:value={calloutType}>
					<option value="note">Заметка (note)</option>
					<option value="tip">Совет (tip)</option>
					<option value="warning">Предупреждение (warning)</option>
					<option value="caution">Осторожно (caution)</option>
				</select>
			</div>
			<div>
				<label for="callout-text" class="mb-1.5 block text-xs font-medium text-muted-foreground">
					Текст
				</label>
				<textarea
					id="callout-text"
					class="h-24 w-full resize-none rounded-md border border-input bg-transparent px-3 py-2 text-sm outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50"
					placeholder="Текст врезки (Markdown)"
					bind:value={calloutText}
				></textarea>
			</div>
		</div>

		<Dialog.Footer>
			<Button variant="outline" onclick={() => (calloutOpen = false)}>Отмена</Button>
			<Button onclick={insertCallout}>Вставить</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
