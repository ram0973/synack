<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { api, ApiError } from '$lib/api';
	import { auth } from '$lib/auth.svelte';

	let current = $state('');
	let next = $state('');
	let repeat = $state('');
	let error = $state('');
	let saving = $state(false);
	let loading = $state(true);

	onMount(async () => {
		await auth.refresh();

		if (!auth.value) {
			await goto('/login');
			return;
		}

		loading = false;
	});

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		error = '';

		if (next.length < 8) {
			error = 'Новый пароль должен быть не короче 8 символов';
			return;
		}

		if (next !== repeat) {
			error = 'Пароли не совпадают';
			return;
		}

		saving = true;

		try {
			await api.changePassword(current, next);
			current = '';
			next = '';
			repeat = '';
			await auth.refresh();
			toast.success('Пароль изменён, остальные сессии сброшены');
		} catch (e) {
			error =
				e instanceof ApiError && e.status === 401
					? 'Текущий пароль неверный'
					: e instanceof Error
						? e.message
						: 'не удалось изменить пароль';
		} finally {
			saving = false;
		}
	}
</script>

<div>
	<h1 class="text-2xl font-semibold tracking-tight">Пароль</h1>
	<p class="mt-1 text-sm text-muted-foreground">
		Меняется здесь и хранится в базе хешем argon2id. Остальные сессии после смены сбрасываются,
		текущая — остаётся.
	</p>
</div>

{#if !loading}
	<Card.Root class="mt-8 max-w-md">
		<Card.Content class="py-5">
			<form class="space-y-4" onsubmit={submit}>
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

				{#if error}
					<p class="text-sm text-destructive">{error}</p>
				{/if}

				<Button type="submit" disabled={saving || !current || !next || !repeat}>
					{saving ? 'Меняю…' : 'Сменить пароль'}
				</Button>
			</form>
		</Card.Content>
	</Card.Root>
{/if}
