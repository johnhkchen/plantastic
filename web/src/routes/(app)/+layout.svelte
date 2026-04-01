<script lang="ts">
	import Sidebar from '$lib/components/Sidebar.svelte';
	import Header from '$lib/components/Header.svelte';

	let { children } = $props();
	let sidebarOpen = $state(false);

	function toggleSidebar() {
		sidebarOpen = !sidebarOpen;
	}

	function closeSidebar() {
		sidebarOpen = false;
	}
</script>

<svelte:window onkeydown={(e) => { if (e.key === 'Escape') closeSidebar(); }} />

<div class="flex h-screen overflow-hidden bg-surface-alt">
	<!-- Mobile backdrop -->
	{#if sidebarOpen}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="fixed inset-0 z-40 bg-black/50 md:hidden"
			onclick={closeSidebar}
			onkeydown={(e) => { if (e.key === 'Escape') closeSidebar(); }}
		></div>
	{/if}

	<Sidebar open={sidebarOpen} onClose={closeSidebar} />

	<div class="flex flex-1 flex-col overflow-hidden">
		<Header onToggleSidebar={toggleSidebar} />

		<main class="flex-1 overflow-y-auto p-6">
			{@render children()}
		</main>
	</div>
</div>
