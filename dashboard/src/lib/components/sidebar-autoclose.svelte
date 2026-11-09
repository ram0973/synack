<script lang="ts">
	import { page } from '$app/state';
	import { useSidebar } from '$lib/components/ui/sidebar/index.js';

	const sidebar = useSidebar();

	// страховка на случай перехода, который не меняет путь (например, «Главная» на главной)
	$effect(() => {
		page.url.pathname;
		sidebar.setOpenMobile(false);
	});

	// закрываем панель по клику на любой пункт меню или ссылку
	function onClick(event: MouseEvent) {
		const target = event.target as HTMLElement | null;
		if (!target) return;

		if (target.closest('a[href], [data-sidebar="menu-button"]')) {
			sidebar.setOpenMobile(false);
		}
	}
</script>

<svelte:window onclick={onClick} />
