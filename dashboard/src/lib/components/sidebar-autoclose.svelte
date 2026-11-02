<script lang="ts">
	import { page } from '$app/state';
	import { useSidebar } from '$lib/components/ui/sidebar/index.js';

	const sidebar = useSidebar();

	// на мобильных и в узком окне панель должна закрываться после перехода
	$effect(() => {
		page.url.pathname;
		sidebar.setOpenMobile(false);
	});
</script>
