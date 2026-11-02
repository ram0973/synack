<script lang="ts">
	import { goto } from '$app/navigation';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Lock } from '@lucide/svelte';
	import { api, ApiError } from '$lib/api';
	import { auth } from '$lib/auth.svelte';

	let password = $state('');
	let error = $state('');
	let busy = $state(false);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		busy = true;
		error = '';

		try {
			await api.login(password);
			auth.set(true);
			await goto('/admin');
		} catch (e) {
			error =
				e instanceof ApiError && e.status === 401
					? 'Неверный пароль'
					: e instanceof Error
						? e.message
						: 'не удалось войти';
		} finally {
			busy = false;
		}
	}
</script>

<div class="mx-auto mt-16 max-w-sm">
	<div class="mb-6 flex items-center gap-2">
		<div class="flex size-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
			<Lock class="size-4" />
		</div>
		<div>
			<h1 class="text-lg font-semibold tracking-tight">Вход</h1>
		</div>
	</div>

	<Card.Root>
		<Card.Content class="py-5">
			<form onsubmit={submit} class="space-y-3">
				<Input type="password" placeholder="Пароль" bind:value={password} autocomplete="current-password" />

				{#if error}
					<p class="text-sm text-destructive">{error}</p>
				{/if}

				<Button type="submit" class="w-full" disabled={busy || !password}>
					{busy ? 'Проверяю…' : 'Войти'}
				</Button>
			</form>
		</Card.Content>
	</Card.Root>
</div>
