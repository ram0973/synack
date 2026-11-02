<script lang="ts">
import { wikiHref } from '$lib/paths';
	import '@fontsource-variable/inter';
	import '../app.css';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { ModeWatcher, mode, toggleMode } from 'mode-watcher';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Toaster } from '$lib/components/ui/sonner/index.js';
	import {
		BookOpen,
		ChevronDown,
		ChevronRight,
		FileText,
		Folder,
		House,
		KeyRound,
		LogIn,
		LogOut,
		Moon,
		Search,
		Settings,
		Sun,
		Terminal
	} from '@lucide/svelte';
	import { api, type CategoryNode, type PageSummary } from '$lib/api';
	import { auth } from '$lib/auth.svelte';

	let { children } = $props();

	let tree = $state<CategoryNode[]>([]);
	let pages = $state<PageSummary[]>([]);
	let error = $state('');
	let searchTerm = $state('');
	let expanded = $state<Set<number>>(new Set());

	type Entry =
		| {
				kind: 'category';
				id: number;
				parentId: number | null;
				depth: number;
				path: string;
				name: string;
				hasChildren: boolean;
		  }
		| { kind: 'page'; id: number; depth: number; path: string; title: string };

	async function load() {
		try {
			const [t, p] = await Promise.all([api.tree(), api.pages()]);
			tree = t;
			pages = p;
			error = '';
		} catch (e) {
			error = e instanceof Error ? e.message : 'не удалось загрузить дерево';
		}
	}

	$effect(() => {
		page.url.pathname;
		load();
		auth.refresh();
	});

	async function signOut() {
		try {
			await api.logout();
		} catch {
			/* ignore */
		}
		auth.set(false);
		await goto('/login');
	}

	const current = $derived(page.url.pathname);
	const activePath = $derived(current.replace(/^\//, ''));

	const pagesByCategory = $derived.by(() => {
		const map = new Map<number, PageSummary[]>();

		for (const item of pages) {
			const list = map.get(item.category_id) ?? [];
			list.push(item);
			map.set(item.category_id, list);
		}

		for (const list of map.values()) {
			list.sort((a, b) => a.title.localeCompare(b.title));
		}

		return map;
	});

	function isOpen(id: number, path: string) {
		return expanded.has(id) || activePath.startsWith(path + '/');
	}

	function buildEntries(nodes: CategoryNode[], depth: number, prefix: string): Entry[] {
		const out: Entry[] = [];

		for (const node of nodes) {
			const path = prefix ? prefix + '/' + node.slug : node.slug;
			const children = node.children ?? [];
			const items = pagesByCategory.get(node.id) ?? [];

			out.push({
				kind: 'category',
				id: node.id,
				parentId: node.parent_id,
				depth,
				path,
				name: node.name,
				hasChildren: children.length > 0 || items.length > 0
			});

			if (isOpen(node.id, path)) {
				out.push(...buildEntries(children, depth + 1, path));

				for (const item of items) {
					out.push({
						kind: 'page',
						id: item.id,
						depth: depth + 1,
						path: path + '/' + item.slug,
						title: item.title
					});
				}
			}
		}

		return out;
	}

	const entries = $derived(buildEntries(tree, 0, ''));

	const parentOf = $derived.by(() => {
		const map = new Map<number, number | null>();
		const stack: CategoryNode[] = [...tree];

		while (stack.length) {
			const node = stack.pop();
			if (!node) break;
			map.set(node.id, node.parent_id);
			stack.push(...node.children);
		}

		return map;
	});

	function toggleCategory(id: number, parentId: number | null) {
		const next = new Set(expanded);

		if (next.has(id)) {
			next.delete(id);
		} else {
			// на одном уровне открыт только один раздел
			for (const other of [...next]) {
				if ((parentOf.get(other) ?? null) === parentId) next.delete(other);
			}
			next.add(id);
		}

		expanded = next;
	}
</script>

<svelte:head>
	<title>Synack</title>
</svelte:head>

<ModeWatcher />
<Toaster position="bottom-right" />

<Sidebar.Provider style="--sidebar-width: 15.5rem">
	<Sidebar.Root collapsible="offcanvas" class="border-r">
		<Sidebar.Header>
			<a
				href="/"
				class="flex items-center gap-2 px-2 py-1 text-inherit no-underline transition-opacity hover:opacity-80"
			>
				<div
					class="flex size-6 items-center justify-center rounded-md bg-primary text-primary-foreground"
				>
					<Terminal class="size-3.5" />
				</div>
				<span class="text-[15px] font-semibold tracking-tight">Synack</span>
			</a>

			<form
				class="mt-3 px-1"
				onsubmit={(event) => {
					event.preventDefault();
					const value = searchTerm.trim();
					if (value) goto('/wiki/search?q=' + encodeURIComponent(value));
				}}
			>
				<div class="relative">
					<Search
						class="pointer-events-none absolute left-2 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground"
					/>
					<Input class="h-8 pl-7 text-sm" placeholder="Поиск" bind:value={searchTerm} />
				</div>
			</form>
		</Sidebar.Header>

		<Sidebar.Content>
			<Sidebar.Group>
				<Sidebar.GroupLabel>Навигация</Sidebar.GroupLabel>
				<Sidebar.GroupContent>
					<Sidebar.Menu>
						<Sidebar.MenuItem>
							<Sidebar.MenuButton isActive={current === '/'} onclick={() => goto('/')}>
								<House />
								<span>Главная</span>
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
						<Sidebar.MenuItem>
							<Sidebar.MenuButton
								isActive={current === '/wiki' || current.startsWith('/wiki/')}
								onclick={() => goto('/wiki')}
							>
								<BookOpen />
								<span>Вики</span>
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
						{#if auth.value}
							<Sidebar.MenuItem>
								<Sidebar.MenuButton
									isActive={current === '/admin' || current.startsWith('/admin/')}
									onclick={() => goto('/admin')}
								>
									<Settings />
									<span>Админка</span>
								</Sidebar.MenuButton>
							</Sidebar.MenuItem>
						{/if}
					</Sidebar.Menu>
				</Sidebar.GroupContent>
			</Sidebar.Group>

			<Sidebar.Group>
				<Sidebar.GroupLabel>Категории</Sidebar.GroupLabel>
				<Sidebar.GroupContent>
					<Sidebar.Menu>
						{#each entries as entry (entry.kind + '-' + entry.id)}
							<Sidebar.MenuItem>
								{#if entry.kind === 'category'}
									<Sidebar.MenuButton
										isActive={current === '/wiki/' + entry.path}
										onclick={() => toggleCategory(entry.id, entry.parentId)}
										style={'padding-left: ' + (8 + entry.depth * 12) + 'px'}
									>
										<Folder class="size-4 shrink-0 text-muted-foreground" />
										<span class="truncate">{entry.name}</span>
									</Sidebar.MenuButton>

									{#if entry.hasChildren}
										<Sidebar.MenuAction
											title={isOpen(entry.id, entry.path) ? 'Свернуть' : 'Развернуть'}
											aria-expanded={isOpen(entry.id, entry.path)}
											onclick={() => toggleCategory(entry.id, entry.parentId)}
										>
											{#if isOpen(entry.id, entry.path)}
												<ChevronDown />
											{:else}
												<ChevronRight />
											{/if}
										</Sidebar.MenuAction>
									{/if}
								{:else}
									<Sidebar.MenuButton
										isActive={current === '/' + entry.path}
										onclick={() => goto(wikiHref(entry.path))}
										style={'padding-left: ' + (8 + entry.depth * 12) + 'px'}
									>
										<FileText class="size-4 shrink-0 text-muted-foreground" />
										<span class="truncate">{entry.title}</span>
									</Sidebar.MenuButton>
								{/if}
							</Sidebar.MenuItem>
						{/each}

						{#if !entries.length && !error}
							<p class="px-2 py-1 text-xs text-muted-foreground">пока пусто</p>
						{/if}
						{#if error}
							<p class="px-2 py-1 text-xs text-destructive">{error}</p>
						{/if}
					</Sidebar.Menu>
				</Sidebar.GroupContent>
			</Sidebar.Group>
		</Sidebar.Content>

		<Sidebar.Footer>
			<div class="flex items-center justify-between px-1 py-1">
				<span class="text-[11px] text-muted-foreground">v0.1.0</span>
				<div class="flex items-center gap-1">
					<Button
						variant="ghost"
						size="icon"
						class="size-7"
						title="Сменить тему"
						onclick={() => toggleMode()}
					>
						{#if mode.current === 'dark'}
							<Sun class="size-4" />
						{:else}
							<Moon class="size-4" />
						{/if}
					</Button>

					{#if auth.value}
						<Button
							variant="ghost"
							size="icon"
							class="size-7"
							title="Пароль"
							onclick={() => goto('/admin/password')}
						>
							<KeyRound class="size-4" />
						</Button>
						<Button variant="ghost" size="icon" class="size-7" title="Выйти" onclick={signOut}>
							<LogOut class="size-4" />
						</Button>
					{:else}
						<Button
							variant="ghost"
							size="icon"
							class="size-7"
							title="Войти"
							onclick={() => goto('/login')}
						>
							<LogIn class="size-4" />
						</Button>
					{/if}
				</div>
			</div>
		</Sidebar.Footer>
	</Sidebar.Root>

	<Sidebar.Inset>
		<header
			class="sticky top-0 z-10 flex h-12 shrink-0 items-center gap-2 border-b bg-background/80 px-4 backdrop-blur"
		>
			<Sidebar.Trigger class="-ml-1" />
			<span class="text-sm font-medium text-muted-foreground">
				{current === '/'
					? 'Главная'
					: current.startsWith('/admin/wiki/page')
						? 'Редактор'
						: current.startsWith('/admin/password')
							? 'Пароль'
							: current.startsWith('/admin')
								? 'Админка'
								: current.startsWith('/wiki/search')
									? 'Поиск'
									: current.startsWith('/login')
										? 'Вход'
										: 'Статья'}
			</span>
		</header>

		<div class="mx-auto w-full max-w-6xl px-10 py-10">
			{#if auth.value && auth.defaultPassword && current.startsWith('/admin')}
				<div
					class="mb-6 flex flex-wrap items-center justify-between gap-3 rounded-lg border border-amber-300 bg-amber-50 px-4 py-3 text-sm text-amber-900 dark:border-amber-500/40 dark:bg-amber-500/10 dark:text-amber-200"
				>
					<span>
						Вы вошли со стандартным паролем — смените его, иначе в админку сможет войти любой.
					</span>
					<a href="/admin/password" class="font-medium underline underline-offset-2">
						Сменить пароль
					</a>
				</div>
			{/if}

			{@render children()}
		</div>
	</Sidebar.Inset>
</Sidebar.Provider>
