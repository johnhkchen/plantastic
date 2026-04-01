<script lang="ts">
	import { session } from '$lib/stores/session.svelte';

	let { onToggleSidebar }: { onToggleSidebar?: () => void } = $props();
</script>

<header class="flex h-14 items-center justify-between border-b border-border bg-surface px-6">
	<div class="flex items-center gap-2">
		<button
			type="button"
			class="flex min-h-[44px] min-w-[44px] items-center justify-center rounded-md text-text-secondary hover:bg-surface-hover md:hidden"
			onclick={onToggleSidebar}
			aria-label="Toggle sidebar"
		>
			<svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
				<path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
			</svg>
		</button>
		<h2 class="font-display text-sm font-semibold text-text-secondary">{session.tenant}</h2>
	</div>

	<div class="flex items-center gap-3">
		<div
			class="flex h-8 w-8 items-center justify-center rounded-full bg-primary-light text-xs font-medium text-text-invert"
		>
			JC
		</div>
	</div>
</header>
