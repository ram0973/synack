<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { ImagePlus, Trash2 } from '@lucide/svelte';
	import { api, ApiError } from '$lib/api';
	import { auth } from '$lib/auth.svelte';

	let loading = $state(true);

	let name = $state('');
	let avatar = $state('');
	let savingProfile = $state(false);
	let avatarInput = $state<HTMLInputElement | null>(null);

	let current = $state('');
	let next = $state('');
	let repeat = $state('');
	let passwordError = $state('');
	let savingPassword = $state(false);

	onMount(async () => {
		await auth.refresh();
		if (!auth.value) {
			await goto('/login');
			return;
		}

		try {
			const settings = await api.settings();
			name = settings.author_name;
			avatar = settings.author_avatar;
		} catch {
			/* пустые настройки — не ошибка */
		} finally {
			loading = false;
		}
	});

	async function uploadAvatar(file: File) {
		try {
			const media = await api.uploadMedia(file);
			avatar = media.url;
			toast.success('Аватар загружен, не забудь сохранить');
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'не загрузилось');
		}
	}

	function onAvatarPicked(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (file) uploadAvatar(file);
		input.value = '';
	}

	async function saveProfile() {
		savingProfile = true;

		try {
			const saved = await api.updateSettings({ author_name: name, author_avatar: avatar });
			name = saved.author_name;
			avatar = saved.author_avatar;
			toast.success('Профиль сохранён');
		} catch (e) {
			toast.error(e instanceof Error ? e.message : 'не удалось сохранить');
		} finally {
			savingProfile = false;
		}
	}

	async function savePassword(event: SubmitEvent) {
		event.preventDefault();
		passwordError = '';

		if (next.length < 8) {
			passwordError = 'Новый пароль должен быть не короче 8 символов';
			return;
		}

		if (next !== repeat) {
			passwordError = 'Пароли не совпадают';
			return;
		}

		savingPassword = true;

		try {
			await api.changePassword(current, next);
			current = '';
			next = '';
			repeat = '';
			await auth.refresh();
			toast.success('Пароль изменён, остальные сессии сброшены');
		} catch (e) {
			passwordError =
				e instanceof ApiError && e.status === 401
					? 'Текущий пароль неверный'
					: e instanceof Error
						? e.message
						: 'не удалось изменить пароль';
		} finally {
			savingPassword = false;
		}
	}
</script>

<div>
	<h1 class="text-3xl font-semibold tracking-tight">Настройки</h1>
	<p class="mt-1 text-sm text-muted-foreground">Профиль автора и пароль от админки.</p>
</div>

{#if !loading}
	<div class="mt-7 grid gap-5 lg:grid-cols-2">
		<Card.Root>
			<Card.Header>
				<Card.Title class="text-base">Профиль</Card.Title>
				<Card.Description>Имя и аватар показываются в сайдбаре.</Card.Description>
			</Card.Header>
			<Card.Content class="space-y-4">
				<div class="flex items-center gap-4">
					{#if avatar}
						<img src={avatar} alt="Аватар" class="size-14 rounded-full object-cover" />
					{:else}
						<div
							class="flex size-14 items-center justify-center rounded-full bg-muted text-sm font-medium uppercase text-muted-foreground"
						>
							{name ? name.slice(0, 2) : '—'}
						</div>
					{/if}

					<div class="flex flex-wrap items-center gap-2">
						<Button variant="outline" size="sm" onclick={() => avatarInput?.click()}>
							<ImagePlus class="size-4" />
							{avatar ? 'Заменить' : 'Загрузить'}
						</Button>
						{#if avatar}
							<Button variant="ghost" size="sm" onclick={() => (avatar = '')}>
								<Trash2 class="size-4" />
								Убрать
							</Button>
						{/if}
						<input
							type="file"
							accept="image/*"
							class="hidden"
							bind:this={avatarInput}
							onchange={onAvatarPicked}
						/>
					</div>
				</div>

				<div>
					<label for="author-name" class="mb-1.5 block text-xs font-medium text-muted-foreground">
						Имя автора
					</label>
					<Input id="author-name" placeholder="Например, Роман" bind:value={name} />
				</div>

				<Button onclick={saveProfile} disabled={savingProfile}>
					{savingProfile ? 'Сохраняю…' : 'Сохранить'}
				</Button>
			</Card.Content>
		</Card.Root>

		<Card.Root>
			<Card.Header>
				<Card.Title class="text-base">Пароль</Card.Title>
				<Card.Description>После смены пароля остальные сессии сбрасываются.</Card.Description>
			</Card.Header>
			<Card.Content>
				<form class="space-y-4" onsubmit={savePassword}>
					<div>
						<label for="pw-current" class="mb-1.5 block text-xs font-medium text-muted-foreground">
							Текущий пароль
						</label>
						<Input
							id="pw-current"
							type="password"
							autocomplete="current-password"
							bind:value={current}
						/>
					</div>

					<div>
						<label for="pw-next" class="mb-1.5 block text-xs font-medium text-muted-foreground">
							Новый пароль
						</label>
						<Input id="pw-next" type="password" autocomplete="new-password" bind:value={next} />
					</div>

					<div>
						<label for="pw-repeat" class="mb-1.5 block text-xs font-medium text-muted-foreground">
							Ещё раз
						</label>
						<Input id="pw-repeat" type="password" autocomplete="new-password" bind:value={repeat} />
					</div>

					{#if passwordError}
						<p class="text-sm text-destructive">{passwordError}</p>
					{/if}

					<Button type="submit" disabled={savingPassword || !current || !next || !repeat}>
						{savingPassword ? 'Меняю…' : 'Сменить пароль'}
					</Button>
				</form>
			</Card.Content>
		</Card.Root>
	</div>
{/if}
